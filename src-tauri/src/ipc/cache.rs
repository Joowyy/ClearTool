// ipc/cache.rs — gestión del catálogo de cachés y limpieza.

use crate::core::AppResult;
use crate::domain;
use crate::models::cache::{CacheLocation, CacheScanReport, CleanCacheInput, CleanReport};
use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheDebugEvent {
    pub level: String,
    pub location: String,
    pub message: String,
    pub bytes_freed: u64,
    pub files_deleted: u64,
}

#[tauri::command]
pub async fn list_cache_locations() -> AppResult<Vec<CacheLocation>> {
    domain::cache::list_locations()
}

#[tauri::command]
pub async fn scan_cache_locations(ids: Vec<String>) -> AppResult<Vec<CacheScanReport>> {
    domain::cache::scan(&ids)
}

#[tauri::command]
pub async fn clean_cache_locations(
    app: tauri::AppHandle,
    input: CleanCacheInput,
) -> AppResult<CleanReport> {
    let emit = |level: &str, location: &str, message: &str, bytes_freed: u64, files_deleted: u64| {
        let _ = app.emit("cache:debug", CacheDebugEvent {
            level: level.to_string(),
            location: location.to_string(),
            message: message.to_string(),
            bytes_freed,
            files_deleted,
        });
    };
    domain::cache::clean(&input, emit)
}
