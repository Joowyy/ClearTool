// commands/services.rs — gestión de servicios de Windows.

use crate::error::{AppError, AppResult};
use crate::models::service::Service;

#[tauri::command]
pub async fn list_services() -> AppResult<Vec<Service>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn set_service_state(
    _name: String,
    _start_type: String,
    _dry_run: bool,
) -> AppResult<()> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn apply_service_preset(_preset: String, _dry_run: bool) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
