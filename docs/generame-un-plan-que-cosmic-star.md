# Plan — Fix de bugs críticos detectados en instalación real

## Context

El usuario probó el instalador en un sistema ajeno. La app abre y muestra el dashboard, pero presenta defectos serios que la vuelven inutilizable. Cada fix está fragmentado en pasos pequeños e independientes para que la IA pueda ejecutarlos uno a uno sin perder el hilo.

---

## Fix 1 — Consolas PowerShell parpadeantes

### Problema

Cada ~1.5 s aparece una ventana negra de PowerShell con título "Administrador:" que roba foco y bloquea la navegación.

**Causa raíz**: la telemetría de GPU lanza `powershell.exe` sin el flag `CREATE_NO_WINDOW` desde `src-tauri/src/platform/gpu.rs:74-81`, y el hook `use-telemetry.ts` la invoca con `refetchInterval: 1500`.

### Paso 1.1 — Crear helper centralizado de PowerShell

**Archivo**: `src-tauri/src/platform/powershell.rs` (nuevo)

Crear un módulo Rust que centralice TODA invocación a `powershell.exe`. Requisitos:

- Flag `CREATE_NO_WINDOW` obligatorio en Windows.
- Timeout configurable (default 20 s).
- Solo acepta `&'static str` como script — impide inyección de input dinámico.
- Captura stdout/stderr como UTF-8.
- Retorna `AppResult<Output>`.

```rust
use std::process::{Command, Output};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use crate::core::{AppError, AppResult};
use std::time::Duration;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);

pub fn run_script(script: &'static str) -> AppResult<Output> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile", "-NonInteractive",
        "-ExecutionPolicy", "Bypass",
        "-Command", script,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map_err(|e| AppError::Io(e.to_string()))
}
```

### Paso 1.2 — Exportar módulo en `platform/mod.rs`

**Archivo**: `src-tauri/src/platform/mod.rs`

Añadir línea: `pub mod powershell;`

Verificar que ya exporta `gpu`, `elevation`, `filesystem`.

### Paso 1.3 — Migrar `gpu.rs` al helper

**Archivo**: `src-tauri/src/platform/gpu.rs`

- Localizar líneas 74-81 donde se hace `Command::new("powershell").args([...]).output()`.
- Reemplazar por `powershell::run_script(PS_SCRIPT)` donde `PS_SCRIPT` es un literal `&'static str` con el script embebido.
- Añadir `use crate::platform::powershell;` en los imports.

### Paso 1.4 — Buscar y migrar otras invocaciones de PowerShell

Ejecutar búsqueda global por `Command::new("powershell")` en todo `src-tauri/src/`.

Archivos sospechosos de tener invocaciones:
- `platform/cpu.rs`
- `platform/storage.rs`
- `domain/debloat.rs`

Migrar cada una al helper `powershell::run_script()`. Si alguna necesita argumentos dinámicos, validar contra allowlist antes de construir el script.

### Paso 1.5 — Reducir polling de telemetría

**Archivo**: `src/hooks/use-telemetry.ts`

Cambiar `refetchInterval` de `1500` a `3000` ms.

Justificación: aun con `CREATE_NO_WINDOW`, lanzar PowerShell 40 veces/min es desperdicio de CPU. 3 s es suficiente para telemetría en vivo.

### Verificación Fix 1

1. `npm run tauri dev`
2. Abrir dashboard, esperar 60 s
3. **Aceptación**: ninguna ventana de consola aparece
4. Verificar que la sección GPU sigue mostrando datos correctamente

---

## Fix 2 — UAC no salta al inicio

### Problema

La app no pide elevación al arrancar, contradiciendo el principio "elevación al inicio". Hay un typo en el manifest y el instalador NSIS no está configurado para elevarse.

### Paso 2.1 — Corregir typo en `build.rs`

**Archivo**: `src-tauri/build.rs`, línea 38

Cambiar:
```xml
<dpiAware>true/PM</dpiAware>
```
Por:
```xml
<dpiAware>True/PM</dpiAware>
```

Añadir también el campo moderno:
```xml
<dpiAwareness>PerMonitorV2</dpiAwareness>
```

### Paso 2.2 — Verificar embed del manifest en release

Tras compilar en release, verificar que el manifest se embebió correctamente:

```
mt.exe -inputresource:ClearTool.exe;#1 -out:check.xml
```

