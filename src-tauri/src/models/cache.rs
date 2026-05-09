// models/cache.rs — DTOs de cachés y reportes de limpieza.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheLocation {
    pub id: String,
    pub label: String,
    pub category: String,
    pub path: String,
    pub requires_admin: bool,
    pub estimated_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheScanResult {
    pub items: Vec<CacheScanItem>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheScanItem {
    pub id: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub items: Vec<CleanReportItem>,
    pub freed_bytes: u64,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReportItem {
    pub id: String,
    pub freed_bytes: u64,
    pub deleted_files: u64,
    pub error: Option<String>,
}
