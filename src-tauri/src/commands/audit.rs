// commands/audit.rs — log de auditoría y reversa fina por entrada.

use crate::error::{AppError, AppResult};
use crate::models::restore::AuditEntry;

#[tauri::command]
pub async fn list_audit_log() -> AppResult<Vec<AuditEntry>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn revert_audit_entry(_run_id: String) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
