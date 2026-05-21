// models/debloat.rs — DTOs del catálogo de bloatware y resultados.

use serde::{Deserialize, Serialize};

/// Envelope del JSON `bloatware-catalog.json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareCatalogFile {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub updated_at: String,
    pub entries: Vec<BloatwareEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareEntry {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub removal_strategy: Option<String>,
    #[serde(default)]
    pub package_names: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub scheduled_tasks: Vec<String>,
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub consequences: Vec<String>,
    #[serde(default)]
    pub reversal_method: Option<String>,
    #[serde(default)]
    pub reversal_details: Option<String>,
    #[serde(default)]
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPackage {
    pub entry_id: String,
    pub installed_for_user: bool,
    pub installed_all_users: bool,
    pub provisioned: bool,
    pub package_full_name: Option<String>,
    pub install_location: Option<String>,
    pub size_estimate_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveBloatwareInput {
    pub entry_ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub apply_policies: bool,
    pub disable_services: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepLog {
    pub kind: String,
    pub target: String,
    pub ok: bool,
    pub stderr: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerEntryResult {
    pub entry_id: String,
    pub status: String,
    pub steps: Vec<StepLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveReport {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub restore_point_id: Option<u32>,
    pub per_entry: Vec<PerEntryResult>,
    pub total_removed: u64,
    pub total_skipped: u64,
    pub total_failed: u64,
}
