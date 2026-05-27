// ipc/system_info.rs — comandos de info general del sistema.

use crate::core::AppResult;
use crate::domain;
use crate::models::system::SystemSummary;
use crate::platform;
use serde::Serialize;

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppVersionInfo {
    pub version: String,
    pub build_date: String,
    pub git_commit: Option<String>,
}

#[tauri::command]
pub async fn app_version() -> AppResult<AppVersionInfo> {
    Ok(AppVersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: option_env!("BUILD_DATE").unwrap_or("unknown").to_string(),
        git_commit: option_env!("GIT_COMMIT").map(String::from),
    })
}
