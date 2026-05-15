// commands/cache.rs — gestión del catálogo de cachés y limpieza.

use crate::error::{AppError, AppResult};
use crate::models::cache::{CacheLocation, CacheScanReport, CleanCacheInput, CleanReport};

#[tauri::command]
pub async fn list_cache_locations() -> AppResult<Vec<CacheLocation>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn scan_cache_locations(_ids: Vec<String>) -> AppResult<Vec<CacheScanReport>> {
    Err(AppError::NotImplemented)
}

#[tauri::command]
pub async fn clean_cache_locations(_input: CleanCacheInput) -> AppResult<CleanReport> {
    Err(AppError::NotImplemented)
}
