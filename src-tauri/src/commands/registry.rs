// commands/registry.rs — tweaks del registro (HKLM/HKCU controlados).

use crate::error::{AppError, AppResult};
use crate::models::registry::{RegistryTweak, RegistryTweakState, TweakResult};

#[tauri::command]
pub async fn list_registry_tweaks() -> AppResult<Vec<RegistryTweak>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn read_registry_tweak_state(_id: String) -> AppResult<RegistryTweakState> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn apply_registry_tweak(_id: String, _dry_run: bool) -> AppResult<TweakResult> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn apply_registry_tweak_batch(
    _ids: Vec<String>,
    _dry_run: bool,
) -> AppResult<Vec<TweakResult>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn revert_registry_tweak(_id: String) -> AppResult<TweakResult> {
    Err(AppError::NotImplemented)
}
