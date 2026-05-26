// domain/audit.rs — log de auditoría append-only y reversa fina.
//
// Cada operación destructiva escribe un `AuditEntry` con `reverse_recipe`.
// El archivo vive en %APPDATA%\ClearTool\audit.jsonl (ver core::config).

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;

use crate::core::{AppError, AppResult, config};
use crate::models::restore::{AuditEntry, ReverseRecipe};

const ROTATION_THRESHOLD: u64 = 10 * 1024 * 1024; // 10 MB
const LOCK_RETRY: u32 = 3;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(50);

pub fn write_entry(entry: &AuditEntry) -> AppResult<()> {
    rotate_if_needed()?;
    let path = config::audit_log_path();
    let line = serde_json::to_string(entry)
        .map_err(|e| AppError::Audit(format!("serialize: {}", e)))?;

    let mut last_err = None;
    for _ in 0..LOCK_RETRY {
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                writeln!(f, "{}", line)
                    .map_err(|e| AppError::Audit(format!("write: {}", e)))?;
                return Ok(());
            }
            Err(e) => {
                last_err = Some(e);
                std::thread::sleep(LOCK_RETRY_DELAY);
            }
        }
    }
    Err(AppError::Audit(format!(
        "open after {} retries: {:?}",
        LOCK_RETRY, last_err
    )))
}

pub fn make_entry(
    module: &str,
    operation: &str,
    dry_run: bool,
    restore_point_seq: Option<u32>,
    items_affected: Vec<String>,
    reverse_recipe: ReverseRecipe,
    status: &str,
    error: Option<String>,
) -> AuditEntry {
    AuditEntry {
        run_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        module: module.to_string(),
        operation: operation.to_string(),
        dry_run,
        restore_point_seq,
        items_affected,
        reverse_recipe,
        status: status.to_string(),
        error,
    }
}

pub fn list_log() -> AppResult<Vec<AuditEntry>> {
    let path = config::audit_log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Audit(format!("read: {}", e)))?;
    let mut entries = Vec::with_capacity(64);
    for (line_no, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<AuditEntry>(line) {
            Ok(e) => entries.push(e),
            Err(err) => {
                log::warn!("audit line {} skipped: {}", line_no, err);
            }
        }
    }
    Ok(entries)
}

pub fn find_entry(run_id: &str) -> AppResult<Option<AuditEntry>> {
    Ok(list_log()?.into_iter().find(|e| e.run_id == run_id))
}

pub fn revert_entry(run_id: &str) -> AppResult<()> {
    let entry = find_entry(run_id)?
        .ok_or_else(|| AppError::Audit(format!("entry {} not found", run_id)))?;

    if entry.dry_run {
        return Err(AppError::Audit(
            "dry-run entry cannot be reverted".into(),
        ));
    }

    match &entry.reverse_recipe {
        ReverseRecipe::Registry { operations } => {
            for op in operations {
                crate::platform::registry::write_value_or_delete(
                    &op.hive,
                    &op.key,
                    &op.name,
                    op.previous_value.as_ref(),
                )?;
            }
        }
        ReverseRecipe::Service {
            service_name,
            previous_start_type,
            previous_state,
        } => {
            crate::platform::services::set_start_type(service_name, previous_start_type)?;
            if previous_state == "Running" {
                let _ = crate::platform::services::start(service_name);
            }
        }
        ReverseRecipe::AppxReinstall {
            package_family_name,
            store_url,
        } => {
            if let Some(url) = store_url {
                let _ = open::that(url);
            } else {
                let url = format!(
                    "ms-windows-store://search?query={}",
                    package_family_name
                );
                let _ = open::that(url);
            }
        }
        ReverseRecipe::Noop { reason } => {
            return Err(AppError::Audit(format!(
                "entry no reversible: {}",
                reason
            )));
        }
    }

    // Loggear el revert como nueva entry
    let revert_entry = make_entry(
        &entry.module,
        &format!("revert:{}", entry.operation),
        false,
        None,
        entry.items_affected.clone(),
        ReverseRecipe::Noop {
            reason: format!("revert of {}", entry.run_id),
        },
        "success",
        None,
    );
    write_entry(&revert_entry)?;
    Ok(())
}

/// Ejecuta `op` con la red de seguridad completa:
/// 1. Crea restore point (si no dry-run).
/// 2. Ejecuta `op`, recibiendo el SequenceNumber.
/// 3. Escribe el AuditEntry final.
pub fn with_audit<F>(module: &str, operation: &str, dry_run: bool, op: F) -> AppResult<AuditEntry>
where
    F: FnOnce(Option<u32>) -> AppResult<AuditEntry>,
{
    let seq = if dry_run {
        None
    } else {
        let desc = format!("ClearTool — {} {}", module, operation);
        crate::domain::restore::ensure_or_create(&desc).ok().flatten()
    };

    let result = op(seq);

    match &result {
        Ok(entry) => {
            let _ = write_entry(entry);
        }
        Err(e) => {
            let failed = make_entry(
                module,
                operation,
                dry_run,
                seq,
                Vec::new(),
                ReverseRecipe::Noop {
                    reason: "failed before applying changes".into(),
                },
                "failed",
                Some(format!("{}", e)),
            );
            let _ = write_entry(&failed);
        }
    }

    result
}

fn rotate_if_needed() -> AppResult<()> {
    let path = config::audit_log_path();
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if size < ROTATION_THRESHOLD {
        return Ok(());
    }
    let archive_name = format!("audit.{}.jsonl", Utc::now().format("%Y%m%d-%H%M%S"));
    let dest = config::audit_log_archive_dir().join(&archive_name);
    std::fs::rename(&path, &dest)
        .map_err(|e| AppError::Audit(format!("rotate: {}", e)))?;
    log::info!("rotated audit log to {:?}", dest);
    Ok(())
}
