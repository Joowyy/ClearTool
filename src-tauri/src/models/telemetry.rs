// models/telemetry.rs — DTOs de monitorización en vivo del sistema.
//
// Producidos por `domain::telemetry::snapshot()` y consumidos por el
// frontend cada ~1.5s para alimentar el Dashboard.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub index: u32,
    pub name: String,
    pub vendor: Option<String>,
    pub usage_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temp_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    pub cpu_total_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub top_processes: Vec<ProcessInfo>,
    pub gpus: Vec<GpuInfo>,
    pub timestamp: String,
}
