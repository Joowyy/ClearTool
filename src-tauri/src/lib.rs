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

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init());

    // decorum: titlebar HTML custom conservando Snap Layouts de Win11.
    // Solo en Windows — en Linux usamos decoraciones nativas del WM.
    #[cfg(target_os = "windows")]
    let builder = builder.plugin(tauri_plugin_decorum::init());

    builder
        .setup(|app| {
            // Inicializar preferencias persistentes.
            crate::core::settings::init();

            // Validación de catálogos: se realiza bajo demanda al primer uso
            // del módulo correspondiente (Debloat, Cache, etc.) en lugar de
            // aquí. Acelera el arranque; un JSON roto en embed falla en CI, no
            // en el primer paint del usuario.

            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                // Nota: NO llamamos a `create_overlay_titlebar()` porque la
                // X y demás botones los pinta nuestra titlebar HTML.
                // decorum sigue cargado para `show_snap_overlay` (Win+Z) si
                // queremos preservar Snap Layouts vía atajo de teclado.

                // En debug, abrir DevTools para ver errores del frontend.
                #[cfg(debug_assertions)]
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // system_info
            ipc::system_info::is_elevated,
            ipc::system_info::system_summary,
            ipc::system_info::relaunch_as_admin,
            ipc::system_info::app_version,
            // telemetry (monitorización en vivo)
            ipc::telemetry::get_telemetry_snapshot,
            // explorer
            ipc::explorer::scan_tree,
            ipc::explorer::list_dir,
            ipc::explorer::cancel_scan,
            ipc::explorer::compute_directory_size,
            // cache
            ipc::cache::list_cache_locations,
            ipc::cache::scan_cache_locations,
            ipc::cache::clean_cache_locations,
            ipc::cache::analyze_cache_locations,
            ipc::cache::execute_clean_plan,
            ipc::cache::verify_clean,
            // debloat
            ipc::debloat::list_bloatware_catalog,
            ipc::debloat::detect_installed_bloatware,
            ipc::debloat::remove_bloatware,
            // services
            ipc::services::list_services,
            ipc::services::set_service_state,
            ipc::services::service_dependencies,
            ipc::services::apply_service_preset,
            // registry
            ipc::registry::list_registry_tweaks,
            ipc::registry::read_registry_tweak_state,
            ipc::registry::apply_registry_tweak,
            ipc::registry::apply_registry_tweak_batch,
            ipc::registry::revert_registry_tweak,
            // restore
            ipc::restore::ensure_restore_enabled,
            ipc::restore::enable_system_protection,
            ipc::restore::create_restore_point,
            ipc::restore::list_restore_points,
            ipc::restore::restore_to_point,
            // audit
            ipc::audit::list_audit_log,
            ipc::audit::revert_audit_entry,
            ipc::audit::audit_log_path,
            // settings
            ipc::settings::get_settings,
            ipc::settings::update_settings,
            ipc::settings::reset_settings_to_defaults,
            ipc::settings::settings_file_path,
            ipc::settings::open_settings_file,
            // processes
            ipc::processes::list_processes,
            ipc::processes::kill_process,
            ipc::processes::kill_process_tree,
            ipc::processes::suspend_process,
            ipc::processes::resume_process,
            ipc::processes::close_gracefully,
            ipc::processes::who_locks_path,
            ipc::processes::release_caches,
            // startup
            ipc::startup::list_startup,
            ipc::startup::disable_startup,
            ipc::startup::enable_startup,
            // disk analyzer
            ipc::disk::build_treemap_data,
            // network utilities
            ipc::network::flush_dns,
            ipc::network::renew_ip,
            ipc::network::reset_winsock,
            ipc::network::reset_tcpip,
            ipc::network::reset_proxy,
            ipc::network::restore_hosts_file,
            // diagnostics
            ipc::diagnostics::export_diagnostic_zip,
            // boot-time cleanup (pending renames)
            ipc::boot_cleanup::list_pending_renames,
            ipc::boot_cleanup::cancel_pending_rename,
            ipc::boot_cleanup::clear_all_pending_renames,
            // privacy hardening
            ipc::privacy::get_privacy_preset_preview,
            ipc::privacy::apply_privacy_preset,
            // universal app inventory
            ipc::inventory::list_installed_apps,
            ipc::inventory::compute_residual_hints,
            ipc::inventory::uninstall_app,
            ipc::inventory::clean_residuals,
            ipc::inventory::uninstall_app_complete,
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecuta la aplicación");
}
