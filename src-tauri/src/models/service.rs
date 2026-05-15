// models/service.rs — DTOs de servicios de Windows.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub state: String,
    pub start_type: String,
    pub pid: Option<u32>,
    pub can_stop: bool,
    pub can_pause: bool,
}
