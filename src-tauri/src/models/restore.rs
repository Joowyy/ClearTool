// models/restore.rs — DTOs de System Restore Points y audit log.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,
    pub creation_time: String,
    pub restore_point_type: u32,
    pub event_type: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRestorePointInput {
    pub description: String,
    pub restore_type: Option<u32>,
    #[serde(default = "default_bypass")]
    pub bypass_throttle: bool,
}

fn default_bypass() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub sequence_number: u32,
    pub created_at: String,
    pub description: String,
    pub bypassed_throttle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub run_id: String,
    pub timestamp: String,
    pub module: String,
    pub operation: String,
    pub dry_run: bool,
    pub restore_point_seq: Option<u32>,
    pub items_affected: Vec<String>,
    pub reverse_recipe: ReverseRecipe,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReverseRecipe {
    Registry {
        operations: Vec<RegistryRevertOp>,
    },
    Service {
        service_name: String,
        previous_start_type: String,
        previous_state: String,
    },
    AppxReinstall {
        package_family_name: String,
        store_url: Option<String>,
    },
    StartupToggle {
        origin: crate::models::startup::StartupOrigin,
        previous_enabled: bool,
    },
    Composite {
        recipes: Vec<ReverseRecipe>,
    },
    Noop {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRevertOp {
    pub hive: String,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub previous_value: Option<serde_json::Value>,
}
