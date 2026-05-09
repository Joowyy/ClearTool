// commands/audit.rs — log de auditoría y reversa fina por entrada.

use crate::error::{AppError, AppResult};

// Importamos los modelos cuando existan (audit log vive en services/audit_log.rs).
// Por ahora, devolvemos JSON crudo en `list_audit_log` para no fijar la forma
// del DTO antes de implementar el writer/reader.

#[tauri::command]
pub async fn list_audit_log(_limit: Option<usize>) -> AppResult<Vec<serde_json::Value>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn revert_audit_entry(_entry_id: String, _dry_run: bool) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
