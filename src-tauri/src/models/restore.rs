// models/restore.rs — DTOs de System Restore Points.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,
    pub creation_time: String, // ISO-8601; cuando metamos chrono será DateTime<Utc>.
    pub restore_point_type: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
    pub requires_reboot: bool,
}
