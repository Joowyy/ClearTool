// commands/explorer.rs — exploración del árbol de directorios.

use crate::error::{AppError, AppResult};
use crate::models::tree::{ScanOptions, TreeNode};

#[tauri::command]
pub async fn scan_tree(_path: String, _options: Option<ScanOptions>) -> AppResult<TreeNode> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn cancel_scan(_scan_id: String) -> AppResult<()> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn compute_directory_size(_path: String) -> AppResult<u64> {
    Err(AppError::NotImplemented)
}
