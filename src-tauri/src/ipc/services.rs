// ipc/services.rs — gestión de servicios de Windows.

use crate::core::AppResult;
use crate::domain;
use crate::models::service::{
    ApplyPresetReport, ApplyServicePresetInput, Service, ServiceProgressEvent,
    SetServiceStateInput,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn list_services() -> AppResult<Vec<Service>> {
    domain::services::list()
}

#[tauri::command]
pub async fn set_service_state(input: SetServiceStateInput) -> AppResult<()> {
    domain::services::set_state(&input)
}

#[tauri::command]
pub async fn service_dependencies(name: String) -> AppResult<Vec<String>> {
    crate::platform::services::dependencies_of(&name)
}

#[tauri::command]
pub async fn apply_service_preset(
    app: AppHandle,
    input: ApplyServicePresetInput,
) -> AppResult<ApplyPresetReport> {
    domain::services::apply_preset(&input, move |processed, total, current| {
        let _ = app.emit(
            "services:progress",
            ServiceProgressEvent {
                run_id: String::new(),
                processed,
                total,
                current_name: current.to_string(),
            },
        );
    })
}
