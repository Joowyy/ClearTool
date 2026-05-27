// domain/cache.rs — catálogo de cachés y limpieza orquestada.

use crate::core::AppResult;
use crate::domain::{audit, catalog};
use crate::models::cache::{
    BlockedAction, BlockedLocation, CacheFilters, CacheLocation, CacheScanReport, CleanCacheInput,
    CleanPlan, CleanReport, CleanReportV2, CleanStrategy, ExecutePlanOpts, LocationResult,
    LocationStatus, PermissionLocation, PerLocationResult, ReadyLocation, SkipReason,
    SkippedLocation, VerifyLocationResult, VerifyReport,
};
use crate::models::process::LockingProcess;
use crate::models::restore::ReverseRecipe;
use crate::platform::filesystem;
use chrono::Utc;
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

pub fn list_locations() -> AppResult<Vec<CacheLocation>> {
    catalog::load_cache_locations()
}

pub fn scan(ids: &[String]) -> AppResult<Vec<CacheScanReport>> {
    let catalog_items = catalog::load_cache_locations()?;
    let mut reports = Vec::new();

    for id in ids {
        let Some(loc) = catalog_items.iter().find(|l| &l.id == id) else {
            continue;
        };

        let resolved = expand_path(&loc.path);
        let path = Path::new(&resolved);
        let exists = path.exists();
        let (bytes, files, _) = if exists {
            filesystem::directory_stats(path, false).unwrap_or((0, 0, 0))
        } else {
            (0, 0, 0)
        };

        let (filtered_bytes, filtered_files) = if exists && has_filters(&loc.filters) {
            filesystem::directory_stats_filtered(path, false, |p, m| {
                file_passes_filters(p, m, &loc.filters)
            })
            .unwrap_or((bytes, files))
        } else {
            (bytes, files)
        };

        reports.push(CacheScanReport {
            id: id.clone(),
            resolved_path: resolved,
            exists,
            bytes,
            file_count: files,
            matched_after_filters: filtered_files,
            bytes_after_filters: filtered_bytes,
        });
    }

    Ok(reports)
}

