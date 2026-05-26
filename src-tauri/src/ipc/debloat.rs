// ipc/debloat.rs — eliminación de bloatware (Appx + uninstallers).

use crate::core::AppResult;
use crate::domain;
use crate::models::debloat::{
    BloatwareEntry, DebloatProgressEvent, DetectedPackage, RemoveBloatwareInput, RemoveReport,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn list_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    domain::debloat::list_catalog()
}

#[tauri::command]
pub async fn detect_installed_bloatware() -> AppResult<Vec<DetectedPackage>> {
    domain::debloat::detect_installed()
}

#[tauri::command]
pub async fn remove_bloatware(
    app: AppHandle,
    input: RemoveBloatwareInput,
) -> AppResult<RemoveReport> {
    domain::debloat::remove(&input, move |processed, total, current_id, current_display| {
        let _ = app.emit(
            "debloat:progress",
            DebloatProgressEvent {
                run_id: String::new(),
                processed,
                total,
                current_id: current_id.to_string(),
                current_display: current_display.to_string(),
            },
        );
    })
}
