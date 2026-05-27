# Sonnet 4.6 — Debloat universal + ClearTool como app ligera

> **Dos cambios estructurales** que afectan a varios módulos y a la filosofía
> del producto. Requieren diseño cuidadoso y decisiones que no se pueden
> deshacer fácilmente. **No delegar.**

---

## Parte A · Debloat universal (detección + uninstall completo + residuales)

### Visión

Hoy `bloatware-catalog.json` tiene 11 entradas curadas. El usuario quiere que ClearTool
**detecte todas las apps instaladas** (Win32 + UWP + tiendas de juegos) y permita
**desinstalar cualquiera con limpieza total de residuales** (registry, AppData,
ProgramData, accesos directos, tareas programadas, servicios, reglas firewall,
drivers).

El catálogo curado **no desaparece** — sigue marcando qué es bloatware "conocido" y
qué disclaimers mostrar. Pero la página Debloat se reformula como **un inventario
universal con filtros**, con el catálogo curado como una vista filtrada más.

### Nueva arquitectura

```
src-tauri/src/
├── platform/
│   ├── inventory/                    NUEVO
│   │   ├── mod.rs
│   │   ├── appx.rs                   ← Get-AppxPackage (ya existe parcialmente)
│   │   ├── uninstall_registry.rs     ← HKLM + HKCU \...\Uninstall walk
│   │   ├── steam.rs                  ← steamapps/libraryfolders.vdf parser
│   │   ├── epic.rs                   ← %PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests\*.item
│   │   ├── gog.rs                    ← HKLM\SOFTWARE\WOW6432Node\GOG.com\Games
│   │   ├── xbox.rs                   ← UWP filtrado por categoría Microsoft.GamingApp
│   │   ├── msstore.rs                ← AppxProvisioned con publisher Microsoft.Store
│   │   └── winget.rs                 ← `winget list --source winget` si está disponible
│   └── uninstaller.rs                ← refactor del actual
└── domain/
    └── inventory.rs                  NUEVO orquestador
```

### Modelo unificado

```rust
pub struct InstalledApp {
    pub id: String,                     // único, estable entre escaneos
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub source: AppSource,              // ver enum
    pub install_location: Option<PathBuf>,
    pub install_date: Option<String>,
    pub size_bytes: Option<u64>,        // si la fuente la reporta
    pub uninstall_method: UninstallMethod,
    pub is_system_critical: bool,       // detección heurística + allowlist
    pub catalog_match: Option<CatalogMatch>,  // si está en bloatware-catalog
    pub residual_hints: ResidualHints,  // ver más abajo
}

pub enum AppSource {
    AppxPackage { full_name: String, family_name: String, kind: AppxKind },
    AppxProvisioned { full_name: String },
    Win32Uninstaller { registry_key: String, hive: String },
    Steam { app_id: u64, library_path: PathBuf },
    EpicGames { catalog_item_id: String, manifest_path: PathBuf },
    Gog { game_id: u64 },
    Xbox { package_family_name: String, msstore_id: Option<String> },
    Winget { id: String },
}

pub enum AppxKind { User, Provisioned, Framework, Bundle }

pub enum UninstallMethod {
    AppxRemove,                                  // Remove-AppxPackage
    AppxProvisionedRemove,                       // Remove-AppxProvisionedPackage
    UninstallString { exe: PathBuf, args: Vec<String>, requires_admin: bool },
    QuietUninstallString { exe: PathBuf, args: Vec<String>, requires_admin: bool },
    MsiUninstall { product_code: String },       // msiexec /x {GUID} /qn
    SteamUninstall { app_id: u64 },              // steam://uninstall/<id>
    EpicUninstall { catalog_item_id: String },   // com.epicgames.launcher://uninstall?...
    GogUninstall { exe: PathBuf },
    NoUninstaller,                               // requiere borrado manual de carpeta + registry
}

pub struct ResidualHints {
    pub appdata_roaming: Vec<PathBuf>,
    pub appdata_local: Vec<PathBuf>,
    pub programdata: Vec<PathBuf>,
    pub registry_keys: Vec<(String, String)>,    // (hive, path)
    pub start_menu_shortcuts: Vec<PathBuf>,
    pub desktop_shortcuts: Vec<PathBuf>,
    pub scheduled_tasks: Vec<String>,
    pub services: Vec<String>,
    pub firewall_rules: Vec<String>,
    pub drivers: Vec<String>,
}
```