pub fn clean<F>(input: &CleanCacheInput, mut emit: F) -> AppResult<CleanReport>
where
    F: FnMut(&str, &str, &str, u64, u64),
{
    let started_at = Utc::now().to_rfc3339();
    let run_id = Uuid::new_v4().to_string();
    let catalog_items = catalog::load_cache_locations()?;
    let mut per_location = Vec::new();
    let mut total_bytes_freed: u64 = 0;
    let mut total_files_deleted: u64 = 0;
    let mut items_affected: Vec<String> = Vec::new();

    let restore_seq = if input.create_restore_point && !input.dry_run {
        crate::platform::restore_point::create(
            &format!("ClearTool — cache clean ({} ubicaciones)", input.ids.len()),
            12,
            true,
        )
        .ok()
    } else {
        None
    };

    for id in &input.ids {
        let Some(loc) = catalog_items.iter().find(|l| &l.id == id) else {
            per_location.push(PerLocationResult {
                id: id.clone(),
                status: "not-in-catalog".to_string(),
                bytes_freed: 0,
                files_deleted: 0,
                errors: vec!["id no presente en cache-locations.json".to_string()],
            });
            continue;
        };

        let resolved = expand_path(&loc.path);
        let path = Path::new(&resolved);

        emit("info", &loc.display_name, &format!("Iniciando escaneo de: {}", resolved), 0, 0);

        if !path.exists() {
            emit("warn", &loc.display_name, "La ruta no existe", 0, 0);
            per_location.push(PerLocationResult {
                id: id.clone(),
                status: "missing".to_string(),
                bytes_freed: 0,
                files_deleted: 0,
                errors: Vec::new(),
            });
            continue;
        }

        if input.dry_run {
            let (bytes, files) = if has_filters(&loc.filters) {
                filesystem::directory_stats_filtered(path, false, |p, m| {
                    file_passes_filters(p, m, &loc.filters)
                })
                .unwrap_or((0, 0))
            } else {
                let (b, f, _) = filesystem::directory_stats(path, false).unwrap_or((0, 0, 0));
                (b, f)
            };
            emit(
                "info",
                &loc.display_name,
                &format!(
                    "Encontrados {} archivos, {} en total (dry-run)",
                    files,
                    format_bytes(bytes)
                ),
                0,
                0,
            );
            per_location.push(PerLocationResult {
                id: id.clone(),
                status: "dry-run".to_string(),
                bytes_freed: bytes,
                files_deleted: files,
                errors: Vec::new(),
            });
            continue;
        }

        let result = if has_filters(&loc.filters) {
            filesystem::delete_recursive_filtered(path, |p, m| {
                file_passes_filters(p, m, &loc.filters)
            })
        } else {
            filesystem::delete_recursive_robust(path)
        };

        total_bytes_freed = total_bytes_freed.saturating_add(result.bytes_freed);
        total_files_deleted = total_files_deleted.saturating_add(result.files_deleted);

        emit(
            "info",
            &loc.display_name,
            &format!(
                "Completado: {} archivos eliminados, {} liberados",
                result.files_deleted,
                format_bytes(result.bytes_freed)
            ),
            result.bytes_freed,
            result.files_deleted,
        );

        for pending in &result.pending_reboot {
            emit(
                "warn",
                &loc.display_name,
                &format!(
                    "Archivo en uso, se eliminará en el próximo reboot: {}",
                    pending
                ),
                0,
                0,
            );
        }
        for err in &result.errors {
            emit("error", &loc.display_name, &format!("Error: {}", err), 0, 0);
        }

        let mut errors = result.errors.clone();
        let status = if errors.is_empty() && result.pending_reboot.is_empty() {
            "ok"
        } else if result.bytes_freed > 0 {
            "partial"
        } else {
            "failed"
        };

        if !result.pending_reboot.is_empty() {
            errors.push(format!(
                "{} archivos marcados para borrado en reboot",
                result.pending_reboot.len()
            ));
        }

        items_affected.push(id.clone());
        per_location.push(PerLocationResult {
            id: id.clone(),
            status: status.to_string(),
            bytes_freed: result.bytes_freed,
            files_deleted: result.files_deleted,
            errors,
        });
    }

    // Audit entry — uno por run
    if !input.dry_run && !items_affected.is_empty() {
        let entry = audit::make_entry(
            "cache",
            "clean",
            false,
            restore_seq,
            items_affected.clone(),
            ReverseRecipe::Noop {
                reason: format!(
                    "borrado de archivos; usar restore point seq #{} si aplica",
                    restore_seq
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "<sin punto>".into())
                ),
            },
            "success",
            None,
        );
        let _ = audit::write_entry(&entry);
    }

    Ok(CleanReport {
        run_id,
        started_at,
        finished_at: Utc::now().to_rfc3339(),
        restore_point_id: restore_seq,
        per_location,
        total_bytes_freed,
        total_files_deleted,
    })
}

fn has_filters(f: &CacheFilters) -> bool {
    f.older_than_days.is_some() || !f.exclude.is_empty()
}

pub fn file_passes_filters(path: &Path, metadata: &std::fs::Metadata, filters: &CacheFilters) -> bool {
    if let Some(days) = filters.older_than_days {
        let now = SystemTime::now();
        let modified = metadata.modified().ok();
        if let Some(m) = modified {
            let age_days = now
                .duration_since(m)
                .map(|d| d.as_secs() / 86400)
                .unwrap_or(0);
            if age_days < days {
                return false;
            }
        }
    }

    if !filters.include.is_empty() {
        let matches_include = filters.include.iter().any(|pattern| {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if glob_match(pattern, name) {
                    return true;
                }
            }
            if pattern.contains("**") || pattern.contains('/') || pattern.contains('\\') {
                let path_str = path.to_string_lossy();
                if glob_match(pattern, &path_str) {
                    return true;
                }
            }
            false
        });
        if !matches_include {
            return false;
        }
    }

    for pattern in &filters.exclude {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if glob_match(pattern, name) {
                return false;
            }
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", ext);
            if glob_match(pattern, &ext_with_dot) {
                return false;
            }
        }
    }

    true
}

fn glob_match(pattern: &str, text: &str) -> bool {
    match glob::Pattern::new(pattern) {
        Ok(p) => p.matches(text),
        Err(_) => text == pattern,
    }
}

#[cfg(test)]
mod glob_tests {
    use super::glob_match;

