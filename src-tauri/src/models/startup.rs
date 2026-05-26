// models/startup.rs — DTOs de auto-arranque.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    pub id: String,
    pub origin: StartupOrigin,
    pub display_name: String,
    pub command: String,
    pub exe_path: Option<String>,
    pub icon_path: Option<String>,
    pub publisher: Option<String>,
    pub signature_valid: Option<bool>,
    pub impact: StartupImpact,
    pub last_modified: Option<String>,
    pub enabled: bool,
    pub category: StartupCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StartupOrigin {
    Registry { hive: String, key: String, name: String },
    StartupFolder { lnk_path: String },
    ScheduledTask { task_path: String },
    Service { service_name: String },
    UwpAutoStart { package_family_name: String, task_id: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum StartupImpact {
    Unknown,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum StartupCategory {
    Updater,
    Launcher,
    Widget,
    CloudSync,
    Communication,
    Media,
    Security,
    Driver,
    UserApp,
    System,
    Unknown,
}
