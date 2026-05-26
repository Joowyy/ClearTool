// ipc/settings.rs — preferencias persistentes.

use crate::core::{settings, AppResult};
use crate::models::settings::Settings;

#[tauri::command]
pub async fn get_settings() -> AppResult<Settings> {
    Ok(settings::get())
}

#[tauri::command]
pub async fn update_settings(updated: Settings) -> AppResult<()> {
    settings::update(updated)
}

#[tauri::command]
pub async fn reset_settings_to_defaults() -> AppResult<Settings> {
    settings::reset_to_defaults()?;
    Ok(settings::get())
}

#[tauri::command]
pub async fn settings_file_path() -> AppResult<String> {
    Ok(settings::settings_path().to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn open_settings_file() -> AppResult<()> {
    let p = settings::settings_path();
    if let Some(parent) = p.parent() {
        let _ = open::that(parent);
    }
    Ok(())
}
