// models/settings.rs — preferencias persistentes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings_version: u32,
    pub appearance: AppearanceSettings,
    pub safety: SafetySettings,
    pub behavior: BehaviorSettings,
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,
    pub language: String,
    pub density: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySettings {
    pub dry_run_global: bool,
    pub auto_create_restore_point: bool,
    pub require_confirm_before_batch: bool,
    pub bypass_throttling_restore: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorSettings {
    pub check_updates_on_start: bool,
    pub last_seen_audit_run_id: Option<String>,
    pub remember_window_size: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSettings {
    pub log_level: String,
    pub audit_log_max_mb: u32,
    pub diagnostic_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: 1,
            appearance: AppearanceSettings {
                theme: "dark".into(),
                language: "system".into(),
                density: "normal".into(),
            },
            safety: SafetySettings {
                dry_run_global: false,
                auto_create_restore_point: true,
                require_confirm_before_batch: true,
                bypass_throttling_restore: true,
            },
            behavior: BehaviorSettings {
                check_updates_on_start: true,
                last_seen_audit_run_id: None,
                remember_window_size: true,
            },
            advanced: AdvancedSettings {
                log_level: "info".into(),
                audit_log_max_mb: 10,
                diagnostic_mode: false,
            },
        }
    }
}
