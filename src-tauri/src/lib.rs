// ClearTool — punto de entrada de la librería.
//
// Estructura por capas:
//   - commands/  : comandos Tauri expuestos al frontend (capa de IPC).
//   - services/  : lógica de negocio (FS, registro, PowerShell, audit, etc.).
//   - models/    : DTOs y tipos compartidos (serializables).
//   - error.rs   : tipo de error unificado AppError.
//   - elevation.rs: helpers de elevación de privilegios.

pub mod commands;
pub mod error;
pub mod elevation;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // system_info
            commands::system_info::is_elevated,
            commands::system_info::system_summary,
            // explorer
            commands::explorer::scan_tree,
            commands::explorer::cancel_scan,
            commands::explorer::compute_directory_size,
            // cache
            commands::cache::list_cache_locations,
            commands::cache::scan_cache_locations,
            commands::cache::clean_cache_locations,
            // debloat
            commands::debloat::list_bloatware_catalog,
            commands::debloat::detect_installed_bloatware,
            commands::debloat::remove_bloatware,
            // services
            commands::services::list_services,
            commands::services::set_service_state,
            commands::services::apply_service_preset,
            // registry
            commands::registry::list_registry_tweaks,
            commands::registry::read_registry_tweak_state,
            commands::registry::apply_registry_tweak,
            commands::registry::apply_registry_tweak_batch,
            commands::registry::revert_registry_tweak,
            // restore
            commands::restore::ensure_restore_enabled,
            commands::restore::create_restore_point,
            commands::restore::list_restore_points,
            commands::restore::restore_to_point,
            // audit
            commands::audit::list_audit_log,
            commands::audit::revert_audit_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecuta la aplicación");
}
