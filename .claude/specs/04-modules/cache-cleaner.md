# 04.2 — Módulo Cache Cleaner

## Propósito

Eliminar archivos en ubicaciones de caché conocidas y poco visibles del SO (Windows Update, DISM/CBS logs, prefetch, thumbnails, fonts cache, font fallback cache, package cache de Windows Installer huérfano, NVIDIA/Intel/AMD shader caches, Edge/Chrome/Firefox caches, etc.) **respetando** el principio de reversibilidad y mostrando al usuario exactamente qué se va a borrar.

Este módulo nunca toca archivos de usuario en `Documents`, `Pictures`, `Videos`, `Desktop` ni `OneDrive`. Está restringido al catálogo `cache-locations.json`.

## Stakeholders

- Subagente líder: `tauri-rust-backend` con apoyo de `windows-systems-expert` para precondiciones (services, COM apartments).
- Revisión obligatoria: `security-auditor` para cualquier cambio al catálogo.

## Capabilities Tauri requeridas

- `default.json` para `list_cache_locations` y `scan_cache_locations` (solo lectura).
- `elevated.json` para `clean_cache_locations` (escritura/borrado).

## Comandos expuestos

```rust
// commands/cache.rs

#[tauri::command]
pub async fn list_cache_locations() -> Result<Vec<CacheLocation>, AppError>;

#[tauri::command]
pub async fn scan_cache_locations(
    ids: Vec<String>,
) -> Result<Vec<CacheScanReport>, AppError>;

#[tauri::command]
pub async fn clean_cache_locations(
    input: CleanCacheInput,
) -> Result<CleanReport, AppError>;
```

### Modelos

```rust
// models/cache.rs

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct CacheLocation {
    pub id: String,                 // ej. "windows-update.softwaredistribution"
    pub display_name: String,
    pub category: CacheCategory,
    pub path_template: String,      // %SYSTEMROOT%\\SoftwareDistribution\\Download
    pub requires_admin: bool,
    pub risk: Risk,                 // Low | Medium | High
    pub preconditions: Vec<Precondition>,
    pub filters: CacheFilters,
    pub consequences: Vec<String>,  // strings i18n keys o literales
    pub average_size: Option<String>, // "200MB-10GB" textual
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum CacheCategory {
    UserTemp, SystemTemp, WindowsUpdate, Logs, Prefetch,
    Thumbnails, Fonts, Browser, Gpu, PackageCache, Misc,
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum Precondition {
    /// Detener un servicio antes; restaurarlo después.
    StopService(String),
    /// Cerrar un proceso si corre. Si está corriendo y `kill = false`, abortar.
    CloseProcess { name: String, kill: bool },
    /// Build mínimo de Windows.
    MinBuild(u32),
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct CacheFilters {
    pub older_than_days: Option<u32>,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    /// Si está vacío, borra todo. Si no, solo extensiones listadas.
    pub extensions: Vec<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CacheScanReport {
    pub id: String,
    pub resolved_path: String,
    pub exists: bool,
    pub bytes: u64,
    pub file_count: u64,
    pub matched_after_filters: u64, // archivos que se borrarían
    pub bytes_after_filters: u64,
    pub blocked_by: Vec<BlockerReport>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BlockerReport {
    pub kind: BlockerKind, // ProcessRunning, ServiceCantStop, NotElevated, MissingPath
    pub detail: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CleanCacheInput {
    pub ids: Vec<String>,
    pub dry_run: bool,
    /// Si true, crea restore point antes de empezar (default true cuando dry_run=false).
    pub create_restore_point: bool,
    /// Si true, cierra procesos bloqueantes con kill=true; si false, los respeta y omite la entrada.
    pub force_close_processes: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CleanReport {
    pub run_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub restore_point_id: Option<u32>,
    pub per_location: Vec<PerLocationResult>,
    pub total_bytes_freed: u64,
    pub total_files_deleted: u64,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerLocationResult {
    pub id: String,
    pub status: CleanStatus, // Done | Skipped | Partial | Failed
    pub bytes_freed: u64,
    pub files_deleted: u64,
    pub errors: Vec<String>,
}
```

## Eventos

| Evento | Payload | Cuándo |
|---|---|---|
| `cache:scan-progress` | `{ id, scanned, current_path }` | Cada 200 ms o cada 2 000 archivos |
| `cache:clean-start` | `{ run_id, ids, dry_run }` | Al iniciar |
| `cache:clean-progress` | `{ run_id, id, files_processed, bytes_processed }` | Cada 250 ms |
| `cache:clean-location-done` | `PerLocationResult` | Al cerrar cada ubicación |
| `cache:clean-done` | `CleanReport` | Al terminar todo |

## Catálogo

Vive en `.claude/skills/cache-scanner/RESOURCES/cache-locations.json` y se carga al arrancar la app por `services::catalog::load_cache_catalog()`. El loader:

1. Valida contra `cache-locations.schema.json`.
2. Resuelve variables de entorno (`%SYSTEMROOT%`, `%LOCALAPPDATA%`) por usuario actual.
3. Filtra entradas cuyo `MinBuild` no se cumpla.
4. Cachea en memoria (`OnceCell`).

## Flujo de ejecución

