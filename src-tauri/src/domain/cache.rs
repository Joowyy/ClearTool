// domain/cache.rs — catálogo de cachés y limpieza orquestada.

use crate::core::AppResult;
use crate::domain::{audit, catalog};
use crate::models::cache::{
    CacheFilters, CacheLocation, CacheScanReport, CleanCacheInput, CleanReport, PerLocationResult,
};
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
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with("*.") {
        let suffix = &pattern[1..];
        return text.ends_with(suffix);
    }
    text == pattern
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
