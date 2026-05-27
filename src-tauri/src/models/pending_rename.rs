// models/pending_rename.rs — DTO para operaciones de rename pendientes en reboot.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingRename {
    pub source: String,
    pub destination: String,
    pub is_delete: bool,
}
