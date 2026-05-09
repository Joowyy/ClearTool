// commands/restore.rs — System Restore Points (vía SRSetRestorePointW).

use crate::error::{AppError, AppResult};
use crate::models::restore::{RestorePoint, RestoreResult};

#[tauri::command]
pub async fn ensure_restore_enabled() -> AppResult<bool> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn create_restore_point(_description: String) -> AppResult<RestorePoint> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn list_restore_points() -> AppResult<Vec<RestorePoint>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn restore_to_point(_sequence_number: u32) -> AppResult<RestoreResult> {
    Err(AppError::NotImplemented)
}
