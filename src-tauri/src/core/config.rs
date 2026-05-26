// core/config.rs — paths estándar y constantes globales de la aplicación.

use std::path::PathBuf;

pub const APP_NAME: &str = "ClearTool";

pub fn app_data_dir() -> std::io::Result<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| dirs_home_fallback())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "no se pudo resolver %APPDATA%",
                )
            })?
    } else {
        dirs_home_fallback().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no se pudo resolver $HOME")
        })?
    };

    let dir = base.join(APP_NAME);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn audit_log_path() -> PathBuf {
    let dir = app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    dir.join("audit.jsonl")
}

pub fn audit_log_archive_dir() -> PathBuf {
    let base = app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dir = base.join("audit-archive");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn operational_log_path() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let dir = local.join(APP_NAME).join("logs");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("app.log")
}

pub fn backups_dir() -> std::io::Result<PathBuf> {
    let dir = app_data_dir()?.join("backups");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn settings_path() -> PathBuf {
    let dir = app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    dir.join("settings.json")
}

fn dirs_home_fallback() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}
