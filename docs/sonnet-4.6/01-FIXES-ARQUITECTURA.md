# Sonnet 4.6 — Tareas de arquitectura (M1 → M4)

> **Para Sonnet 4.6.** Cada tarea aquí requiere decisiones de diseño no triviales,
> APIs de Win32 que pueden colgar la app si se manejan mal, o refactors que tocan
> varios módulos a la vez. **No delegar a Qwen.**
>
> Estilo: comentarios en español, identificadores en inglés. `cargo fmt` +
> `cargo clippy -D warnings` antes de commit. Tests cuando toques lógica destructiva.

---

## 1. `who_locks_path` con Restart Manager API

**Archivo:** [src-tauri/src/platform/processes.rs:431-449](../../src-tauri/src/platform/processes.rs#L431-L449)

**Problema actual.** La implementación compara `exe_path.to_lowercase() == path.to_lowercase()`. Solo dispara si la ruta consultada es **literalmente** el `.exe` de un proceso. Cuando `domain::cache::analyze_locations` pregunta "¿quién tiene abiertos archivos en `%LOCALAPPDATA%\Spotify\Storage`?", obtiene **siempre `[]`** y la sección "Bloqueadas" del plan queda vacía aunque Spotify esté corriendo y manteniendo handles. Esto rompe el contrato del M2 (`v2/02-cache-engine-rewrite.md`).

**Solución.** Usar Windows **Restart Manager** (`RstrtMgr.dll`):

```rust
RmStartSession      → handle de sesión
RmRegisterResources → pasar el path (o lista de paths) que queremos liberar
RmGetList           → devuelve RM_PROCESS_INFO[] con PID, ProcessName, AppType
RmEndSession        → cleanup obligatorio (Drop guard)
```

**Detalles que importan:**

- `RmRegisterResources` acepta hasta `RM_FILE_ADVISORY` = ~262144 ficheros por llamada. Si el path es un **directorio**, hay que walk-tree y registrar archivos individualmente, o pasar el directorio como recurso de tipo `RmFile` y dejar que Windows lo expanda (depende de la versión — verifica en Win11 24H2).
- `RmGetList` se invoca con un buffer; si `ERROR_MORE_DATA`, reasignar y reintentar (loop con tope de 3).
- El handle de sesión es por-thread; envolver en un `RestartManagerSession` con `Drop` que llame a `RmEndSession`. Si el `Drop` no se ejecuta, la sesión queda colgando.
- Filtrar PIDs protegidos (`is_system_protected_pid_lookup`) antes de devolver — no proponer matar `lsass.exe` aunque tenga el handle.
- Mapear `RM_APP_TYPE` (Explorer, MainWindow, OtherWindow, Service, Console) a un campo nuevo en `LockingProcess` para que la UI decida la `BlockedAction` (`CloseProcess` vs `ScheduleReboot` vs `SkipOnly`).

**Crate windows-rs.** Activar feature `Win32_System_RestartManager` en `Cargo.toml`. Los símbolos viven en `windows::Win32::System::RestartManager::*`.

**Fallback.** Si `RmStartSession` devuelve error (Restart Manager está deshabilitado en algunas SKUs), **caer al método actual** como degradado y emitir un `log::warn!`. No fallar la operación entera.

**Tests.**
- Unit: mock `RmGetList` para verificar el parsing de `RM_PROCESS_INFO` (estructura packed).
- Integración: abrir un archivo con `tokio::fs::File::create` y verificar que `who_locks_path` devuelve el PID actual.

**Acceptance.** Tras este cambio, abrir Spotify, llenar su caché, ejecutar `analyze_cache_locations(['spotify-cache'])` desde la UI: el plan debe poner Spotify en `blocked`, no en `ready`.

---

## 2. `apply_privacy_preset` (backend) + Privacy Hardening real

**Archivos a crear:**
- `src-tauri/src/domain/privacy.rs` (nuevo)
- `src-tauri/src/ipc/privacy.rs` (nuevo)
- `src-tauri/src/models/privacy.rs` (nuevo)

**Archivos a modificar:**
- `src-tauri/src/lib.rs` — registrar el comando
- [src/features/privacy/privacy-page.tsx:68-69](../../src/features/privacy/privacy-page.tsx#L68-L69) — quitar `setTimeout` mock, conectar a la mutación real
- `src/api/client.ts` — exponer `applyPrivacyPreset`, `getPrivacyPresetPreview`

**Diseño.**

```rust
pub enum PrivacyLevel { Balanced, Strict, Paranoid }

pub struct PrivacyPresetDefinition {
    pub level: PrivacyLevel,
    pub registry_tweak_ids: Vec<String>,    // ids del catálogo registry-tweaks
    pub service_changes: Vec<(String, String)>,  // (service_name, start_type)
    pub debloat_entry_ids: Vec<String>,     // ids del bloatware-catalog
}

pub struct PrivacyPresetPreview {
    pub level: PrivacyLevel,
    pub registry_ops: Vec<PreviewRegistryOp>,
    pub service_ops: Vec<PreviewServiceOp>,
    pub appx_ops: Vec<PreviewAppxOp>,
    pub estimated_changes: u32,
}

pub fn preview(level: PrivacyLevel) -> AppResult<PrivacyPresetPreview>;
pub fn apply(level: PrivacyLevel, dry_run: bool) -> AppResult<PrivacyApplyReport>;
```

**Reglas no negociables.**
- `apply` SIEMPRE crea un restore point (no opcional) — vía `domain::audit::with_audit`.
- Cada sub-operación (tweak, servicio, appx) usa su propia función del dominio (`domain::registry::apply_tweak`, etc.). **No reimplementar.** Las cuatro pasadas son atómicas en cuanto a "todo o nada de cada bloque": si falla un tweak, los anteriores quedan, pero el restore point permite revertir global.
- Escribir **una única** `AuditEntry` con `module: "privacy"`, `operation: "apply_preset"`, y un `ReverseRecipe::Composite` (ver tarea 3) que liste cada sub-reverse para revert fino.
- El listado de tweaks/servicios/appx por nivel vive en un JSON nuevo: `.claude/skills/.../privacy-presets.json` con su schema. **No hardcodear en Rust.**

**Frontend.** Sustituir el contenido hardcoded de `LEVELS` en [privacy-page.tsx:14-60](../../src/features/privacy/privacy-page.tsx#L14-L60) por un `useQuery` que llame a `getPrivacyPresetPreview(level)`. Mostrar los cambios reales que devuelve el preview, no la lista cosmética. Botón "Ver cambios detallados" abre un modal con la lista expandida.

**Acceptance.**
- VM Win11 OEM: aplicar "Paranoid", reboot, verificar que las claves del registro indicadas están aplicadas, los servicios listados están `Disabled`, los Appx eliminados no aparecen.
- Audit log tiene una entry con `ReverseRecipe::Composite` que al ejecutar `revert_audit_entry` deshace los tres bloques.

---

## 3. `ReverseRecipe::Composite` + Startup reverse real

**Archivos:**
- [src-tauri/src/models/restore.rs](../../src-tauri/src/models/restore.rs) — añadir variante al enum
- [src-tauri/src/domain/audit.rs:105-146](../../src-tauri/src/domain/audit.rs#L105-L146) — manejar la nueva variante en `revert_entry`
- [src-tauri/src/domain/startup.rs:63-67, 107-110](../../src-tauri/src/domain/startup.rs#L63-L67) — generar el recipe correcto

**Diseño del enum.**

```rust
#[serde(tag = "kind")]
pub enum ReverseRecipe {
    Registry { operations: Vec<RegistryReverseOp> },
    Service { service_name: String, previous_start_type: String, previous_state: String },
    AppxReinstall { package_family_name: String, store_url: Option<String> },
    StartupToggle {  // NUEVO
        origin: StartupOrigin,
        previous_enabled: bool,
    },
    Composite { recipes: Vec<ReverseRecipe> },  // NUEVO — para apply_privacy_preset
    Noop { reason: String },
}
```

**Por qué `Composite` y no `Vec<ReverseRecipe>` directo en `AuditEntry`:** mantener la API serializada estable (`reverse_recipe` sigue siendo un único campo discriminado). Si más adelante un preset compuesto tiene que componer otro compuesto, ya está soportado por recursión.

**`revert_entry` para `StartupToggle`:** llama a `enable_startup`/`disable_startup` según el `previous_enabled`. Cuidado con la recursión audit (no escribir entry de revert que a su vez se autorevierta).

**`revert_entry` para `Composite`:** iterar la lista en **orden inverso al de aplicación** (LIFO) y llamar recursivamente. Si una sub-reverse falla, **continuar con las demás** y agregar los errores en un `Vec<String>` que se loguea pero no aborta — el comportamiento "best effort" es preferible a dejar el sistema medio revertido.

**Tests.**
- Unit: serializar y deserializar cada variante; verificar que `serde(tag = "kind")` produce el discriminator esperado para el frontend.
- Unit: `Composite` con dos sub-reverses, una falla, otra ok → reporta error de la fallida pero aplica la ok.

---

## 4. Lazy loading real + chunk splitting

**Problema.** El bundle principal pesa **1.8 MB** ([build log](../HISTORIAL-SESIONES.md) — Vite warning "chunks larger than 500 kB"). `router.tsx` envuelve cada ruta en `<Suspense>` pero los `import` son síncronos. Estás pagando todo el coste de cmdk + recharts + three.js + framer-motion en el primer paint.

**Cambios:**

[src/router.tsx:7-18](../../src/router.tsx#L7-L18) — sustituir imports por `lazy()`:

```ts
import { lazy } from "react";
const HomePage = lazy(() => import("./features/home/home-page").then(m => ({ default: m.HomePage })));
const CachePage = lazy(() => import("./features/cache-cleaner/cache-page").then(m => ({ default: m.CachePage })));
// ... resto igual
```

**Manual chunks en vite.config.ts** — separar dependencias pesadas para que se cacheen entre rebuilds:

```ts
build: {
  rollupOptions: {
    output: {
      manualChunks: {
        'react-vendor': ['react', 'react-dom', 'react-router-dom'],
        'tanstack': ['@tanstack/react-query', '@tanstack/react-table', '@tanstack/react-virtual'],
        'three': ['three', '@react-three/fiber', '@react-three/drei'],
        'charts': ['recharts'],
        'radix': [
          // glob de @radix-ui/* — Vite los agrupa
        ],
        'motion': ['framer-motion'],
      },
    },
  },
  chunkSizeWarningLimit: 600,
}
```

**Decisión a tomar** (registrar en `CLAUDE.md` "Decisiones arquitectónicas vivas"):
- ¿Mantener `three` + `@react-three/fiber` + `@react-three/drei` en el Home? Pesan ~600 KB combinados y solo se usan en el hero. **Sonnet decide**: o (a) eliminar el hero 3D y reemplazar por un visual 2D liviano (Canvas API), o (b) mantenerlo pero verificar que el `lazy()` actual en `home-page.tsx:18` realmente funciona y no carga cuando el usuario navega directo a `/cache`. Es probable (b) sea suficiente con el chunk split.

**Acceptance.**
- `npm run build` reporta el chunk principal `< 600 KB`.
- Network tab al cargar `/cache` directo: no descarga `three.js` ni `recharts`.

---

## 5. Disk Analyzer — treemap real con d3-hierarchy

**Archivo:** [src/features/disk/disk-page.tsx:97-103](../../src/features/disk/disk-page.tsx#L97-L103)

Hoy la página dice literal "Treemap canvas — pendiente de implementación".

**Stack.**
- `npm install d3-hierarchy d3-scale` (~30 KB total)
- Render en `<canvas>` 2D, **no SVG** — con 5000+ nodos SVG es inaceptable. Spec M4 dice `< 5k+ nodos sin lag`.

**Diseño del componente.**

```tsx
function Treemap({ root, width, height, onNodeClick }: Props) {
  const layout = useMemo(() => {
    const h = d3.hierarchy(root, d => d.children)
      .sum(d => d.children?.length ? 0 : d.sizeBytes)  // solo hojas suman
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));
    return d3.treemap<TreemapNode>()
      .size([width, height])
      .paddingInner(1)
      .round(true)(h);
  }, [root, width, height]);

  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (!ctx) return;
    drawTreemap(ctx, layout, colorScale);
  }, [layout]);
  // ...
}
```

**Interacciones.**
- Click en rectángulo → zoom (re-renderiza con ese nodo como root, animación de 200ms con `requestAnimationFrame`).
- Doble click → menú contextual: "Abrir en Explorador", "Mostrar tamaño detallado", "Eliminar" (con confirmación + restore point si el path está en la denylist de cache).
- Hover → tooltip flotante con nombre, tamaño humanizado, tipo.
- Color: hash de la extensión a paleta categórica (categorías: media, code, docs, archives, executables, other).

**Performance.**
- Memoizar `layout` por `(root.path, width, height)`.
- Si > 2000 nodos visibles, **skip nodos con `value < 0.1% del root`** (no se ven igualmente y agregan latencia).
- `ResizeObserver` con debounce de 200ms para resize de ventana.

**Acceptance manual.**
- Escanear `C:\` con `maxDepth=4, minSizeMb=10` debe devolver entre 200-2000 nodos. Render < 300ms. Zoom y vuelta-atrás (botón) < 100ms.

---

## 6. Boot-time Cleanup — exponer pending_renames vía IPC + UI

**Backend.**

[src-tauri/src/platform/pending_rename.rs](../../src-tauri/src/platform/pending_rename.rs) ya tiene `list_pending_renames()` y `schedule_delete_on_reboot()`. Falta:

- `cancel_pending_rename(source: String) -> AppResult<()>` — leer `PendingFileRenameOperations`, filtrar la entrada cuyo source coincide, reescribir el `REG_MULTI_SZ` sin esa entrada. **Cuidado:** el formato es de pares (source, destination) — borrar el par entero, no solo el source. Si destination es vacío, es un delete (eso ya lo entiende `list_pending_renames`).
- `clear_all_pending_renames() -> AppResult<u32>` — devuelve count de entradas eliminadas. Solo afecta entries creadas por ClearTool (filtrar por path prefix conocido — emitir `log::warn!` si hay entries que no son nuestras y no tocarlas).

**IPC nuevo:** `src-tauri/src/ipc/boot_cleanup.rs`
```rust
#[tauri::command] pub async fn list_pending_renames() -> AppResult<Vec<PendingRename>>;
#[tauri::command] pub async fn cancel_pending_rename(source: String) -> AppResult<()>;
```

**Registrar** en [src-tauri/src/lib.rs:71-143](../../src-tauri/src/lib.rs#L71-L143).

**UI nueva.** No crear página dedicada — añadir un panel colapsable dentro de **Cache Cleaner** (`cache-page.tsx`) que liste las entries pendientes. Cabecera: "X archivos programados para borrarse en el próximo reboot · [Cancelar todos] · [Ver detalles]". Justificación: el usuario las creó desde ahí; tiene sentido revisarlas en el mismo sitio.

**Acceptance.**
- Ejecutar `analyze_cache_locations` con un archivo bloqueado, ejecutar `execute_clean_plan` con `scheduleBlockedForReboot: true`, ir a Cache page: el panel muestra el archivo programado.
- Click "Cancelar" → desaparece del listado y `list_pending_renames` no lo devuelve.
