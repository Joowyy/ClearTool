// ipc/disk.rs — comandos Tauri para Disk Analyzer.

use crate::core::{AppError, AppResult};
use crate::domain::disk;
use crate::models::disk::{BuildTreemapInput, DiskAnalysisResult, DriveListing};

#[tauri::command]
pub async fn list_drives() -> AppResult<Vec<DriveListing>> {
    // El walker es rápido (< 50 ms) pero abre handles Win32: dejarlo en
    // spawn_blocking igualmente protege el runtime de UAC freezes raros.
    tokio::task::spawn_blocking(disk::list_drives)
        .await
        .map_err(|e| AppError::External(format!("list_drives join error: {e}")))?
}

#[tauri::command]
pub async fn build_treemap_data(
    app: tauri::AppHandle,
    input: BuildTreemapInput,
) -> AppResult<DiskAnalysisResult> {
    if input.root.contains("..") {
        return Err(AppError::Permission("invalid root path".into()));
    }
    if input.root.trim().is_empty() {
        return Err(AppError::Validation("root cannot be empty".into()));
    }
    disk::run_analysis(app, input).await
}

#[tauri::command]
pub async fn cancel_disk_scan(scan_id: String) -> AppResult<()> {
    disk::cancel_scan(&scan_id);
    Ok(())
}
