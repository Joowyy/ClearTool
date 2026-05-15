// commands/explorer.rs — exploración del árbol de directorios.

use crate::error::{AppError, AppResult};
use crate::models::tree::{DirectorySize, ScanTreeHandle, ScanTreeInput};

#[tauri::command]
pub async fn scan_tree(_input: ScanTreeInput) -> AppResult<ScanTreeHandle> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn cancel_scan(_handle: ScanTreeHandle) -> AppResult<()> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn compute_directory_size(
    _path: String,
    _follow_reparse_pints: bool,
) -> AppResult<DirectorySize> {
    Err(AppError::NotImplemented)
}
