// models/debloat.rs — DTOs del catálogo de bloatware y resultados.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareEntry {
    pub id: String,
    pub label: String,
    pub kind: BloatwareKind,
    pub appx_pattern: Option<String>,
    pub uninstaller_id: Option<String>,
    pub category: String,
    pub disclaimer: Option<String>,
    pub default_in_total: bool,
    pub default_in_recommended: bool,
    pub default_in_minimum: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BloatwareKind {
    AppxPackage,
    AppxProvisioned,
    Uninstaller,
    Service,
    Feature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledBloatware {
    pub id: String,
    pub installed_for_current_user: bool,
    pub installed_provisioned: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebloatReport {
    pub items: Vec<DebloatReportItem>,
    pub dry_run: bool,
    pub restore_point_sequence: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebloatReportItem {
    pub id: String,
    pub success: bool,
    pub message: String,
}
