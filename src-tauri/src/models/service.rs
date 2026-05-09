// models/service.rs — DTOs de servicios de Windows.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub start_type: ServiceStartType,
    pub current_state: ServiceState,
    pub category: String,
    pub disclaimer: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceStartType {
    Boot,
    System,
    Auto,
    Manual,
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceState {
    Running,
    Stopped,
    StartPending,
    StopPending,
    Paused,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStateChange {
    pub name: String,
    pub new_start_type: Option<ServiceStartType>,
    pub stop_now: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServicesPreset {
    Minimum,
    Recommended,
    Total,
}
