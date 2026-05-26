# Paso 05 — Ejecución: `execute_plan` con retry + estrategias

**Área**: 02-cache-engine
**Tiempo estimado**: 8-10 horas
**Dependencias**: Paso 03 (modelos), Paso 04 (analyze)

## Qué hacemos

Implementar la fase de ejecución: dado un `CleanPlan` aprobado por el usuario, ejecutar las operaciones (eliminar / programar reboot / cerrar procesos según `ExecutePlanOpts`).

## Por qué

Es el corazón. Aquí pasamos del "qué" (analyze) al "cómo" (ejecutar con retry, manejo de bloqueadores, schedule reboot si aplica).

## Archivos que tocamos

- `src-tauri/src/domain/cache.rs` (función `execute_plan`)
- `src-tauri/src/platform/processes.rs` (helpers para close_gracefully — si no están del Paso `03-process-manager`)

## Cómo

### 1. Esqueleto

```rust
// src-tauri/src/domain/cache.rs

pub async fn execute_plan(
    plan: CleanPlan,
    opts: ExecutePlanOpts,
    emit_progress: impl Fn(CleanProgressEvent) + Send + Sync,
) -> AppResult<CleanReportV2> {
    let started_at = Utc::now().to_rfc3339();
    let run_id = Uuid::new_v4().to_string();

    // 1) Restore point obligatorio si no dry-run
    let restore_point_seq = if !opts.dry_run && opts.create_restore_point {
        let desc = format!("ClearTool — caché ({} ubicaciones)", plan.ready.len());
        super::restore::ensure_or_create(&desc).ok().flatten()
    } else { None };

    // 2) Cerrar procesos bloqueadores si auto_close_blocking
    let mut closed_pids: Vec<u32> = Vec::new();
    if opts.auto_close_blocking {
        for blocked in &plan.blocked {
            if let BlockedAction::CloseProcess { pid, .. } = &blocked.suggested_action {
                emit_progress(CleanProgressEvent::ClosingProcess { pid: *pid });
                let ok = crate::platform::processes::close_gracefully(*pid, 5000).await;
                if ok.is_ok() {
                    closed_pids.push(*pid);
                }
            }
        }
    }

    // 3) Iterar ready
    let mut per_location: Vec<LocationResult> = Vec::new();
    let mut total_bytes_freed: u64 = 0;
    let mut total_bytes_scheduled: u64 = 0;
    let mut total_bytes_failed: u64 = 0;

    for ready_loc in &plan.ready {
        emit_progress(CleanProgressEvent::StartingLocation {
            id: ready_loc.id.clone(),
        });

        let result = if opts.dry_run {
            LocationResult {
                id: ready_loc.id.clone(),
                status: LocationStatus::Cleaned,
                bytes_freed: ready_loc.bytes,
                bytes_scheduled: 0,
                files_deleted: ready_loc.file_count,
                files_scheduled: 0,
                files_failed: 0,
                error: None,
                duration_ms: 0,
            }
        } else {
            execute_one_location(ready_loc, &opts, &emit_progress).await
        };

        total_bytes_freed += result.bytes_freed;
        total_bytes_scheduled += result.bytes_scheduled;
        total_bytes_failed += if matches!(result.status, LocationStatus::Failed) {
            ready_loc.bytes
        } else { 0 };

        per_location.push(result);
    }

    // 4) Si schedule_blocked_for_reboot, procesar blocked
    if opts.schedule_blocked_for_reboot && !opts.dry_run {
        for blocked in &plan.blocked {
            // Sólo si NO los cerramos antes
            if matches!(&blocked.suggested_action,
                BlockedAction::ScheduleReboot |
                BlockedAction::SkipOnly { .. })
            {
                schedule_location_for_reboot(blocked, &emit_progress).await;
            }
        }
    }

    Ok(CleanReportV2 {
        plan_id: plan.plan_id,
        run_id,
        started_at,
        finished_at: Utc::now().to_rfc3339(),
        restore_point_seq,
        per_location,
        total_bytes_freed,
        total_bytes_scheduled_reboot: total_bytes_scheduled,
        total_bytes_failed,
        closed_processes: closed_pids,
    })
}

async fn execute_one_location(
    loc: &ReadyLocation,
    opts: &ExecutePlanOpts,
    emit: &(impl Fn(CleanProgressEvent) + Send + Sync),
) -> LocationResult {
    let start = std::time::Instant::now();
    let path = std::path::Path::new(&loc.resolved_path);

    let mut bytes_freed: u64 = 0;
    let mut bytes_scheduled: u64 = 0;
    let mut files_deleted: u32 = 0;
    let mut files_scheduled: u32 = 0;
    let mut files_failed: u32 = 0;
    let mut error_msg: Option<String> = None;

    // Walk + delete con retry
    if let Err(e) = walk_and_delete(
        path,
        &mut bytes_freed,
        &mut bytes_scheduled,
        &mut files_deleted,
        &mut files_scheduled,
        &mut files_failed,
        opts.schedule_blocked_for_reboot,
    ).await {
        error_msg = Some(format!("{}", e));
    }

    let status = if files_failed == 0 && files_scheduled == 0 {
        LocationStatus::Cleaned
    } else if files_scheduled > 0 && files_failed == 0 {
        LocationStatus::PartialReboot
    } else {
        LocationStatus::Failed
    };

    LocationResult {
        id: loc.id.clone(),
        status,
        bytes_freed,
        bytes_scheduled,
        files_deleted,
        files_scheduled,
        files_failed,
        error: error_msg,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn walk_and_delete(
    path: &std::path::Path,
    bytes_freed: &mut u64,
    bytes_scheduled: &mut u64,
    files_deleted: &mut u32,
    files_scheduled: &mut u32,
    files_failed: &mut u32,
    schedule_on_fail: bool,
) -> AppResult<()> {
    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(it) => it.filter_map(|r| r.ok()).collect(),
        Err(e) => return Err(AppError::Io(format!("read_dir {}: {}", path.display(), e))),
    };

    for entry in entries {
        let entry_path = entry.path();
        let md = match entry.metadata() { Ok(m) => m, Err(_) => continue };
        let size = md.len();

        if md.is_dir() {
            // Recursivo
            Box::pin(walk_and_delete(
                &entry_path, bytes_freed, bytes_scheduled,
                files_deleted, files_scheduled, files_failed,
                schedule_on_fail,
            )).await?;
            // Intentar borrar el directorio vacío
            let _ = std::fs::remove_dir(&entry_path);
        } else {
            match delete_with_retry(&entry_path).await {
                Ok(_) => {
                    *bytes_freed += size;
                    *files_deleted += 1;
                }
                Err(_) if schedule_on_fail => {
                    match crate::platform::pending_rename::schedule_delete_on_reboot(&entry_path) {
                        Ok(_) => {
                            *bytes_scheduled += size;
                            *files_scheduled += 1;
                        }
                        Err(_) => *files_failed += 1,
                    }
                }
                Err(_) => *files_failed += 1,
            }
        }
    }
    Ok(())
}

async fn delete_with_retry(path: &std::path::Path) -> AppResult<()> {
    use std::time::Duration;
    let mut delay_ms = 100u64;
    for attempt in 0..3 {
        match std::fs::remove_file(path) {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt < 2 && (
                    e.kind() == std::io::ErrorKind::PermissionDenied ||
                    e.raw_os_error() == Some(32) // ERROR_SHARING_VIOLATION
                ) {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                    continue;
                }
                return Err(AppError::Io(format!("{}: {}", path.display(), e)));
            }
        }
    }
    unreachable!()
}

async fn schedule_location_for_reboot(
    blocked: &BlockedLocation,
    emit: &(impl Fn(CleanProgressEvent) + Send + Sync),
) {
    // Recursivo: schedule cada archivo dentro
    let path = std::path::Path::new(&blocked.resolved_path);
    let _ = walk_and_schedule(path, emit, &blocked.id);
}

fn walk_and_schedule(
    path: &std::path::Path,
    emit: &impl Fn(CleanProgressEvent),
    location_id: &str,
) -> AppResult<()> {
    let entries = match std::fs::read_dir(path) {
        Ok(it) => it.filter_map(|r| r.ok()).collect::<Vec<_>>(),
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            walk_and_schedule(&entry_path, emit, location_id)?;
        } else {
            let _ = crate::platform::pending_rename::schedule_delete_on_reboot(&entry_path);
        }
    }
    Ok(())
}
```

