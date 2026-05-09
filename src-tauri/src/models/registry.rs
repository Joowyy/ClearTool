// models/registry.rs — DTOs de tweaks de registro.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTweak {
    pub id: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub hive: RegistryHive,
    pub path: String,
    pub value_name: String,
    pub value_kind: RegistryValueKind,
    pub on_value: serde_json::Value,
    pub off_value: serde_json::Value,
    pub requires_admin: bool,
    pub disclaimer: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegistryHive {
    Hkcu,
    Hklm,
    Hku,
    Hkcr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegistryValueKind {
    String,
    ExpandString,
    Dword,
    Qword,
    Binary,
    MultiString,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegistryTweakState {
    On,
    Off,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakResult {
    pub id: String,
    pub success: bool,
    pub previous_state: RegistryTweakState,
    pub new_state: RegistryTweakState,
    pub message: String,
}
