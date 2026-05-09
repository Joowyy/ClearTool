// commands/cache.rs — gestión del catálogo de cachés y limpieza.

use crate::error::{AppError, AppResult};
use crate::models::cache::{CacheLocation, CacheScanResult, CleanReport};

#[tauri::command]
pub async fn list_cache_locations() -> AppResult<Vec<CacheLocation>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn scan_cache_locations(_ids: Vec<String>) -> AppResult<CacheScanResult> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn clean_cache_locations(_ids: Vec<String>, _dry_run: bool) -> AppResult<CleanReport> {
    Err(AppError::NotImplemented)
}