    #[test]
    fn glob_matches_double_star() {
        assert!(glob_match("**/*.tmp", "foo/bar/baz.tmp"));
        assert!(!glob_match("**/*.tmp", "foo/bar/baz.log"));
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn expand_path(template: &str) -> String {
    let mut out = template.to_string();
    let vars = [
        "APPDATA",
        "LOCALAPPDATA",
        "TEMP",
        "TMP",
        "WINDIR",
        "USERPROFILE",
        "PROGRAMDATA",
        "SYSTEMROOT",
    ];
    for v in vars {
        let token = format!("%{}%", v);
        if let Ok(val) = std::env::var(v) {
            out = out.replace(&token, &val);
        }
    }
    out
}

// ── v2: analyze_locations, execute_plan, verify_after_clean ──

pub fn is_disallowed(id: &str, path: &str) -> bool {
    catalog::is_cache_path_denied(path, id)
}

/// Analiza las ubicaciones del catálogo y devuelve un CleanPlan clasificado.
pub fn analyze_locations(ids: &[String]) -> AppResult<CleanPlan> {
    let catalog_items = catalog::load_cache_locations()?;
    let plan_id = Uuid::new_v4().to_string();
    let generated_at = Utc::now().to_rfc3339();

    let mut ready = Vec::new();
    let mut blocked = Vec::new();
    let mut permission_issues = Vec::new();
    let mut skipped = Vec::new();

    for id in ids {
        let entry = match catalog_items.iter().find(|e| &e.id == id) {
            Some(e) => e,
            None => {
                skipped.push(SkippedLocation {
                    id: id.clone(),
                    display_name: id.clone(),
                    reason: SkipReason::DisallowedByAllowlist,
                });
                continue;
            }
        };

        if is_disallowed(&entry.id, &entry.path) {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::DisallowedByAllowlist,
            });
            continue;
        }

        let resolved = match resolve_env_vars(&entry.path) {
            Ok(r) => r,
            Err(_) => {
                skipped.push(SkippedLocation {
                    id: entry.id.clone(),
                    display_name: entry.display_name.clone(),
                    reason: SkipReason::DoesNotExist,
                });
                continue;
            }
        };
        let path = Path::new(&resolved);

        if !path.exists() {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::DoesNotExist,
            });
            continue;
        }

        let (bytes, file_count, oldest) = match scan_path_stats(path) {
            Ok(s) => s,
            Err(e) => {
                if matches!(e, crate::core::AppError::Permission(_)) {
                    permission_issues.push(PermissionLocation {
                        id: entry.id.clone(),
                        display_name: entry.display_name.clone(),
                        resolved_path: resolved,
                        bytes: 0,
                        reason: format!("{}", e),
                    });
                    continue;
                }
                skipped.push(SkippedLocation {
                    id: entry.id.clone(),
                    display_name: entry.display_name.clone(),
                    reason: SkipReason::DoesNotExist,
                });
                continue;
            }
        };

        if bytes == 0 {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::Empty,
            });
            continue;
        }

        let lockers = crate::platform::processes::who_locks_path(path).unwrap_or_default();

        if !lockers.is_empty() {
            let suggested_action = suggest_action(&lockers, &entry.category);
            blocked.push(BlockedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                resolved_path: resolved,
                bytes,
                locked_by: lockers,
                suggested_action,
            });
        } else {
            ready.push(ReadyLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                resolved_path: resolved,
                bytes,
                file_count,
                strategy: resolve_strategy(&entry.strategy, &entry.category, &entry.path),
                age_oldest_file: oldest,
            });
        }
    }

    let total_estimated = ready.iter().map(|r| r.bytes).sum();
    let total_blocked = blocked.iter().map(|b| b.bytes).sum();

    Ok(CleanPlan {
        plan_id,
        generated_at,
        ready,
        blocked,
        permission_issues,
        skipped,
        total_estimated_bytes: total_estimated,
        total_blocked_bytes: total_blocked,
    })
}

