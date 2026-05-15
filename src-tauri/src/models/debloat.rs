// models/debloat.rs — DTOs del catálogo de bloatware y resultados.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareEntry {
    pub id: String,
    pub display_name: String,
    pub category: String,
    pub removal_strategy: String,
    pub package_names: Vec<String>,
    pub services: Vec<String>,
    pub scheduled_tasks: Vec<String>,
    pub risk: String,
    pub consequences: Vec<String>,
    pub reversal_method: String,
    pub reversal_details: String,
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