### Detección por fuente — notas críticas

**1. Win32 Uninstall registry walk.**
Hives a recorrer:
- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*`
- `HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*`
- `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\*`

Para cada subkey leer: `DisplayName`, `DisplayVersion`, `Publisher`, `InstallLocation`,
`InstallDate`, `EstimatedSize` (KB → bytes), `UninstallString`, `QuietUninstallString`,
`SystemComponent` (descartar si == 1), `WindowsInstaller` (si == 1, parsear el GUID del
UninstallString para `MsiUninstall`).

**Heurística `is_system_critical`:** `Publisher == "Microsoft Corporation"` AND
(`SystemComponent == 1` OR `DisplayName` matchea `^(Microsoft Visual C\+\+|Windows SDK|\.NET|Microsoft Edge WebView2)`). Estas se muestran pero con badge "no recomendado desinstalar".

**2. Appx.** Ya hay scaffolding. Extender para:
- Distinguir `User` vs `Provisioned` vs `Framework`. Frameworks (CRT, VCLibs) **no se ofrecen para desinstalar** salvo con flag avanzado.
- Detectar Xbox/Game Pass games: PFN empieza con publisher Microsoft pero el manifest declara `<uap3:DependencyTarget>` o categoría `gaming`.
- Para cada Appx, intentar resolver el ms-store-id (consulta a `winget search --id <PFN>`) — guardar el deep link `ms-windows-store://pdp/?productid=...` para la reinstalación reverse.

**3. Steam.** Parser de `libraryfolders.vdf` (VDF v2). Por cada library, enumerar
`steamapps/appmanifest_<appid>.acf` y leer `name`, `installdir`, `SizeOnDisk`,
`LastUpdated`. **No usar regex** — escribir un mini-parser VDF (es trivial:
`"key" "value"` o `"key" { ... }`).

