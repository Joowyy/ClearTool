// models/service.rs — DTOs de servicios de Windows.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicesCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(alias = "services")]
    pub entries: Vec<ServiceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEntry {
    #[serde(alias = "name")]
    pub service_name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub category: String,
    #[serde(alias = "riskIfDisabled")]
    pub risk: String,
    #[serde(default, alias = "recommendedAction")]
    pub default_start_type: String,
    #[serde(default)]
    pub recommended_start_type: Option<String>,
    #[serde(default)]
    pub consequences: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub needed_by: Vec<String>,
    #[serde(default)]
    pub min_windows_build: Option<u32>,
    #[serde(default)]
    pub presets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_type: String,
    pub description: Option<String>,
    pub in_catalog: bool,
    pub catalog: Option<ServiceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetServiceStateInput {
    pub name: String,
    pub start_type: String,
    pub stop_now: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyServicePresetInput {
    pub preset: String,
    pub stop_now: bool,
    pub dry_run: bool,
    pub create_restore_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPresetReport {
    pub run_id: String,
    pub preset: String,
    pub total: u32,
    pub applied: u32,
    pub failed: u32,
    pub protected: u32,
    pub skipped: u32,
    pub restore_point_seq: Option<u32>,
    pub per_service: Vec<PerServiceResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerServiceResult {
    pub name: String,
    pub status: String,
    pub previous_start_type: String,
    pub previous_state: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProgressEvent {
    pub run_id: String,
    pub processed: u32,
    pub total: u32,
    pub current_name: String,
}
