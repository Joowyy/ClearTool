// models/restore.rs — DTOs de System Restore Points y audit log.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,
    pub restore_point_type: u32,
    pub event_type: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRestorePointInput {
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub run_id: String,
    pub sequence_number: u32,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub run_id: String,
    pub timestamp: String,
    pub module: String,
    pub operation: String,
    pub restore_point_id: Option<u32>,
    pub items_count: u64,
    pub success: bool,
    pub reverse_recipe: Option<String>,
}
