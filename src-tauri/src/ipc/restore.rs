// ipc/restore.rs — System Restore Points (vía SRSetRestorePointW).

use crate::core::AppResult;
use crate::domain;
use crate::models::restore::{CreateRestorePointInput, RestorePoint, RestoreReport};

#[tauri::command]
pub async fn ensure_restore_enabled() -> AppResult<bool> {
    domain::restore::ensure_enabled()
}

#[tauri::command]
pub async fn enable_system_protection() -> AppResult<()> {
    domain::restore::enable_for_system_drive()
}

#[tauri::command]
pub async fn create_restore_point(input: CreateRestorePointInput) -> AppResult<RestoreReport> {
    domain::restore::create(&input)
}

#[tauri::command]
pub async fn list_restore_points() -> AppResult<Vec<RestorePoint>> {
    domain::restore::list()
}

#[tauri::command]
pub async fn restore_to_point(sequence_number: u32) -> AppResult<()> {
    domain::restore::restore_to(sequence_number)
}
