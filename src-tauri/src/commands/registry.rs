// commands/registry.rs — tweaks del registro (HKLM/HKCU controlados).

use crate::error::{AppError, AppResult};
use crate::models::registry::{ApplyTweakInput, RegistryTweak, TweakState};

#[tauri::command]
pub async fn list_registry_tweaks() -> AppResult<Vec<RegistryTweak>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn read_registry_tweak_state(_id: String) -> AppResult<TweakState> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn apply_registry_tweak(_input: ApplyTweakInput) -> AppResult<()> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn apply_registry_tweak_batch(_inputs: Vec<ApplyTweakInput>) -> AppResult<()> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn revert_registry_tweak(_id: String) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
