// models/system.rs — info general del sistema.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSummary {
    pub os_name: String,
    pub os_version: String,
    pub build_number: String,
    pub username: String,
    pub is_elevated: bool,
    pub total_ram_bytes: u64,
    pub free_disk_bytes: u64,
}
