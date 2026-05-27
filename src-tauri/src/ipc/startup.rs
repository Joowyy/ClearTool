// ipc/startup.rs — comandos Tauri para el Startup Manager.

use crate::core::AppResult;
use crate::domain;
use crate::models::startup::StartupEntry;

#[tauri::command]
pub async fn list_startup() -> AppResult<Vec<StartupEntry>> {
    domain::startup::list_all()
}

#[tauri::command]
pub async fn disable_startup(id: String) -> AppResult<()> {
    domain::startup::disable_startup(&id)
}

#[tauri::command]
pub async fn enable_startup(id: String) -> AppResult<()> {
    domain::startup::enable_startup(&id)
}