fn resolve_env_vars(path: &str) -> Result<String, std::env::VarError> {
    let mut out = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut var = String::new();
            while let Some(&next) = chars.peek() {
                if next == '%' {
                    chars.next();
                    break;
                }
                var.push(next);
                chars.next();
            }
            out.push_str(&std::env::var(&var)?);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn scan_path_stats(path: &Path) -> AppResult<(u64, u32, Option<String>)> {
    let mut total_bytes: u64 = 0;
    let mut file_count: u32 = 0;
    let mut oldest: Option<SystemTime> = None;

    fn walk(
        p: &Path,
        bytes: &mut u64,
        count: &mut u32,
        oldest: &mut Option<SystemTime>,
    ) -> AppResult<()> {
        let entries = std::fs::read_dir(p).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                crate::core::AppError::Permission(format!("{}: {}", p.display(), e))
            } else {
                crate::core::AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{}: {}", p.display(), e),
                ))
            }
        })?;
        for entry_r in entries {
            let entry = match entry_r {
                Ok(e) => e,
                Err(_) => continue,
            };
            let md = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if md.is_dir() {
                let _ = walk(&entry.path(), bytes, count, oldest);
            } else {
                *bytes += md.len();
                *count += 1;
                if let Ok(modified) = md.modified() {
                    match *oldest {
                        Some(o) if modified < o => *oldest = Some(modified),
                        None => *oldest = Some(modified),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    walk(path, &mut total_bytes, &mut file_count, &mut oldest)?;
    let oldest_str = oldest.map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339());
    Ok((total_bytes, file_count, oldest_str))
}

fn suggest_action(lockers: &[LockingProcess], _category: &str) -> BlockedAction {
    // Si CUALQUIER locker es del shell o sistema, nunca proponer cerrar.
    // Incidente 2026-05-27: explorer.exe fue cerrado al limpiar thumbcache.
    let any_critical = lockers.iter().any(|l| {
        crate::platform::processes::is_protected_by_name(&l.name)
    });
    if any_critical {
        return BlockedAction::ScheduleReboot;
    }

    if let Some(main) = lockers.first() {
        return BlockedAction::CloseProcess {
            pid: main.pid,
            process_name: main.name.clone(),
        };
    }
    BlockedAction::SkipOnly {
        reason: "Bloqueado por proceso del sistema sin UI".into(),
    }
}

fn resolve_strategy(catalog_strategy: &Option<String>, category: &str, path: &str) -> CleanStrategy {
    if let Some(s) = catalog_strategy {
        match s.to_lowercase().as_str() {
            "direct-delete" => return CleanStrategy::DirectDelete,
            "browser-aware" => return CleanStrategy::BrowserAware,
            "process-locked" => return CleanStrategy::ProcessLocked,
            "system-restart-required" => return CleanStrategy::SystemRestartRequired,
            "take-ownership-and-delete" => return CleanStrategy::TakeOwnershipAndDelete,
            _ => {}
        }
    }
    parse_strategy(category, path)
}

fn parse_strategy(category: &str, path: &str) -> CleanStrategy {
    let cat_lower = category.to_lowercase();
    let path_lower = path.to_lowercase();

    if cat_lower.contains("uwp") || path_lower.contains("\\packages\\") {
        if let Some(pfn) = extract_package_family(&path_lower) {
            return CleanStrategy::UwpAppAware {
                package_family_name: pfn,
            };
        }
    }
    if cat_lower.contains("browser")
        || path_lower.contains("chrome")
        || path_lower.contains("firefox")
        || path_lower.contains("edge")
    {
        return CleanStrategy::BrowserAware;
    }
    if cat_lower.contains("windows-update") || cat_lower.contains("dism") {
        return CleanStrategy::SystemRestartRequired;
    }
    if path_lower.contains("winsxs") || path_lower.contains("component") {
        return CleanStrategy::TakeOwnershipAndDelete;
    }
    CleanStrategy::DirectDelete
}

fn extract_package_family(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split("\\packages\\").collect();
    if parts.len() < 2 {
        return None;
    }
    let after = parts[1];
    if let Some(end) = after.find('\\') {
        Some(after[..end].to_string())
    } else {
        Some(after.to_string())
    }
}

/// Ejecuta un CleanPlan con retry y estrategias.
pub async fn execute_plan<F>(
    plan: CleanPlan,
    opts: ExecutePlanOpts,
    mut emit_progress: F,
) -> AppResult<CleanReportV2>
where
    F: FnMut(&str, &str, &str, u64, u64) + Send + Sync,
{

    let started_at = Utc::now().to_rfc3339();
    let run_id = Uuid::new_v4().to_string();

    let restore_point_seq = if !opts.dry_run && opts.create_restore_point {
        let desc = format!(
            "ClearTool — caché ({} ubicaciones)",
            plan.ready.len()
        );
        crate::platform::restore_point::create(&desc, 12, true).ok()
    } else {
        None
    };

    let mut closed_pids: Vec<u32> = Vec::new();
    if opts.auto_close_blocking {
        for blocked in &plan.blocked {
            if let BlockedAction::CloseProcess { pid, process_name } = &blocked.suggested_action {
                // Defensa en profundidad: rechazar aunque el plan los traiga
                // (catálogo desactualizado, plan antiguo, etc.).
                if crate::platform::processes::is_protected_by_name(process_name)
                    || crate::platform::processes::is_system_protected_pid_lookup(*pid)
                {
                    emit_progress(
                        "warn",
                        &blocked.display_name,
                        &format!("Omitido cierre de {} (PID {}) — proceso protegido", process_name, pid),
                        0,
                        0,
                    );
                    continue;
                }

                emit_progress(
                    "info",
                    &blocked.display_name,
                    &format!("Cerrando proceso PID {}", pid),
                    0,
                    0,
                );
                let ok = crate::platform::processes::close_gracefully(*pid, 5000).await;
                if ok.unwrap_or(false) {
                    closed_pids.push(*pid);
                }
            }
        }
    }

    let mut per_location: Vec<LocationResult> = Vec::new();
    let mut total_bytes_freed: u64 = 0;
    let mut total_bytes_scheduled: u64 = 0;
    let mut total_bytes_failed: u64 = 0;

    for ready_loc in &plan.ready {
        emit_progress(
            "info",
            &ready_loc.display_name,
            &format!("Limpiando: {}", ready_loc.resolved_path),
            0,
            0,
        );

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
            execute_one_location(ready_loc, &opts, &mut emit_progress).await
        };

        total_bytes_freed += result.bytes_freed;
        total_bytes_scheduled += result.bytes_scheduled;
        if matches!(result.status, LocationStatus::Failed) {
            total_bytes_failed += ready_loc.bytes;
        }

        per_location.push(result);
    }

    if opts.schedule_blocked_for_reboot && !opts.dry_run {
        for blocked in &plan.blocked {
            if matches!(
                &blocked.suggested_action,
                BlockedAction::ScheduleReboot | BlockedAction::SkipOnly { .. }
            ) {
                let path = Path::new(&blocked.resolved_path);
                let _ = walk_and_schedule(path);
            }
        }
    }

    let report = CleanReportV2 {
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
    };

    // El plan cacheado deja de ser válido tras una limpieza real.
    if !opts.dry_run {
        crate::domain::cache_background::invalidate();
        tokio::spawn(async {
            let _ = crate::domain::cache_background::recompute_and_store().await;
        });
    }

    Ok(report)
}

async fn execute_one_location<F>(
    loc: &ReadyLocation,
    _opts: &ExecutePlanOpts,
    emit: &mut F,
) -> LocationResult
where
    F: FnMut(&str, &str, &str, u64, u64) + Send + Sync,
{
    let start = std::time::Instant::now();
    let path = std::path::Path::new(&loc.resolved_path);

    let mut bytes_freed: u64 = 0;
    let mut bytes_scheduled: u64 = 0;
    let mut files_deleted: u32 = 0;
    let mut files_scheduled: u32 = 0;
    let mut files_failed: u32 = 0;
    let mut error_msg: Option<String> = None;

    if let Err(e) = walk_and_delete(
        path,
        &mut bytes_freed,
        &mut bytes_scheduled,
        &mut files_deleted,
        &mut files_scheduled,
        &mut files_failed,
        true,
    )
    .await
    {
        error_msg = Some(format!("{}", e));
        emit("error", &loc.display_name, &format!("{}", e), 0, 0);
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
        Err(e) => {
            return Err(crate::core::AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("read_dir {}: {}", path.display(), e),
            )))
        }
    };

    for entry in entries {
        let entry_path = entry.path();
        let md = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = md.len();

        if md.is_dir() {
            Box::pin(walk_and_delete(
                &entry_path,
                bytes_freed,
                bytes_scheduled,
                files_deleted,
                files_scheduled,
                files_failed,
                schedule_on_fail,
            ))
            .await?;
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
                if attempt < 2
                    && (e.kind() == std::io::ErrorKind::PermissionDenied
                        || e.raw_os_error() == Some(32))
                {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                    continue;
                }
                return Err(crate::core::AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{}: {}", path.display(), e),
                )));
            }
        }
    }
    unreachable!()
}

