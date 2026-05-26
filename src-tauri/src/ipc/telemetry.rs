// ipc/telemetry.rs — comando Tauri que entrega el snapshot de telemetría.
//
// Se ejecuta en `spawn_blocking` porque `sysinfo` y `Get-Counter` son
// síncronos y pueden tardar 50-200ms; no bloquear el reactor.

use crate::core::{AppError, AppResult};
use crate::domain;
use crate::models::telemetry::TelemetrySnapshot;

#[tauri::command]
pub async fn get_telemetry_snapshot() -> AppResult<TelemetrySnapshot> {
    tokio::task::spawn_blocking(domain::telemetry::snapshot)
        .await
        .map_err(|e| AppError::External(format!("join: {e}")))?
}