**4. Epic Games.** Cada `.item` en `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests\`
es JSON. Leer `DisplayName`, `InstallLocation`, `CatalogItemId`.

**5. GOG y Xbox.** Implementación inicial mínima — listar solo, marcar `UninstallMethod::NoUninstaller` para los GOG sin uninstaller registrado y depender de la limpieza de residuales para borrarlos.

### Cálculo de `ResidualHints` (heurístico, sin nukear)

Tras el uninstall, **muchos installers dejan basura**. ClearTool debe ofrecer un
"second pass" opcional que limpia residuales con confirmación. Las heurísticas:

**AppData:**
- Si `install_location` o `display_name` contiene "Vendor X App Y", buscar
  `%APPDATA%\X`, `%APPDATA%\X\Y`, `%LOCALAPPDATA%\X`, `%LOCALAPPDATA%\X\Y`.
- Solo proponer si el path existe AND su tamaño > 0 AND no es system path
  (allowlist negativa: no proponer `%APPDATA%\Microsoft`, `%LOCALAPPDATA%\Microsoft`,
  `%LOCALAPPDATA%\Packages`, etc.).

**Registry:**
- `HKCU\Software\<Publisher>\<DisplayName>` y `HKCU\Software\<DisplayName>`
- `HKLM\SOFTWARE\<Publisher>\<DisplayName>` y `HKLM\SOFTWARE\WOW6432Node\<Publisher>\<DisplayName>`
- Solo proponer si la key existe, y **no es shared con otro publisher** (verificar
  si la key tiene subkeys/valores que mencionen otras apps).

**Shortcuts:**
- `%APPDATA%\Microsoft\Windows\Start Menu\Programs\<DisplayName>*`
- `%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\<DisplayName>*`
- `%USERPROFILE%\Desktop\<DisplayName>*.lnk`

**Scheduled tasks:** `Get-ScheduledTask` filtrar por `Description` o `Author` que
contenga publisher o display name.

**Servicios:** filtrar por `DisplayName` o `BinaryPathName` que apunte a
`install_location`. Si el servicio sigue corriendo, advertir antes.

**Firewall rules:** `Get-NetFirewallRule` + `Get-NetFirewallApplicationFilter`
con `Program` apuntando al `install_location`.

**Drivers:** solo proponer si el driver es **unsigned** o firmado por el publisher
(no por Microsoft). `pnputil /enum-drivers` + filtrar.

### Comandos IPC nuevos

```rust
list_installed_apps() -> Vec<InstalledApp>        // un único listado unificado
compute_residual_hints(app_id) -> ResidualHints   // lazy, on-demand al expandir
uninstall_app(app_id, dry_run) -> UninstallReport
clean_residuals(app_id, selected_residuals, dry_run) -> CleanReport
uninstall_app_complete(app_id, dry_run) -> UninstallCompleteReport  // 1+2 en un solo flow
```

**`uninstall_app_complete`** orquesta:
1. Restore point con descripción `"ClearTool — uninstall <name>"`.
2. `uninstall_app` (espera a que termine, hasta 5min timeout para installers lentos).
3. `compute_residual_hints` (post-uninstall, porque algunos installers borran cosas).
4. `clean_residuals` con TODOS los hints (el usuario habrá confirmado en UI antes).
5. AuditEntry con `ReverseRecipe::AppxReinstall` (si era Appx con ms-store-id) o
   `ReverseRecipe::Noop { reason: "Win32 uninstall — reinstall manual desde <publisher_url>" }`.

### UI

Reescribir `src/features/debloat/debloat-page.tsx` (no spec ahora — diseño libre,
pero respetando el patrón de `cache-page.tsx`: vista `Selección` → `Plan` → `Ejecución`).

**Filtros obligatorios:**
- Texto libre (nombre, publisher).
- Por fuente (Appx, Win32, Steam, Epic, ...).
- Por publisher (lista desplegable autopoblada).
- "Solo bloatware del catálogo" (filtra por `catalog_match.is_some()`).
- "Ocultar componentes del sistema" (filtra `is_system_critical`).
- Por tamaño (slider min-max).
- Ordenar por: nombre, tamaño, fecha instalación.

**Disclaimer obligatorio** antes de desinstalar:
- Si `catalog_match.requires_disclaimer` → modal con riesgos.
- Si `is_system_critical` → confirmación doble.

### Tests

- VM Win11 OEM (Lenovo o HP): `list_installed_apps()` debe devolver al menos 80
  entries (mezcla Appx + Win32 + crapware OEM).
- Instalar manualmente Notepad++ y Spotify. Ejecutar `uninstall_app_complete` para
  Notepad++. Verificar: ya no aparece en `Get-Package`, `%APPDATA%\Notepad++` ya no
  existe, ningún shortcut, ninguna key registry residual.

### No-objetivos (explícitos)

- **No** intentar desinstalar Windows Defender ni componentes que rompan el
  arranque. Allowlist negativa en `is_system_critical`.
- **No** soportar fuentes adicionales (Origin, Battle.net, Ubisoft Connect, ...) en
  esta primera iteración. Documentar como follow-up.

---

## Parte B · ClearTool como app de optimización ligera

### Principio rector

> Una app que se vende como "optimización" no puede ser pesada. Idle: < 80 MB de
> RAM, < 1% de CPU. Bundle: < 1 MB. Startup-to-first-paint: < 800ms en VM con HDD.

Hoy estamos lejos de eso:
- Bundle 1.8 MB ([HISTORIAL-SESIONES.md](../HISTORIAL-SESIONES.md) sesión 4: vite warning).
- `tokio = { features = ["full"] }` ([Cargo.toml:58](../../src-tauri/Cargo.toml#L58)) trae multi-thread runtime + signal + io + process + macros + fs + sync + time + rt-multi-thread + ... pesa ~40k LOC.
- Telemetry polling cada 1.5s desde el frontend ([home-page.tsx:72](../../src/features/home/home-page.tsx#L72)).
- `sysinfo::System::new().refresh_processes(All, true)` cada 2s desde Process page ([processes-page.tsx:17](../../src/features/processes/processes-page.tsx#L17)) — caro.
- Three.js cargado en home aunque haya `lazy()` (verificar; ver tarea 4 del archivo 01).
- `framer-motion` (~70 KB) usado en muy pocos sitios.

### Plan en 4 frentes

**1. Backend Rust — recortar deps y polling.**

```toml
# Cargo.toml — reemplazar tokio = "full" por:
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync", "fs", "process"] }
```

Eliminar features no usadas (`signal`, `io-util`, `net`). Verificar con `cargo bloat --release --crates`.

**Sysinfo refresh selectivo.** En lugar de `refresh_processes(All, true)` (incluye `update_cpu` que recorre todos los procesos), usar:
```rust
sys.refresh_processes_specifics(ProcessesToUpdate::All, ProcessRefreshKind::nothing().with_cpu().with_memory());
```

Cachear el `System` en un `OnceLock<Mutex<System>>` global en lugar de crear uno por llamada (cada `new()` cuesta ~30ms).

**Reducir frecuencia de telemetry.** El front polleaba cada 1.5s; subir a 3s salvo que el usuario abra explícitamente el panel telemetry expandido (en cuyo caso queda en 1.5s).

**2. Frontend — eliminar deps innecesarias.**

- `framer-motion` (~70 KB) → reemplazar por CSS transitions + `react-transition-group` (3 KB) o nada. Las pocas animaciones (`itemVariants`, `containerVariants` en home) se hacen con `@keyframes` + `animation-delay`.
- `recharts` (~120 KB) → si solo se usa para `CpuLineChart` y `RamRing`, reemplazar por un canvas custom de 100 líneas. Mantener si se usa en > 4 sitios.
- `three`, `@react-three/fiber`, `@react-three/drei` (~600 KB combinados) → **decisión arquitectónica**: mantener el hero 3D solo si es el "wow" del producto. Si no, reemplazar por un canvas 2D con shader simple (~5 KB). Documentar la decisión en `CLAUDE.md`.
- `cmdk` (~6 KB) → mantener, es eficiente.

**3. Process detection caching.**

Hoy `who_locks_path` llama a `list_processes_extended()` para cada path. En `analyze_locations` con 50 ubicaciones es 50× el enumerate. Refactor:

```rust
// domain/cache.rs::analyze_locations
let process_snapshot = platform::processes::snapshot()?;  // un solo enumerate
for id in ids {
    let lockers = platform::processes::who_locks_path_cached(&process_snapshot, path)?;
    // ...
}
```

(Esto se vuelve redundante con la implementación correcta de Restart Manager — RM hace su propio snapshot. Pero mientras tanto, evita el quadratic.)

**4. Lazy boot.**

- `lib.rs::run::setup` valida catálogos al arranque (`expect("catálogos embebidos inválidos")`). Mover a primera demanda — si el JSON está roto, falla al abrir Debloat, no al abrir la app. Justificación: arranque rápido > validación temprana cuando los JSON están embebidos y un build malo se detecta en CI.
- `decorum::init` solo cargar si el usuario tiene la titlebar custom activa (futuro setting). Por ahora mantener pero medir su coste.
- `tauri_plugin_*` — auditar cuáles realmente se invocan desde el frontend. `tauri_plugin_opener` y `tauri_plugin_dialog` sí; `tauri_plugin_os`, `tauri_plugin_process`, `tauri_plugin_shell`, `tauri_plugin_fs` — verificar uso real. Cada plugin son ~50-200 KB en el binario final.

### Métricas objetivo (mantener en CI)

| Métrica | Hoy aprox | Objetivo |
|---|---|---|
| Bundle JS principal | 1.8 MB | < 600 KB |
| Bundle JS total (todos los chunks) | 1.9 MB | < 1.4 MB |
| Idle RAM (proceso `cleartool.exe`) | desconocido | < 80 MB |
| Idle CPU | desconocido | < 1% |
| Cold start to first paint | desconocido | < 800ms |
| Binario Windows release | desconocido | < 12 MB |

Añadir un script `scripts/measure-perf.ps1` que arranca la app, espera 5s idle, lee
RAM/CPU vía `Get-Process cleartool`, mide tiempo a primer paint con `Out-Default`. Correr en CI tras `cargo build --release`.

### Acceptance

- `cargo bloat --release --crates` muestra top-10 crates < 8 MB combinados.
- `npm run build` reporta main chunk < 600 KB.
- Smoke manual en VM idle: ClearTool abierto en home sin interacción durante 5 minutos, Task Manager → RAM < 80 MB, CPU 0% (con picos < 2% durante refresh de telemetry).
