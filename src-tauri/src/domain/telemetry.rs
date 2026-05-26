// domain/telemetry.rs — orquesta el snapshot de monitorización en vivo.
//
// Combina:
//   - `platform::sysmon`  → CPU, RAM, top procesos (sysinfo).
//   - `platform::gpu`     → GPUs (PowerShell + WMI + perf counters).
//
// Devuelve `TelemetrySnapshot` listo para enviar al frontend.

use crate::core::AppResult;
use crate::models::telemetry::TelemetrySnapshot;
use crate::platform::{gpu, sysmon};
use chrono::Utc;

pub fn snapshot() -> AppResult<TelemetrySnapshot> {
    let cpu_mem = sysmon::snapshot();
    // GPU es best-effort: si falla / tarda, devolvemos vacío en vez de bloquear.
    let gpus = gpu::snapshot();

    Ok(TelemetrySnapshot {
        cpu_total_percent: cpu_mem.cpu_total_percent,
        cpu_per_core: cpu_mem.cpu_per_core,
        ram_used_bytes: cpu_mem.ram_used_bytes,
        ram_total_bytes: cpu_mem.ram_total_bytes,
        top_processes: cpu_mem.top_processes,
        gpus,
        timestamp: Utc::now().to_rfc3339(),
    })
}
