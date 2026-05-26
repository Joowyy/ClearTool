# v2 · 02 — Cache Engine Rewrite (P0)

## El bug central

Logs reales del cache cleaner v0.1 (sesión 2026-05-25):

```
[WARN] UWP Apps — LocalState\Temp: Archivo en uso, se eliminará en el próximo reboot:
       C:\Users\joels\AppData\Local\Packages\Claude_pzs8sxrjxfjjc
[ERROR] UWP Apps — LocalState\Temp: Error: ...primary.ldb\000089.ldb:
        file in use, could not schedule for reboot deletion
[ERROR] UWP Apps — LocalState\Temp: Error: ...widevinecdm.dll:
        Acceso denegado. (os error 5)
```

Síntomas:
1. Archivos **en uso** por procesos activos (Spotify, Claude Desktop, Discord) no se eliminan.
2. Tras "limpiar", el escaneo siguiente reporta **el mismo tamaño**, como si no se hubiera hecho nada.
3. La gente percibe la app como "no funciona" porque su métrica de éxito es "el número de GB liberados bajó".

## Por qué la v1 falla

El engine actual hace, simplificado:

```
for cada path en catalog:
   walk_dir(path):
      for cada archivo:
         try fs::remove_file(archivo)
         si falla:
            try MoveFileEx(archivo, NULL, MOVEFILE_DELAY_UNTIL_REBOOT)
            si falla:
               loguear error
```

Tres problemas:
1. **Borra una vez, no reintenta**. Si un proceso libera el archivo medio segundo después, perdemos.
2. **No analiza qué proceso lo tiene abierto**. No puede ofrecer acciones útiles al usuario.
3. **MoveFileEx requiere admin Y falla en archivos memory-mapped** (típico en UWP, browsers Chromium-based).
4. **El catálogo no clasifica los paths por estrategia**. Trata `%TEMP%` igual que `%LOCALAPPDATA%\Packages\Spotify\...` cuando son problemas completamente distintos.

## Diseño v2

### Estrategias de limpieza tipificadas

Cada `cache-location.json` declara una `strategy`:

```json
{
  "id": "uwp-spotify-cache",
  "displayName": "Spotify — caché UWP",
  "path": "%LOCALAPPDATA%\\Packages\\SpotifyAB.SpotifyMusic_zpdnekdrzrea0\\LocalCache",
  "strategy": "uwp-app-aware",
  "lockedBy": ["Spotify.exe"],
  "category": "uwp-cache"
}
```

Strategies disponibles:

| Strategy | Qué hace | Cuándo usarla |
|----------|----------|---------------|
| `direct-delete` | `remove_dir_all` directo, retry x3 | Caches del sistema (DISM, CBS), prefetch |
| `uwp-app-aware` | Detecta si el proceso UWP está running. Si sí: ofrece "cerrar app". Si no: limpia directamente | UWP `LocalState`, `LocalCache`, `RoamingState` |
| `browser-aware` | Detecta browser process. Ofrece "cerrar navegador". Limpia. | Chrome, Edge, Firefox, Brave caches |
| `process-locked` | Detecta procesos que tienen handle abierto al path. Lista, ofrece acción | Genérico para archivos con lock |
| `system-restart-required` | Schedule via `PendingFileRenameOperations` registry | Archivos del kernel, drivers, .dll cargadas |
| `take-ownership-and-delete` | `takeown` + `icacls` + delete | TrustedInstaller-owned paths (raro pero existe) |

### Pre-flight check (ANTES de borrar)

Nuevo step en `clean()`:

```rust
// src-tauri/src/domain/cache.rs (v2)
pub fn pre_flight_check(locations: &[ResolvedLocation]) -> CleanPlan {
    let mut plan = CleanPlan::default();
    for loc in locations {
        let report = analyze_location(loc); // ← nuevo
        match report.status {
            LocationStatus::Clean => plan.ready.push(loc.clone()),
            LocationStatus::ProcessLocked(procs) => {
                plan.blocked.push(BlockedLocation {
                    loc: loc.clone(),
                    locked_by: procs,
                    suggested_action: suggest_action(loc, &procs),
                });
            }
            LocationStatus::PermissionDenied => plan.permission_issues.push(loc.clone()),
            LocationStatus::DoesNotExist => plan.skipped.push(loc.clone()),
        }
    }
    plan
}
```

El frontend recibe el `CleanPlan` y muestra al usuario:

```
✓ Lista para limpiar (12 ubicaciones, ~3.2 GB)
⚠ Bloqueada por procesos (4 ubicaciones, ~1.8 GB)
   - Spotify.exe bloquea: ...\Spotify\LocalCache  [Cerrar Spotify]
   - Claude.exe bloquea: ...\Claude\Cache         [Cerrar Claude]
✗ Permisos insuficientes (1 ubicación)
   - %SYSTEMROOT%\WinSxS  [Requiere TrustedInstaller — saltar]
- Saltadas: 3 ubicaciones no existen
```

### Detección de quién tiene el handle

Tres opciones, en orden de preferencia:

#### Opción A — `Restart Manager` (recomendada)
Windows tiene una API nativa para esto: `RmStartSession` + `RmRegisterResources` + `RmGetList`.

```rust
// src-tauri/src/platform/process_lock.rs (NUEVO)
use windows::Win32::System::RestartManager::*;

pub fn who_locks(path: &Path) -> AppResult<Vec<LockingProcess>> {
    unsafe {
        let mut session = 0u32;
        let mut session_key = [0u16; 64];
        RmStartSession(&mut session, 0, session_key.as_mut_ptr())?;

        let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let paths = [path_wide.as_ptr()];
        RmRegisterResources(session, 1, paths.as_ptr(), 0, ptr::null(), 0, ptr::null())?;

        let mut needed = 0u32;
        let mut affected = vec![RM_PROCESS_INFO::default(); 64];
        let mut affected_count = 64u32;
        RmGetList(session, &mut needed, &mut affected_count, affected.as_mut_ptr(), ptr::null_mut())?;

        let procs = affected[..affected_count as usize].iter().map(|p| LockingProcess {
            pid: p.Process.dwProcessId,
            name: String::from_utf16_lossy(&p.strAppName),
            kind: process_kind_from_app(p.ApplicationType),
        }).collect();

        RmEndSession(session);
        Ok(procs)
    }
}
```

Pros: API oficial, funciona en UWP, no necesita admin para query.
Contras: API verbosa.

#### Opción B — `handle.exe` de Sysinternals
Llamar a `handle.exe` como subprocess y parsear su output. **Requiere distribuir handle.exe** (licencia EULA Sysinternals — verificable).

Descartado por dependencia externa.

#### Opción C — NtQueryObject + enumeración de handles
Más rápido pero requiere mover punteros entre user/kernel space. Complejidad alta.

Descartado por mantenibilidad.

**Decisión: Opción A (Restart Manager).**

### Retry logic con backoff

```rust
async fn delete_with_retry(path: &Path) -> AppResult<()> {
    let mut delay = Duration::from_millis(100);
    for attempt in 0..3 {
        match fs::remove_file(path) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempt < 2 => {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(AppError::Io(format!("{}: {}", path.display(), e))),
        }
    }
    unreachable!()
}
```

### Per-location options en el catálogo

Cada `cache-location.json` puede declarar más control fino:

```json
{
  "id": "windows-temp",
  "path": "%TEMP%",
  "strategy": "direct-delete",
  "filters": {
    "olderThanDays": 1,
    "exclude": ["*.lock", "Outlook.dat"],
    "minSizeKb": 0,
    "maxDepthFromRoot": 10
  },
  "preconditions": [
    { "type": "process-not-running", "name": "Outlook.exe" }
  ],
  "rollbackable": false,
  "averageSize": "100MB-2GB",
  "lastUpdated": "2026-05-25"
}
```

### Progress reporting fino

El evento `cache:progress` v2 carga más info útil:

```ts
interface CacheProgress {
  locationId: string;
  phase: "scanning" | "analyzing-locks" | "deleting" | "scheduling-reboot";
  currentFile: string | null;
  filesProcessed: number;
  filesTotal: number;
  bytesFreedSoFar: number;
  bytesEstimatedTotal: number;
  blockedFiles: number;     // ← nuevo
  rebootScheduledFiles: number; // ← nuevo
}
```

La UI muestra una progress bar con dos colores: verde (eliminados) + naranja (programados para reboot) + gris (bloqueados sin acción).

### Verificación post-clean

Tras limpiar, **re-escanear** automáticamente y mostrar:

```
Resultado:
  - 3.2 GB liberados ahora ✓
  - 1.8 GB programados para reboot
  - 200 MB bloqueados (cerrar apps para liberar)
  - 0 MB requieren TrustedInstaller
```

Esto evita la sensación de "no funciona" que tiene el usuario actualmente.

