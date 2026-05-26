// ipc/registry.rs — tweaks del registro (HKLM/HKCU controlados).

use crate::core::AppResult;
use crate::domain;
use crate::models::registry::{
    ApplyTweakBatchInput, ApplyTweakBatchReport, ApplyTweakInput, RegistryProgressEvent,
    RegistryTweak, TweakState,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn list_registry_tweaks() -> AppResult<Vec<RegistryTweak>> {
    domain::registry::list_tweaks()
}

#[tauri::command]
pub async fn read_registry_tweak_state(id: String) -> AppResult<TweakState> {
    domain::registry::read_state(&id)
}

#[tauri::command]
pub async fn apply_registry_tweak(input: ApplyTweakInput) -> AppResult<()> {
    domain::registry::apply(&input)
}

#[tauri::command]
pub async fn apply_registry_tweak_batch(
    app: AppHandle,
    input: ApplyTweakBatchInput,
) -> AppResult<ApplyTweakBatchReport> {
    let report = domain::registry::apply_batch(
        &input.inputs,
        input.create_restore_point,
        move |processed, total, current_id| {
            let _ = app.emit(
                "registry:progress",
                RegistryProgressEvent {
                    run_id: String::new(),
                    processed,
                    total,
                    current_id: current_id.to_string(),
                },
            );
        },
    )?;
    Ok(report)
}

#[tauri::command]
pub async fn revert_registry_tweak(id: String) -> AppResult<()> {
    domain::registry::revert(&id)
}
