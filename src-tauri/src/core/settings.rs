// core/settings.rs — preferencias persistentes en JSON.

use crate::core::{AppError, AppResult, config};
use crate::models::settings::Settings;
use std::sync::RwLock;
use std::sync::OnceLock;

static SETTINGS: OnceLock<RwLock<Settings>> = OnceLock::new();

pub fn settings_path() -> std::path::PathBuf {
    config::settings_path()
}

pub fn load_or_default() -> Settings {
    let path = settings_path();
    if !path.exists() {
        let def = Settings::default();
        let _ = write_atomic(&def);
        return def;
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str::<Settings>(&s) {
            Ok(parsed) => {
                if parsed.settings_version != 1 {
                    log::warn!(
                        "settings versión desconocida: {}, usando defaults",
                        parsed.settings_version
                    );
                    let def = Settings::default();
                    let _ = write_atomic(&def);
                    def
                } else {
                    parsed
                }
            }
            Err(e) => {
                log::warn!("settings corruptos: {}, restaurando defaults", e);
                let bak = path.with_extension("json.bak");
                let _ = std::fs::rename(&path, &bak);
                let def = Settings::default();
                let _ = write_atomic(&def);
                def
            }
        },
        Err(_) => Settings::default(),
    }
}

pub fn init() {
    let loaded = load_or_default();
    let _ = SETTINGS.set(RwLock::new(loaded));
}

pub fn get() -> Settings {
    SETTINGS
        .get()
        .expect("settings::init no llamado")
        .read()
        .map(|s| s.clone())
        .unwrap_or_default()
}

pub fn update(new_settings: Settings) -> AppResult<()> {
    write_atomic(&new_settings)?;
    if let Some(lock) = SETTINGS.get() {
        *lock
            .write()
            .map_err(|_| AppError::Validation("settings lock poisoned".into()))? = new_settings;
    }
    Ok(())
}

fn write_atomic(s: &Settings) -> AppResult<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("json.tmp");
    let payload = serde_json::to_string_pretty(s)
        .map_err(|e| AppError::Audit(format!("serialize settings: {}", e)))?;
    std::fs::write(&tmp, payload)
        .map_err(|e| AppError::Audit(format!("write tmp: {}", e)))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| AppError::Audit(format!("rename atomic: {}", e)))?;
    Ok(())
}

pub fn reset_to_defaults() -> AppResult<()> {
    update(Settings::default())
}

pub fn is_dry_run_global() -> bool {
    get().safety.dry_run_global
}
