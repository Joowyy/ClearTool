// models/debloat.rs — DTOs del catálogo de bloatware y resultados.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    pub entries: Vec<BloatwareEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareReverse {
    pub kind: String,
    pub store_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareEntry {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub category: String,
    pub risk: String,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub removal_method: String,
    #[serde(default)]
    pub appx_package_family_name: Option<String>,
    #[serde(default)]
    pub appx_provisioned_name: Option<String>,
    #[serde(default)]
    pub winget_id: Option<String>,
    #[serde(default)]
    pub uninstall_registry_path: Option<String>,
    #[serde(default = "default_preserves")]
    pub preserves_data_by_default: bool,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub reversible: bool,
    #[serde(default)]
    pub reverse_recipe: Option<BloatwareReverse>,
    #[serde(default)]
    pub min_windows_build: Option<u32>,
    #[serde(default)]
    pub presets: Vec<String>,
}

fn default_preserves() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPackage {
    pub id: String,
    pub display_name: String,
    pub installed_for_user: bool,
    pub installed_provisioned: bool,
    pub size_estimate_mb: Option<u32>,
    pub install_location: Option<String>,
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
pub struct RemoveReport {
    pub run_id: String,
    pub total: u32,
    pub removed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub restore_point_seq: Option<u32>,
    pub per_entry: Vec<PerEntryResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerEntryResult {
    pub id: String,
    pub status: String,
    pub method_used: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebloatProgressEvent {
    pub run_id: String,
    pub processed: u32,
    pub total: u32,
    pub current_id: String,
    pub current_display: String,
}
