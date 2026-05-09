// Capa de comandos Tauri.
//
// Cada submódulo expone los `#[tauri::command]` de un dominio funcional.
// Convención: las funciones son `pub async fn`, devuelven
// `Result<T, AppError>` y aceptan `dry_run: bool` cuando son destructivas.

pub mod audit;
pub mod cache;
pub mod debloat;
pub mod explorer;
pub mod registry;
pub mod restore;
pub mod services;
pub mod system_info;
