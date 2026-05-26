// models/registry.rs — DTOs de tweaks de registro.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(alias = "tweaks")]
    pub entries: Vec<RegistryTweak>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryOp {
    pub hive: String,
    #[serde(alias = "path")]
    pub key: String,
    pub name: String,
    pub kind: String,
    pub enabled_value: serde_json::Value,
    pub disabled_value: serde_json::Value,
    #[serde(default = "default_true")]
    pub create_if_missing: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTweak {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub category: String,
    pub risk: String,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub operations: Vec<RegistryOp>,
    #[serde(default)]
    pub presets: Vec<String>,
    #[serde(default, alias = "minBuild")]
    pub min_windows_build: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: String,
    pub enabled: bool,
    pub partial: bool,
    pub current_values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakInput {
    pub id: String,
    pub enable: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakBatchInput {
    pub inputs: Vec<ApplyTweakInput>,
    pub create_restore_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakBatchReport {
    pub run_id: String,
    pub total: u32,
    pub applied: u32,
    pub failed: u32,
    pub skipped: u32,
    pub restore_point_seq: Option<u32>,
    pub per_tweak: Vec<PerTweakResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerTweakResult {
    pub id: String,
    pub status: String,
    pub error: Option<String>,
    pub previous_values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryProgressEvent {
    pub run_id: String,
    pub processed: u32,
    pub total: u32,
    pub current_id: String,
}
