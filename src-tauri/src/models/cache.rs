// models/cache.rs — DTOs de cachés y reportes de limpieza.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    pub entries: Vec<CacheLocation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheFilters {
    #[serde(default)]
    pub older_than_days: Option<u64>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheLocation {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub path: String,
    pub category: String,
    #[serde(default)]
    pub requires_admin: bool,
    pub risk: String,
    #[serde(default)]
    pub consequences: Vec<String>,
    #[serde(default)]
    pub average_size: Option<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub filters: CacheFilters,
    #[serde(default)]
    pub min_windows_build: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheScanReport {
    pub id: String,
    pub resolved_path: String,
    pub exists: bool,
    pub bytes: u64,
    pub file_count: u64,
    pub matched_after_filters: u64,
    pub bytes_after_filters: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheInput {
    pub ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub force_close_processes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerLocationResult {
    pub id: String,
    pub status: String,
    pub bytes_freed: u64,
    pub files_deleted: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub restore_point_id: Option<u32>,
    pub per_location: Vec<PerLocationResult>,
    pub total_bytes_freed: u64,
    pub total_files_deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgressEvent {
    pub run_id: String,
    pub level: String,
    pub location_display: String,
    pub message: String,
    pub bytes_delta: u64,
    pub files_delta: u64,
}
