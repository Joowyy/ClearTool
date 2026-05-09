// commands/system_info.rs — información general del sistema y elevación.

use crate::elevation;
use crate::error::{AppError, AppResult};
use crate::models::system::SystemSummary;

#[tauri::command]
pub async fn is_elevated() -> bool {
    elevation::is_elevated()
}

#[tauri::command]
pub async fn system_summary() -> AppResult<SystemSummary> {
    Err(AppError::NotImplemented)
}
