// commands/debloat.rs — eliminación de bloatware (Appx + uninstallers).

use crate::error::{AppError, AppResult};
use crate::models::debloat::{BloatwareEntry, DetectedPackage, RemoveBloatwareInput, RemoveReport};

#[tauri::command]
pub async fn list_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn detect_installed_bloatware() -> AppResult<Vec<DetectedPackage>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn remove_bloatware(_input: RemoveBloatwareInput) -> AppResult<RemoveReport> {
    Err(AppError::NotImplemented)
}
