// ClearTool — bootstrap del backend Tauri.
//
// Topología (lee de arriba a abajo, no hay ciclos):
//
//   core      → error, config, util         (no depende de nada)
//   platform  → Win32 / winreg / PowerShell (depende de core)
//   domain    → reglas de negocio           (depende de core + platform + models)
//   models    → DTOs serializables          (depende de serde solo)
//   ipc       → #[tauri::command]           (depende de domain + models + core)
//
// El frontend NUNCA habla con platform o domain directamente — sólo con ipc.

pub mod core;
pub mod domain;
pub mod ipc;
pub mod models;
pub mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // En release, el manifest es `requireAdministrator` (Windows pide UAC al
    // arrancar) y nunca llegamos aquí sin elevación. En debug usamos
    // `asInvoker` para evitar el error 740 de `cargo run`; ahí NO relanzamos
    // con UAC porque crearía un proceso descoordinado con `tauri dev`.
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        if !platform::elevation::is_elevated() {
            if platform::elevation::try_relaunch_as_admin() {
                std::process::exit(0);
            }
            // Usuario canceló UAC → app sigue en modo lectura
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Validar catálogos embebidos al arranque — detecta JSON corrupto
            // en debug en vez de en el primer click de la UI.
            domain::catalog::validate_all()
                .expect("catálogos embebidos inválidos");

            // En debug, abrir DevTools automáticamente para ver errores
            // del frontend sin tener que hacer click derecho.
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // system_info
            ipc::system_info::is_elevated,
            ipc::system_info::system_summary,
            ipc::system_info::relaunch_as_admin,
            // telemetry (monitorización en vivo)
            ipc::telemetry::get_telemetry_snapshot,
            // explorer
            ipc::explorer::scan_tree,
            ipc::explorer::cancel_scan,
            ipc::explorer::compute_directory_size,
            // cache
            ipc::cache::list_cache_locations,
            ipc::cache::scan_cache_locations,
            ipc::cache::clean_cache_locations,
            // debloat
            ipc::debloat::list_bloatware_catalog,
            ipc::debloat::detect_installed_bloatware,
            ipc::debloat::remove_bloatware,
            // services
            ipc::services::list_services,
            ipc::services::set_service_state,
            ipc::services::apply_service_preset,
            // registry
            ipc::registry::list_registry_tweaks,
            ipc::registry::read_registry_tweak_state,
            ipc::registry::apply_registry_tweak,
            ipc::registry::apply_registry_tweak_batch,
            ipc::registry::revert_registry_tweak,
            // restore
            ipc::restore::ensure_restore_enabled,
            ipc::restore::create_restore_point,
            ipc::restore::list_restore_points,
            ipc::restore::restore_to_point,
            // audit
            ipc::audit::list_audit_log,
            ipc::audit::revert_audit_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecuta la aplicación");
}
