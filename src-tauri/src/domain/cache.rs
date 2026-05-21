// domain/cache.rs — catálogo de cachés y limpieza orquestada.
//
// El catálogo viene del JSON `cache-locations.json` (allowlist). Cualquier
// path fuera del catálogo es rechazado por `domain::catalog::resolve_cache_id`.

use crate::core::AppResult;
use crate::domain::catalog;
use crate::models::cache::{CacheLocation, CacheScanReport, CleanCacheInput, CleanReport, PerLocationResult};
use crate::platform::filesystem;
use chrono::Utc;
use std::path::Path;
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

        reports.push(CacheScanReport {
            id: id.clone(),
            resolved_path: resolved,
            exists,
            bytes,
            file_count: files,
            matched_after_filters: files,
            bytes_after_filters: bytes,
        });
    }

    Ok(reports)
}

pub fn clean<F>(input: &CleanCacheInput, mut emit: F) -> AppResult<CleanReport>
where
    F: FnMut(&str, &str, &str, u64, u64),
{
    let started_at = Utc::now().to_rfc3339();
    let catalog_items = catalog::load_cache_locations()?;
    let mut per_location = Vec::new();
    let mut total_bytes_freed: u64 = 0;
    let mut total_files_deleted: u64 = 0;

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
            let (bytes, files, _) = filesystem::directory_stats(path, false).unwrap_or((0, 0, 0));
            emit("info", &loc.display_name, &format!("Encontrados {} archivos, {} en total (dry-run)", files, format_bytes(bytes)), 0, 0);
            per_location.push(PerLocationResult {
                id: id.clone(),
                status: "dry-run".to_string(),
                bytes_freed: bytes,
                files_deleted: files,
                errors: Vec::new(),
            });
            continue;
        }

        let result = filesystem::delete_recursive_robust(path);
        total_bytes_freed = total_bytes_freed.saturating_add(result.bytes_freed);
        total_files_deleted = total_files_deleted.saturating_add(result.files_deleted);

        emit(
            "info",
            &loc.display_name,
            &format!("Completado: {} archivos eliminados, {} liberados",
                result.files_deleted, format_bytes(result.bytes_freed)),
            result.bytes_freed,
            result.files_deleted,
        );

        for pending in &result.pending_reboot {
            emit("warn", &loc.display_name, &format!("Archivo en uso, se eliminará en el próximo reboot: {}", pending), 0, 0);
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
            errors.push(format!("{} archivos marcados para borrado en reboot", result.pending_reboot.len()));
        }

        per_location.push(PerLocationResult {
            id: id.clone(),
            status: status.to_string(),
            bytes_freed: result.bytes_freed,
            files_deleted: result.files_deleted,
            errors,
        });
    }

    Ok(CleanReport {
        run_id: Uuid::new_v4().to_string(),
        started_at,
        finished_at: Utc::now().to_rfc3339(),
        restore_point_id: None,
        per_location,
        total_bytes_freed,
        total_files_deleted,
    })
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

/// Expande variables `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%`, `%WINDIR%`,
/// `%USERPROFILE%`, `%PROGRAMDATA%` en plantillas de paths del catálogo.
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