### Reset de app UWP (escape hatch para casos UWP problemáticos)

Para los packages tipo `SpotifyAB.SpotifyMusic_zpdnekdrzrea0`, ofrecer un botón "Reset app" que llama a:

```powershell
Get-AppxPackage SpotifyAB.SpotifyMusic | Reset-AppxPackage
```

Esto destruye TODA la data del package (login incluido) pero garantiza limpieza al 100%. La UI debe avisar claramente: "Esto cerrará sesión en Spotify".

Disponible sólo en modo experto (toggle en Settings).

## Refactor del módulo

```
src-tauri/src/domain/cache.rs
  ├── pub fn analyze_locations(...) -> CleanPlan
  ├── pub fn execute_plan(plan: CleanPlan, opts: CleanOpts) -> Result<CleanReport>
  ├── pub fn verify_after(plan_id: Uuid) -> Vec<LocationResult>
  └── pub fn reset_uwp_app(package_family_name: &str) -> Result<()>

src-tauri/src/platform/process_lock.rs (NUEVO)
  ├── pub fn who_locks(path) -> Vec<LockingProcess>
  └── pub fn close_process_gracefully(pid) -> Result<()>

src-tauri/src/platform/pending_rename.rs (NUEVO)
  ├── pub fn schedule_delete_on_reboot(path) -> Result<()>
  └── pub fn list_pending_renames() -> Vec<PendingRename>
```

### IPC commands v2

```rust
#[tauri::command] pub async fn scan_cache_locations_v2(ids: Vec<String>) -> Result<CleanPlan>;
#[tauri::command] pub async fn execute_clean_plan(plan: CleanPlan, opts: CleanOpts) -> Result<CleanReport>;
#[tauri::command] pub async fn verify_clean(plan_id: String) -> Result<Vec<LocationResult>>;
#[tauri::command] pub async fn reset_uwp_app(pfn: String) -> Result<()>;
```

## UX cambios

### Pantalla Caché (rediseño)

```
┌────────────────────────────────────────────────────────────────┐
│ Caché del sistema                                              │
│ 12 ubicaciones · 5.2 GB potenciales · 3 bloqueadas             │
│                                                                │
│ ┌──────────────────────────────────────────────────────────┐  │
│ │ [Escanear] [Limpiar seleccionadas]   [⚙ Avanzado]        │  │
│ └──────────────────────────────────────────────────────────┘  │
│                                                                │
│ ✓ Listas para limpiar (9)                              3.2 GB │
│  ☑ Windows Temp                                       450 MB │
│  ☑ DISM componentes desuso                           1.2 GB │
│  ...                                                          │
│                                                                │
│ ⚠ Bloqueadas por procesos activos (3)                  1.8 GB │
│  ⚠ Spotify cache              Spotify.exe             900 MB │
│    └─ [Cerrar Spotify y limpiar] [Saltar]                    │
│  ⚠ Claude cache               Claude.exe              500 MB │
│    └─ [Cerrar Claude y limpiar] [Saltar]                     │
│  ⚠ Edge cache                 msedge.exe              400 MB │
│    └─ [Cerrar Edge y limpiar] [Saltar]                       │
│                                                                │
│ ✗ Requieren reboot (2)                                  200 MB │
│  Windows drivers cache         (reboot necesario)            │
└────────────────────────────────────────────────────────────────┘
```

## Criterio de "done"

- [ ] `who_locks(path)` funciona y devuelve PIDs correctos para Spotify, Claude, Chrome en una VM de prueba.
- [ ] `analyze_locations` devuelve un `CleanPlan` con las 4 secciones (ready / blocked / permission / not-exist).
- [ ] `execute_plan` con retry x3 elimina archivos liberados en el segundo intento.
- [ ] `schedule_delete_on_reboot` usa `MoveFileEx + PendingFileRenameOperations` y funciona en UWP (test con un archivo bajo `\Packages\`).
- [ ] La UI muestra el bloqueador con botón "Cerrar app".
- [ ] Tras "Limpiar", `verify_after` corre automáticamente y reporta resultado real.
- [ ] Sesión de testing en VM Win11 clean limpia 5+ ubicaciones con apps abiertas y reporta correctamente lo que liberó vs lo bloqueado.

## Dependencias
- `01-error-model-fix.md` — necesario para mostrar "Spotify.exe lo bloquea" en lugar de `[object Object]`.
- `03-process-manager.md` — provee el `kill_process` que se llama desde "Cerrar Spotify".
