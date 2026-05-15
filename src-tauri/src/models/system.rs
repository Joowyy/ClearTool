// models/system.rs — info general del sistema.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub letter: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSummary {
    pub os_name: String,
    pub os_version: String,
    pub build_number: String,
    pub username: String,
    pub is_elevated: bool,
    pub total_ram_bytes: u64,
    pub drives: Vec<DriveInfo>,
}
