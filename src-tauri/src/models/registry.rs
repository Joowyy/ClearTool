// models/registry.rs — DTOs de tweaks de registro.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTweak {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub hive: String,
    pub path: String,
    pub value_name: String,
    pub enabled_value: serde_json::Value,
    pub disabled_value: serde_json::Value,
    pub requires_reboot: bool,
    pub risk: String,
    pub consequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: String,
    pub is_enabled: Option<bool>,
    pub current_raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakInput {
    pub id: String,
    pub enable: bool,
    pub dry_run: bool,
}
