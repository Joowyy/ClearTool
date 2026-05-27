// ipc/diagnostics.rs — exportar reporte de diagnóstico como ZIP.

use crate::core::{AppError, AppResult};
use tauri::Manager;

fn zip_err(e: zip::result::ZipError) -> AppError {
    AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

#[tauri::command]
pub async fn export_diagnostic_zip(app: tauri::AppHandle) -> AppResult<String> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let dir = app.path().download_dir()
        .map_err(|e| AppError::Validation(format!("{}", e)))?;
    let zip_path = dir.join(format!("cleartool-diag-{}.zip", chrono::Utc::now().format("%Y%m%d-%H%M%S")));

    let file = std::fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. system_summary.json
    if let Ok(s) = crate::domain::system_info::summary() {
        zip.start_file("system_summary.json", opts).map_err(zip_err)?;
        zip.write_all(serde_json::to_string_pretty(&s)?.as_bytes())?;
    }

    // 2. settings actuales
    let settings = crate::core::settings::get();
    zip.start_file("settings.json", opts).map_err(zip_err)?;
    zip.write_all(serde_json::to_string_pretty(&settings)?.as_bytes())?;

    // 3. audit log (últimas 200 entries)
    if let Ok(audit) = crate::domain::audit::list_log() {
        let recent: Vec<_> = audit.iter().rev().take(200).collect();
        zip.start_file("audit-recent.json", opts).map_err(zip_err)?;
        zip.write_all(serde_json::to_string_pretty(&recent)?.as_bytes())?;
    }

    // 4. catálogos cargados
    if let Ok(c) = crate::domain::catalog::load_cache_locations() {
        zip.start_file("cache-catalog.json", opts).map_err(zip_err)?;
        zip.write_all(serde_json::to_string_pretty(&c)?.as_bytes())?;
    }

    // 5. info de versión
    zip.start_file("version.txt", opts).map_err(zip_err)?;
    let build_date = option_env!("BUILD_DATE").unwrap_or("unknown");
    zip.write_all(format!("ClearTool v{}\nBuilt: {}\n", env!("CARGO_PKG_VERSION"), build_date).as_bytes())?;

    zip.finish().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    Ok(zip_path.display().to_string())
}