O alternativamente con `sigcheck -m`. Confirmar que muestra `level="requireAdministrator"`.

### Paso 2.3 — Configurar instalador NSIS para pedir UAC

**Archivo**: `src-tauri/tauri.conf.json`

Editar la sección `bundle.windows.nsis`:

```json
"nsis": {
  "installerIcon": "icons/icon.ico",
  "installMode": "perMachine",
  "displayLanguageSelector": false
}
```

`installMode: perMachine` obliga al instalador a pedir UAC porque instala en `Program Files`.

### Paso 2.4 — Verificar script NSIS tiene `RequestExecutionLevel admin`

Comprobar que el template NSIS que inyecta Tauri incluye `RequestExecutionLevel admin`. Si no, añadir hook NSIS personalizado vía `"installerHooks"` o un template propio.

### Paso 2.5 — Refactor de `elevation.rs`: función con retorno bool

**Archivo**: `src-tauri/src/platform/elevation.rs`

Renombrar `relaunch_as_admin_if_needed()` → `try_relaunch_as_admin() -> bool`.

- Retorna `true` si se relanzó como admin (el proceso hijo toma el control).
- Retorna `false` si el usuario canceló UAC o falló el relaunch.

### Paso 2.6 — Lógica de "intento de elevación + modo limitado" en `lib.rs`

**Archivo**: `src-tauri/src/lib.rs`, sección setup (líneas 25-30 aprox.)

Reemplazar la lógica actual por:

```rust
#[cfg(all(target_os = "windows", not(debug_assertions)))]
{
    if !platform::elevation::is_elevated() {
        if platform::elevation::try_relaunch_as_admin() {
            std::process::exit(0);
        }
        // Usuario canceló UAC → app sigue en modo lectura
    }
}
```

### Paso 2.7 — Banner de "Modo limitado" en frontend

**Archivo**: `src/components/layout/app-shell.tsx`

- Leer `isElevated` desde `src/hooks/use-elevation.ts`.
- Si `isElevated === false`, mostrar banner amarillo/naranja con texto: "Modo limitado: solo lectura. [Reabrir como administrador]".
- Deshabilitar botones de acciones destructivas (cache clean, debloat, registry write, services state change) cuando no hay elevación.

### Verificación Fix 2

1. `npm run tauri build` → genera `target/release/bundle/nsis/*.exe`
2. Instalar en VM Windows 11 limpia
3. Al abrir: prompt UAC aparece → aceptar → app abre elevada → `isElevated = true` → sin banner
4. Reabrir y cancelar UAC → app abre → banner visible → botones destructivos deshabilitados

---

## Fix 3 — Catálogos embebidos en binario (Cache no aparece)

### Problema

El comando `list_cache_locations` resuelve catálogo desde rutas relativas a `cwd` / `current_exe()`. En una instalación real, `.claude/skills/cache-scanner/RESOURCES/cache-locations.json` no se distribuye, así que `load_json` devuelve `[]` silenciosamente. La UI muestra `EmptyState` sin error.

### Paso 3.1 — Refactor de `domain/catalog.rs` con `include_str!`

**Archivo**: `src-tauri/src/domain/catalog.rs`

Reescribir para usar `include_str!` con rutas relativas al crate. Eliminar toda la lógica de `catalog_candidates` y rutas filesystem.

```rust
use crate::core::{AppError, AppResult};
use crate::models::cache::{CacheCatalogFile, CacheLocation};

const CACHE_LOCATIONS_JSON: &str = include_str!(
    "../../../.claude/skills/cache-scanner/RESOURCES/cache-locations.json"
);
const BLOATWARE_CATALOG_JSON: &str = include_str!(
    "../../../.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json"
);

pub fn load_cache_locations() -> AppResult<Vec<CacheLocation>> {
    let envelope: CacheCatalogFile = serde_json::from_str(CACHE_LOCATIONS_JSON)
        .map_err(|e| AppError::Catalog(format!("cache-locations.json: {e}")))?;
    Ok(envelope.entries)
}
```

**Notas clave**:
- `include_str!` resuelve en tiempo de compilación → el JSON queda inlineado en el `.exe`.
- Si el JSON falla al deserializar → `AppError::Catalog` con mensaje claro. NUNCA devolver `[]` silencioso.
- Cero dependencias de filesystem en runtime para los catálogos.