```
1. UI: usuario marca N ubicaciones, click "Limpiar".
2. <ConfirmDestructive> abre con resumen.
3. invoke('scan_cache_locations', { ids }) — paralelo, 8 workers max.
4. UI muestra el desglose; usuario revisa y aprueba.
5. invoke('clean_cache_locations', { ids, dry_run: false, create_restore_point: true }).
6. Backend:
   a. is_elevated() — si false y alguna entrada lo requiere, error temprano `NotElevated`.
   b. Crear restore point (descrito en restore-point-system.md).
   c. Para cada id, secuencialmente (no paralelo: precondiciones servicios pueden chocarse):
      - Resolver preconditions (stop services, kill procesos si autorizado).
      - Iterar archivos aplicando filters.
      - Borrar uno por uno con retry x3 ante `ERROR_SHARING_VIOLATION`.
      - Registrar bytes_freed/files_deleted.
      - Restaurar servicios al estado anterior aunque el borrado falle.
   d. Loguear `CleanReport` en `audit.jsonl`.
7. UI muestra reporte con CTA "Ver detalle" y "Deshacer (System Restore)".
```

## Borrado seguro

- `std::fs::remove_file` para archivos. Para directorios vacíos, `remove_dir`. **Nunca** `remove_dir_all` recursivo en una sola llamada: queremos contabilizar archivo por archivo y reaccionar a errores individuales.
- Validación de cada path antes de borrar:
  - Debe estar **dentro** del `resolved_path` (canonicalizar y comparar prefijos).
  - No es un junction que apunte fuera (re-canonicalizar tras leer reparse data).
  - No es uno de los paths blacklisted globales (`C:\Windows\System32`, `C:\Users\<u>\Documents`, etc.).
- En `dry_run`, en vez de borrar, sumar bytes y contar.

## Manejo de servicios

```rust
// services/service_manager.rs (extracto, módulo full en 04.4)
pub struct ServiceGuard {
    name: String,
    previous_state: ServiceState,
}

pub fn stop_with_guard(name: &str) -> Result<ServiceGuard, AppError>;
impl Drop for ServiceGuard {
    fn drop(&mut self) {
        // Reactivar al estado previo (tolerante a fallos: log warning).
    }
}
```

Usar `windows-service` crate. `wuauserv` y `bits` son los más comunes. Timeout 30 s por servicio. Si no se logra detener tras 30 s, marcar la ubicación como `BlockerReport::ServiceCantStop` y saltar.

## Procesos bloqueantes

Algunos cachés (Edge, Chrome, Firefox) requieren cerrar el navegador. Estrategia:

1. Detectar via `tasklist` (rápido) o `EnumProcesses` (preciso).
2. Si `force_close_processes = false`: marcar `ProcessRunning` y saltar.
3. Si `true`: enviar WM_CLOSE (graceful) → esperar 5 s → `TerminateProcess` solo si sigue vivo. Loguear ambas etapas.

## Filtros

```rust
fn matches(filters: &CacheFilters, entry: &DirEntry) -> bool {
    // older_than_days
    if let Some(days) = filters.older_than_days {
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
        if let Some(t) = modified {
            let cutoff = SystemTime::now() - Duration::from_secs(days as u64 * 86_400);
            if t > cutoff { return false; }
        }
    }
    // extensions
    if !filters.extensions.is_empty() {
        let ext = entry.path().extension().and_then(|s| s.to_str()).unwrap_or("");
        if !filters.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return false;
        }
    }
    // include / exclude globs (globset crate)
    // …
    true
}
```

## Errores

| Variante | Causa típica | Retry/UX |
|---|---|---|
| `NotElevated` | entrada requería admin | Banner "reiniciar como admin" |
| `External` | service no detuvo | Saltar entrada, log + reportar |
| `Io(SharingViolation)` | archivo en uso | Retry x3 con backoff 100/300/900 ms |
| `Permission` | path fuera de catálogo (validación falló) | Aborta toda la run, log crítico |
| `RestoreUnavailable` | System Restore deshabilitado | Avisar y preguntar si proceder sin punto |

## Idempotencia

- Re-ejecutar la misma `clean_cache_locations` no debe fallar ni duplicar registros.
- El log de auditoría usa `run_id` único; si la operación se repite, se crea otro `run_id`.
- El restore point que se crea cada run sí se duplica (uno por sesión destructiva), por diseño.

## Tests

- Fixture `temp_dir()` con árbol controlado: validar `dry_run` no borra, `clean` sí.
- Fixture con archivos modificados a mano (`set_modified`) para probar `older_than_days`.
- Mock de `ServiceGuard` para verificar que el `Drop` reactiva incluso si la operación panickea.
- Test contractual: ningún `id` del catálogo resuelve a un path fuera del root indicado.
- Test de fuzzing: paths con caracteres unicode, espacios, longitud > MAX_PATH.

## UI (resumen, detalle en spec 03)

- Página `/cache` con tabla virtualizada:
  - Checkbox por fila.
  - Columnas: nombre, categoría (chip), tamaño detectado, riesgo, bloqueadores.
  - Filtros: por categoría, por riesgo, por requiere-admin.
  - Toolbar: "Escanear seleccionadas", "Limpiar seleccionadas", "Seleccionar todo seguro" (preset risk = Low).
- Modal `<ConfirmDestructive>` con tabs Resumen / Detalle / Riesgos / Reversa.

## Futuro (no en MVP)

- Programar limpiezas (Task Scheduler).
- Detectar cachés "huérfanas" no listadas en el catálogo (heurística: directorios > 1 GB con LastWriteTime > 90 días en `%LOCALAPPDATA%\Packages`).