fn walk_and_schedule(path: &std::path::Path) -> AppResult<()> {
    let entries = match std::fs::read_dir(path) {
        Ok(it) => it.filter_map(|r| r.ok()).collect::<Vec<_>>(),
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            walk_and_schedule(&entry_path)?;
        } else {
            let _ = crate::platform::pending_rename::schedule_delete_on_reboot(&entry_path);
        }
    }
    Ok(())
}

/// Verifica el resultado real tras la limpieza.
pub fn verify_after_clean(
    plan: &CleanPlan,
    _report: &CleanReportV2,
) -> AppResult<VerifyReport> {
    let mut per_location = Vec::new();
    let mut total_actually_freed = 0u64;
    let mut total_still_present = 0u64;

    let pendings = crate::platform::pending_rename::list_pending_renames().unwrap_or_default();

    for ready in &plan.ready {
        let path = std::path::Path::new(&ready.resolved_path);
        let bytes_after = if path.exists() {
            scan_path_stats(path).map(|(b, _, _)| b).unwrap_or(0)
        } else {
            0
        };

        let bytes_before = ready.bytes;
        let bytes_actually_freed = bytes_before.saturating_sub(bytes_after);

        let files_pending = pendings
            .iter()
            .filter(|p| {
                p.is_delete && path_under(&p.source, &ready.resolved_path)
            })
            .count() as u32;

        let success_percent = if bytes_before == 0 {
            100.0
        } else {
            (bytes_actually_freed as f32 / bytes_before as f32) * 100.0
        };

        total_actually_freed += bytes_actually_freed;
        total_still_present += bytes_after;

        per_location.push(VerifyLocationResult {
            id: ready.id.clone(),
            display_name: ready.display_name.clone(),
            bytes_before,
            bytes_after,
            bytes_actually_freed,
            files_pending_reboot: files_pending,
            success_percent,
        });
    }

    Ok(VerifyReport {
        plan_id: plan.plan_id.clone(),
        verified_at: chrono::Utc::now().to_rfc3339(),
        per_location,
        total_actually_freed,
        total_still_present,
    })
}