### Paso 3.2 — Añadir validación al inicio de la app

**Archivo**: `src-tauri/src/lib.rs`, función `setup`

Cargar los catálogos una vez al arranque y abortar con mensaje útil si fallan:

```rust
.setup(|app| {
    let _ = domain::catalog::load_cache_locations()
        .expect("cache-locations.json embebido es inválido");
    // ...resto del setup
})
```

Esto detecta JSON corrupto en debug en vez de en el primer click de la UI.

### Paso 3.3 — Verificar paths exactos de los JSON

Confirmar existencia de:
- `.claude/skills/cache-scanner/RESOURCES/cache-locations.json` (ya verificado)
- `.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json` (verificar antes de añadir el `include_str!`)

Si alguno no existe, NO añadir su `include_str!` hasta que se cree el archivo.

### Verificación Fix 3

1. Navegar a la pestaña Cache
2. **Aceptación**: lista poblada con entradas del JSON (Update, DISM, Prefetch, thumbcache, etc.)
3. Prueba negativa: borrar temporalmente el JSON del repo, recompilar → error explícito en consola, NO `[]` silencioso

---

## Fix 4 — Explorer: árbol expandible con tamaños lazy

### Problema

El backend devuelve `size = 0` para directorios por diseño. El cálculo debía hacerse lazy al expandir, pero el frontend renderiza una tabla plana sin expansión. La columna "Tamaño" muestra "—" para todo directorio. `compute_directory_size` nunca se invoca.

### Paso 4.1 — Confirmar que el backend está listo

**Archivos**: `src-tauri/src/ipc/explorer.rs`, `src-tauri/src/lib.rs`

Verificar que:
- `compute_directory_size(path)` está registrado como Tauri command.
- `compute_directory_size` emite eventos de progreso `explorer:size-progress` o devuelve resultado parcial cancelable.
- `scan_tree` con `maxDepth: 1` devuelve nodos con `kind: "Dir" | "File"` y `path` único.

Si todo está OK, no se requieren cambios de backend en este paso.

### Paso 4.2 — Crear hook `use-explorer-tree.ts`

**Archivo**: `src/features/explorer/use-explorer-tree.ts` (nuevo)

Hook con la lógica de carga lazy + cache de nodos. Estado:

```ts
type TreeState = {
  rootPath: string;
  byPath: Map<string, NodeState>;
};

type NodeState = {
  children: TreeNode[] | "loading" | "error";
  size: number | "computing" | undefined;
  expanded: boolean;
};
```

Funciones del hook:
- `scanRoot(path)`: invoca `scanTree({ root, maxDepth: 1 })` y pobla el primer nivel.
- `expandNode(path)`: si no expandido, marca `expanded: true`, escanea hijos si no existen, dispara `compute_directory_size` en paralelo.
- `collapseNode(path)`: marca `expanded: false`, no borra datos cacheados.
- `cancelAll()`: invoca `cancelScan` y limpia estados `"computing"` pendientes.

### Paso 4.3 — Crear componente `tree-view.tsx`

**Archivo**: `src/features/explorer/tree-view.tsx` (nuevo)

Componente contenedor del árbol. Recibe:
- `rootPath`: ruta raíz a escanear.
- `nodes`: array de nodos del primer nivel.
- `onExpand`: callback para expandir un nodo.
- `onCancel`: callback para cancelar operaciones en curso.

Renderiza una lista de `TreeRow` con indentación según profundidad.

### Paso 4.4 — Crear componente `tree-row.tsx`

**Archivo**: `src/features/explorer/tree-row.tsx` (nuevo)

Fila individual recursiva. Cada fila muestra:
- Chevron expandir/colapsar (solo para directorios).
- Icono: `Folder` / `FolderOpen` / `File` / `FileText` / `FileImage` / `FileCode` (desde `lucide-react`) según tipo y extensión.
- Nombre del archivo/carpeta con truncate + tooltip si es muy largo.
- Tamaño formateado con `formatBytes()`, spinner si está `"computing"`, "—" si `undefined`.
- Indentación: cada nivel añade 20px de padding-left + línea vertical sutil.

### Paso 4.5 — Reescribir `explorer-page.tsx`

**Archivo**: `src/features/explorer/explorer-page.tsx`

Reemplazar la tabla plana por la estructura de árbol:

```
<ExplorerPage>
  ├─ Header: input ruta + botón Escanear + breadcrumb de path actual
  ├─ Toolbar: ordenar por (Tamaño / Nombre / Modificado), buscar
  └─ TreeView
       └─ TreeRow (recursivo)
```

Comportamiento:
1. **Escanear**: invoca `scanTree({ root, maxDepth: 1 })`. Los eventos `explorer:node` pueblan los hijos.
2. **Click en chevron de Dir**: expande, escanea hijos si no existen, calcula tamaño en paralelo.
3. **Click en chevron de Dir expandido**: colapsa sin perder datos.
4. **Cancelar**: botón visible mientras hay tamaños calculándose.

### Paso 4.6 — Mejoras de UX

Implementar en los componentes del árbol:

- **Colores por tamaño**: texto verde si <1MB, amarillo 1MB-1GB, rojo >1GB.
- **Búsqueda local**: input que filtra el árbol ya cargado (no escanea más, solo oculta filas que no matchean).
- **Loading skeleton**: mientras el primer nivel se escanea, mostrar 5-10 filas skeleton en lugar del `EmptyState` actual.

### Verificación Fix 4

1. Click "Escanear" sobre `C:\` → primer nivel aparece en <5 s con iconos, ordenado.
2. Click chevron en `C:\Windows` → se expande mostrando hijos, spinner en columna tamaño que tras unos segundos muestra el tamaño total recursivo.
3. Click chevron de nuevo → colapsa sin perder el tamaño calculado.
4. Click cancelar mientras un tamaño se calcula → spinner desaparece, app sigue responsiva.
5. Probar con `C:\Users\<user>\Downloads` → búsqueda local filtra el árbol.

---

## Fix 5 — Cache no se borra completamente + consola de debug

### Problema

Al seleccionar todas las checkboxes de caché y ejecutar la eliminación, **queda mucho espacio ocupado en las carpetas**. Los archivos no se eliminan completamente. Además, el usuario no tiene visibilidad de qué carpetas se están procesando en tiempo real.

### Paso 5.1 — Diagnosticar por qué quedan archivos residuales

Investigar el comando Tauri de limpieza de caché actual (`clean_cache` o equivalente). Posibles causas:

- **Archivos en uso**: Windows bloquea archivos que están siendo usados por procesos activos. El borrador debe usar `MoveFileEx` con `MOVEFILE_DELAY_UNTIL_REBOOT` para estos casos, o reportarlos como "no se pudo eliminar".
- **Permisos insuficientes**: algunas carpetas de caché requieren privilegios de TrustedInstaller o SYSTEM. Verificar que la app corre elevada y usa `SE_TAKE_OWNERSHIP_NAME` si es necesario.
- **Reparse points / junctions**: el scanner puede estar saltándose junctions que apuntan a directorios con caché real.
- **Filtros de extensión demasiado agresivos**: puede haber un filtro que excluye archivos que sí deberían borrarse.
- **Profundidad de escaneo limitada**: si el scanner solo mira el nivel superficial de cada carpeta de caché, los subdirectorios anidados quedan intactos.
- **Errores silenciosos**: el código puede estar capturando errores de IO y continuando sin reportarlos.

### Paso 5.2 — Implementar borrado recursivo robusto

**Archivo**: módulo de limpieza de caché (localizar en `src-tauri/src/domain/` o `src-tauri/src/platform/`)

Reescribir la función de borrado para que:

1. **Recorra recursivamente** todos los subdirectorios de cada ubicación de caché seleccionada.
2. **Intente eliminar cada archivo individualmente**, capturando errores por archivo (no por carpeta completa).
3. **Maneje archivos en uso**: si un archivo está bloqueado, intentar:
   - Primero: cambiar atributos a normal (`SetFileAttributesW`).
   - Segundo: si sigue bloqueado, marcar para borrado en el próximo reboot (`MoveFileEx` con `MOVEFILE_DELAY_UNTIL_REBOOT`).
   - Tercero: registrar en la lista de "no eliminados" con razón.
4. **Elimine directorios vacíos** tras borrar su contenido (de dentro hacia fuera).
5. **No falle silenciosamente**: cada error se registra y se reporta al frontend.

### Paso 5.3 — Añadir consola de debug en tiempo real

**Frontend**: crear componente `DebugConsole` en `src/components/debug/debug-console.tsx`

- Panel colapsable en la parte inferior de la pantalla (altura ~150px, redimensionable).
- Fondo oscuro, texto monoespaciado, estilo terminal.
- Scroll automático hacia abajo cuando llegan nuevos mensajes.
- Botones: limpiar consola, copiar log, toggle on/off.

**Backend**: modificar el comando de limpieza de caché para emitir eventos de progreso detallados:

```rust
// Ejemplo de eventos a emitir durante el borrado:
app.emit("cache:debug", CacheDebugEvent {
    level: "info",       // info, warn, error
    message: "Eliminando C:\\Windows\\Temp\\*.tmp...",
    path: "C:\\Windows\\Temp",
    files_deleted: 142,
    bytes_freed: 52428800,
});
```

Eventos a emitir por cada carpeta de caché procesada:
- `[INFO] Iniciando escaneo de: <path>`
- `[INFO] Encontrados X archivos, Y MB en total`
- `[INFO] Eliminando: <subpath>` (por cada subdirectorio/archivo)
- `[WARN] Archivo en uso, se eliminará en el próximo reboot: <path>`
- `[ERROR] Permiso denegado: <path>`
- `[INFO] Completado: Z archivos eliminados, W MB liberados`

**Frontend hook**: `src/hooks/use-cache-debug.ts`

- Suscribirse a eventos `cache:debug`.
- Mantener un buffer circular de los últimos 500 mensajes.
- Exponer estado `messages: DebugMessage[]` al componente `DebugConsole`.

### Paso 5.4 — Ampliar catálogo de carpetas de caché de Windows

**Archivo**: `.claude/skills/cache-scanner/RESOURCES/cache-locations.json`

El catálogo actual es insuficiente. Añadir MUCHAS más ubicaciones de caché conocidas de Windows 11. Categorías a cubrir:

#### Caché del sistema operativo
- `C:\Windows\Temp\*` (temporal del SO)
- `C:\Windows\SoftwareDistribution\Download\*` (actualizaciones descargadas)
- `C:\Windows\SoftwareDistribution\DataStore\Logs\*`
- `C:\Windows\Prefetch\*` (prefetch de aplicaciones)
- `C:\Windows\Logs\*` (logs del sistema)
- `C:\Windows\Installer\*.tmp` (temporales del instalador MSI)
- `C:\Windows\WinSxS\Backup\*` (backup de componentes, con precaución)
- `C:\Windows\Memory.DMP` (dump de memoria, si existe)
- `C:\Windows\Minidump\*` (minidumps de BSOD)

#### Caché de Windows Update
- `C:\Windows\SoftwareDistribution\Download\*`
- `C:\Windows\SoftwareDistribution\DataStore\*` (parcial)
- `C:\Windows\WinSxS\Temp\*`
- `C:\Windows\WinSxS\ManifestCache\*`

#### Caché de componentes (CBS/DISM)
- `C:\Windows\Logs\CBS\*` (logs de Component Based Servicing)
- `C:\Windows\Logs\DISM\*` (logs de DISM)
- `C:\Windows\WinSxS\Pending.xml` (si existe, indica operación pendiente)

#### Caché de fuentes
- `C:\Windows\ServiceProfiles\LocalService\AppData\Local\FontCache\*`
- `C:\Windows\System32\config\systemprofile\AppData\Local\FontCache\*`
- `C:\Windows\System32\FNTCACHE.DAT` (cache de fuentes del sistema)

#### Caché de thumbnails
- `%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db`
- `%LOCALAPPDATA%\Microsoft\Windows\Explorer\iconcache_*.db`

#### Caché de Internet Explorer / INetCache
- `%LOCALAPPDATA%\Microsoft\Windows\INetCache\*`
- `%LOCALAPPDATA%\Microsoft\Windows\WebCache\*`

#### Caché de Microsoft Edge
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Cache\*`
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Code Cache\*`
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\GPUCache\*`
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Service Worker\CacheStorage\*`
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\IndexedDB\*` (opcional, puede contener datos de sitios)
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Media Cache\*`
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Session Storage\*`

