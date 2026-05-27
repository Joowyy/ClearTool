// ipc/cache.rs — gestión del catálogo de cachés y limpieza.

use crate::core::AppResult;
use crate::domain;
use crate::models::cache::{
    CacheLocation, CacheScanReport, CleanCacheInput, CleanPlan, CleanReport, CleanReportV2,
    ExecutePlanOpts, VerifyReport,
};
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

// ── v2: analyze, execute_plan, verify ──

#[tauri::command]
pub async fn analyze_cache_locations(ids: Vec<String>) -> AppResult<CleanPlan> {
    domain::cache::analyze_locations(&ids)
}

#[tauri::command]
pub async fn execute_clean_plan(
    app: tauri::AppHandle,
    plan: CleanPlan,
    opts: ExecutePlanOpts,
) -> AppResult<CleanReportV2> {
    let emit = |level: &str, location: &str, message: &str, bytes_freed: u64, files_deleted: u64| {
        let _ = app.emit(
            "cache:progress",
            CacheDebugEvent {
                level: level.to_string(),
                location: location.to_string(),
                message: message.to_string(),
                bytes_freed,
                files_deleted,
            },
        );
    };
    domain::cache::execute_plan(plan, opts, emit).await
}

#[tauri::command]
pub async fn verify_clean(plan: CleanPlan, report: CleanReportV2) -> AppResult<VerifyReport> {
    domain::cache::verify_after_clean(&plan, &report)
}

#[tauri::command]
pub async fn get_cache_plan_warm() -> AppResult<Option<CleanPlan>> {
    Ok(crate::domain::cache_background::get_cached().map(|c| c.plan))
}

#[tauri::command]
pub async fn refresh_cache_plan() -> AppResult<CleanPlan> {
    crate::domain::cache_background::recompute_and_store().await
}