fn path_under(child: &str, parent: &str) -> bool {
    let c = child.to_lowercase().replace('/', "\\");
    let p = parent.to_lowercase().replace('/', "\\");
    c.starts_with(&p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_action_never_closes_shell() {
        let lockers = vec![
            LockingProcess { pid: 1234, name: "explorer.exe".into(), path: None },
            LockingProcess { pid: 5678, name: "spotify.exe".into(), path: None },
        ];
        match suggest_action(&lockers, "user-cache") {
            BlockedAction::ScheduleReboot => {}
            other => panic!("Esperado ScheduleReboot, obtuve {:?}", other),
        }
    }

    #[test]
    fn suggest_action_closes_user_apps() {
        let lockers = vec![
            LockingProcess { pid: 5678, name: "spotify.exe".into(), path: None },
        ];
        match suggest_action(&lockers, "user-cache") {
            BlockedAction::CloseProcess { process_name, .. } => {
                assert_eq!(process_name, "spotify.exe");
            }
            other => panic!("Esperado CloseProcess, obtuve {:?}", other),
        }
    }

    #[test]
    fn suggest_action_all_shell_lockers_schedules_reboot() {
        for name in &["explorer.exe", "shellexperiencehost.exe", "searchhost.exe", "dwm.exe"] {
            let lockers = vec![LockingProcess { pid: 100, name: name.to_string(), path: None }];
            match suggest_action(&lockers, "cache") {
                BlockedAction::ScheduleReboot => {}
                other => panic!("{}: Esperado ScheduleReboot, obtuve {:?}", name, other),
            }
        }
    }
}