#### Caché de Google Chrome (si está instalado)
- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\Cache\*`
- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\Code Cache\*`
- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\GPUCache\*`
- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\CacheStorage\*`
- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\Media Cache\*`

#### Caché de aplicaciones Microsoft Store / UWP
- `%LOCALAPPDATA%\Packages\*\AC\Temp\*` (temporal de cada app UWP)
- `%LOCALAPPDATA%\Packages\*\AC\INetCache\*` (cache de internet de cada app UWP)
- `%LOCALAPPDATA%\Packages\*\LocalState\Temp\*`

#### Caché de .NET
- `%LOCALAPPDATA%\assembly\dl3\*` (download cache de .NET Framework)
- `C:\Windows\Microsoft.NET\assembly\NativeImages_*\*` (NGEN cache, con precaución)
- `%LOCALAPPDATA%\Microsoft\CLR_v4.0\UsageLogs\*`
- `%LOCALAPPDATA%\Microsoft\CLR_v4.0\NativeImages\*`

#### Caché de Visual Studio / Build tools
- `%LOCALAPPDATA%\Microsoft\VisualStudio\*\ComponentModelCache\*`
- `%TEMP%\VSFeedbackIntelliCodeLogs\*`
- `%TEMP%\Microsoft\VisualStudio\*\*.log`

#### Caché de npm / node
- `%LOCALAPPDATA%\npm-cache\*`
- `%LOCALAPPDATA%\node-gyp\*`

#### Caché de Python
- `%LOCALAPPDATA%\pip\Cache\*`
- `__pycache__` en directorios de usuario

#### Caché de Git
- `%LOCALAPPDATA%\GitHubDesktop\*\Cache\*`
- `.git` objects en repos de usuario (NO borrar, solo listar)

#### Caché de OneDrive
- `%LOCALAPPDATA%\Microsoft\OneDrive\logs\*`
- `%LOCALAPPDATA%\Microsoft\OneDrive\cache\*`

#### Caché de Teams
- `%APPDATA%\Microsoft\teams\Cache\*`
- `%APPDATA%\Microsoft\teams\Code Cache\*`
- `%APPDATA%\Microsoft\teams\GPUCache\*`
- `%APPDATA%\Microsoft\teams\IndexedDB\*`
- `%APPDATA%\Microsoft\teams\Service Worker\CacheStorage\*`

#### Caché de Outlook
- `%LOCALAPPDATA%\Microsoft\Outrook\*.ost` (NO borrar, son datos)
- `%LOCALAPPDATA%\Microsoft\Outlook\RoamCache\*`

#### Caché de Windows Error Reporting (WER)
- `%PROGRAMDATA%\Microsoft\Windows\WER\Temp\*`
- `%PROGRAMDATA%\Microsoft\Windows\WER\ReportArchive\*`
- `%LOCALAPPDATA%\Microsoft\Windows\WER\ReportQueue\*`

#### Caché de DNS
- `ipconfig /flushdns` (no es carpeta, pero se puede incluir como operación)

#### Caché de Event Logs (archivos .evtx antiguos)
- `C:\Windows\System32\winevt\Logs\Archive\*`

#### Caché de impresora (spooler)
- `C:\Windows\System32\spool\PRINTERS\*` (trabajos de impresión pendientes)

#### Caché de Windows Search Index
- `C:\ProgramData\Microsoft\Search\Data\Applications\Windows\*.edb` (reconstruir, no borrar directamente)

#### Caché de Credential Manager (temporales)
- `%LOCALAPPDATA%\Microsoft\Credentials\*` (NO borrar, son credenciales)

#### Caché de Remote Desktop
- `%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\*`

#### Caché de Xbox / Gaming Services
- `%LOCALAPPDATA%\Microsoft\GamingServices\*\Cache\*`
- `C:\XboxGames\*.tmp` (si existe)

#### Caché de Windows Terminal
- `%LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_*\LocalState\cache\*`

#### Caché de Widgets
- `%LOCALAPPDATA%\Packages\MicrosoftWindows.Client.CBS_*\TempState\*`

### Paso 5.5 — Categorizar entradas del catálogo con niveles de agresividad

Cada entrada en `cache-locations.json` debe tener un campo `risk_level`:

- `"safe"`: borrable sin riesgo (temporales, logs antiguos, cache de navegador).
- `"moderate"`: puede ralentizar algo temporalmente (prefetch, font cache, update cache).
- `"aggressive"`: puede romper funcionalidad hasta que se regenere (WinSxS backup, NGEN cache, search index).

La UI debe mostrar un warning visual para entradas `moderate` y `aggressive`.

### Verificación Fix 5

1. Seleccionar TODAS las checkboxes de caché.
2. Abrir la consola de debug (toggle).
3. Ejecutar limpieza.
4. **Aceptación**:
   - La consola muestra en tiempo real cada carpeta que se está procesando.
   - Al finalizar, el espacio liberado coincide con el estimado (±10%).
   - Revisar manualmente las carpetas de caché principales: deben estar vacías o con solo archivos en uso marcados para reboot.
   - Los archivos en uso se reportan correctamente con razón.
   - La app no se congela durante el borrado.
5. Verificar que el catálogo incluye al menos 50+ ubicaciones de caché diferentes.

---

## Archivos críticos a modificar (resumen)

| Archivo | Acción |
|---|---|
| `src-tauri/src/platform/powershell.rs` | **Crear** — helper centralizado |
| `src-tauri/src/platform/mod.rs` | Añadir `pub mod powershell;` |
| `src-tauri/src/platform/gpu.rs` | Migrar al helper |
| `src-tauri/src/platform/elevation.rs` | Renombrar → `try_relaunch_as_admin: bool` |
| `src-tauri/src/lib.rs` | Lógica "intento elevación + modo limitado" + validación catálogos |
| `src-tauri/build.rs` | Fix typo `True/PM` + `<dpiAwareness>PerMonitorV2` |
| `src-tauri/tauri.conf.json` | `bundle.windows.nsis.installMode: perMachine` |
| `src-tauri/src/domain/catalog.rs` | Reescrito con `include_str!` |
| `src-tauri/src/domain/cache_cleaner.rs` | **Reescribir** borrado recursivo robusto + eventos debug |
| `src/hooks/use-telemetry.ts` | `refetchInterval: 3000` |
| `src/features/explorer/explorer-page.tsx` | Reescrito a árbol expandible |
| `src/features/explorer/tree-view.tsx` | **Crear** |
| `src/features/explorer/tree-row.tsx` | **Crear** |
| `src/features/explorer/use-explorer-tree.ts` | **Crear** |
| `src/components/layout/app-shell.tsx` | Banner "Modo limitado" si `!isElevated` |
| `src/components/debug/debug-console.tsx` | **Crear** — consola de debug |
| `src/hooks/use-cache-debug.ts` | **Crear** — hook para eventos debug |
| `.claude/skills/cache-scanner/RESOURCES/cache-locations.json` | **Ampliar** con 50+ ubicaciones de caché |

---

## Orden de implementación sugerido

1. **Fix 1 (PowerShell)** — bloquea el uso de la app, urgente. Crear helper, migrar `gpu.rs`, bajar polling.
2. **Fix 3 (Catálogos)** — desbloquea la página Cache. Cambio aislado y barato.
3. **Fix 5 (Cache no se borra + debug + más carpetas)** — crítico para la funcionalidad principal. Requiere diagnóstico + reescritura del borrador + consola + ampliación del catálogo.
4. **Fix 2 (UAC)** — requiere rebuild + reinstalación para validar. Hacer al final del backend.
5. **Fix 4 (Explorer)** — el más grande, todo frontend, puede hacerse en paralelo.

---

## Verification end-to-end

Una vez aplicados todos los fixes:

1. **PowerShell**: `npm run tauri dev`. Dashboard abierto 60 s → 0 consolas. GPU funciona.
2. **Cache**: pestaña Cache muestra lista poblada. Error explícito si JSON inválido.
3. **Limpieza de caché**: seleccionar todo → ejecutar → consola debug muestra progreso → espacio liberado = estimado → carpetas vacías (salvo archivos en uso).
4. **UAC + Modo limitado**: build → instalar en VM → UAC salta → cancelar → banner + botones deshabilitados.
5. **Explorer**: escanear `C:\` → árbol expandible → tamaños lazy → búsqueda local → cancelar funciona.

---

## Pendientes que NO entran en este plan

- WMI nativo para telemetría de GPU sin PowerShell (cambio de raíz, posterior).
- Sistema de overrides de catálogo desde `%APPDATA%` (no necesario aún).
- `react-arborist` u otro tree library (custom basta para este alcance).
- Firma de código del instalador (postergado según CLAUDE.md).
- Cancelación cooperativa del `compute_directory_size` (mejora deseable pero no crítica).
- Reversibilidad granular de limpieza de caché (el restore point ya cubre la reversa total).
