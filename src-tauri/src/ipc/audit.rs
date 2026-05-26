// ipc/audit.rs — log de auditoría y reversa fina por entrada.

use crate::core::{AppResult, config};
use crate::domain;
use crate::models::restore::AuditEntry;

#[tauri::command]
pub async fn list_audit_log() -> AppResult<Vec<AuditEntry>> {
    domain::audit::list_log()
}

#[tauri::command]
pub async fn revert_audit_entry(run_id: String) -> AppResult<()> {
    domain::audit::revert_entry(&run_id)
}

#[tauri::command]
pub async fn audit_log_path() -> AppResult<String> {
    Ok(config::audit_log_path().to_string_lossy().into_owned())
}
