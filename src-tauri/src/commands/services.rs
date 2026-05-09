// commands/services.rs — gestión de servicios de Windows.

use crate::error::{AppError, AppResult};
use crate::models::service::{ServiceInfo, ServiceStateChange, ServicesPreset};

#[tauri::command]
pub async fn list_services() -> AppResult<Vec<ServiceInfo>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn set_service_state(_change: ServiceStateChange, _dry_run: bool) -> AppResult<()> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn apply_service_preset(_preset: ServicesPreset, _dry_run: bool) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
