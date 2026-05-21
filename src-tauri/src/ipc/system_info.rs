// ipc/system_info.rs — comandos de info general del sistema.

use crate::core::AppResult;
use crate::domain;
use crate::models::system::SystemSummary;
use crate::platform;

#[tauri::command]
pub async fn is_elevated() -> bool {
    domain::system_info::is_elevated()
}

#[tauri::command]
pub async fn system_summary() -> AppResult<SystemSummary> {
    domain::system_info::summary()
}

/// Intenta relanzar la app como administrador. Retorna `true` si se
/// relanzó correctamente (el proceso actual debe salir inmediatamente).
#[tauri::command]
pub async fn relaunch_as_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        platform::elevation::try_relaunch_as_admin()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
