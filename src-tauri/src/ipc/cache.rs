// ipc/cache.rs — gestión del catálogo de cachés y limpieza.

use crate::core::AppResult;
use crate::domain;
use crate::domain::cache::CleanEmitter;
use crate::domain::throughput_stats::ThroughputStats;
use crate::models::cache::{
    CacheLocation, CacheScanReport, CleanCacheInput, CleanLogLine, CleanPhaseEvent, CleanPlan,
    CleanProgressPayload, CleanReport, CleanReportV2, CleanSummaryPayload, ExecutePlanOpts,
    VerifyReport,
};
use serde::Serialize;
use tauri::Emitter;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheDebugEvent {
    pub level: String,
    pub location: String,
    pub message: String,
    pub bytes_freed: u64,
    pub files_deleted: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStartedEvent {
    pub run_id: String,
}

// ── TauriCleanEmitter ────────────────────────────────────────────────────

pub struct TauriCleanEmitter {
    app: tauri::AppHandle,
}

impl CleanEmitter for TauriCleanEmitter {
    fn line(&mut self, line: CleanLogLine) {
        let _ = self.app.emit("cache:line", &line);
        // Legacy compatibility.
        let _ = self.app.emit(
            "cache:debug",
            CacheDebugEvent {
                level: line.level.clone(),
                location: line.location.clone(),
                message: line.message.clone(),
                bytes_freed: 0,
                files_deleted: 0,
            },
        );
    }

    fn progress(&mut self, payload: CleanProgressPayload) {
        let _ = self.app.emit("cache:progress-v2", &payload);
        // Legacy.
        let _ = self.app.emit(
            "cache:progress",
            CacheDebugEvent {
                level: "info".into(),
                location: payload.current_location_display_name.unwrap_or_default(),
                message: format!(
                    "{} archivos · {} bytes",
                    payload.files_deleted, payload.bytes_freed
                ),
                bytes_freed: payload.bytes_freed,
                files_deleted: payload.files_deleted as u64,
            },
        );
    }

    fn phase(&mut self, event: CleanPhaseEvent) {
        let _ = self.app.emit("cache:phase", &event);
    }

    fn summary(&mut self, payload: CleanSummaryPayload) {
        let _ = self.app.emit("cache:summary", &payload);
    }
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
    let emit =
        |level: &str, location: &str, message: &str, bytes_freed: u64, files_deleted: u64| {
            let _ = app.emit(
                "cache:debug",
                CacheDebugEvent {
                    level: level.to_string(),
                    location: location.to_string(),
                    message: message.to_string(),
                    bytes_freed,
                    files_deleted,
                },
            );
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
    let run_id = Uuid::new_v4().to_string();
    let cancel = crate::domain::cache_cancellation::register(&run_id);

    // Notificar al frontend antes de arrancar.
    let _ = app.emit("cache:started", CacheStartedEvent { run_id: run_id.clone() });

    let mut emitter = TauriCleanEmitter { app };
    domain::cache::execute_plan(plan, opts, &mut emitter, &run_id, cancel).await
}

#[tauri::command]
pub async fn cancel_clean_plan(run_id: String) -> AppResult<bool> {
    Ok(crate::domain::cache_cancellation::cancel(&run_id))
}

#[tauri::command]
pub async fn get_throughput_stats() -> AppResult<ThroughputStats> {
    Ok(crate::domain::throughput_stats::current())
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

#[tauri::command]
pub async fn ignore_residual_path(path: String) -> AppResult<()> {
    crate::domain::cache_ignore::add_ignored(&path)
}
