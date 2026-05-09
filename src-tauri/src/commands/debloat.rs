// commands/debloat.rs — eliminación de bloatware (Appx + uninstallers).

use crate::error::{AppError, AppResult};
use crate::models::debloat::{BloatwareEntry, DebloatReport, InstalledBloatware};

#[tauri::command]
pub async fn list_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn detect_installed_bloatware() -> AppResult<Vec<InstalledBloatware>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn remove_bloatware(_ids: Vec<String>, _dry_run: bool) -> AppResult<DebloatReport> {
    Err(AppError::NotImplemented)
}