### 2. Eventos de progreso

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CleanProgressEvent {
    StartingLocation { id: String },
    ClosingProcess { pid: u32 },
    FileDeleted { path: String, bytes: u64 },
    FileScheduled { path: String, bytes: u64 },
    FileFailed { path: String, error: String },
    LocationCompleted { id: String, status: LocationStatus },
}
```

Emitirlos vía `app.emit("cache:progress", ...)` desde el comando IPC (Paso 08).

### 3. Sobre el `dry_run`

En dry_run, las funciones de delete y schedule NO se llaman. Pero el report devuelve qué SE HUBIERA hecho. Útil para que el usuario vea el preview exacto.

### 4. Sobre el restore point

`super::restore::ensure_or_create` ya existe en v0.1. Reusarla. Si falla (sin admin, sin protection enabled), `closed_processes` y la operación siguen, pero el report lo refleja con `restore_point_seq: None`.

### 5. Caveat de async

`std::fs::remove_file` es blocking. Si queremos paralelismo real, usar `tokio::fs::remove_file` y `tokio::task::spawn`. Para v1 está OK secuencial — un retry con timeout suele ser suficiente. Pero meter `tokio::time::sleep` ya implica que la función sea async (lo es).

## Criterio de done

- [ ] `execute_plan` devuelve `CleanReportV2` con totales correctos.
- [ ] `delete_with_retry` reintenta 3 veces con backoff exponencial.
- [ ] Si `schedule_blocked_for_reboot=true`, archivos fallidos van a PendingFileRenameOperations.
- [ ] Si `auto_close_blocking=true`, los procesos main-window se cierran ANTES de ejecutar.
- [ ] dry_run no toca nada en disco.
- [ ] Restore point se crea antes (verificable en audit log).
- [ ] Eventos `cache:progress` emitidos a lo largo del proceso.
- [ ] Tests integration: limpia un dir temporal con un archivo, verifica que se borra.

## Próximo paso

`06-verify-after-clean.md`.
