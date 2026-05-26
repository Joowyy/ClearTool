// platform/sysmon.rs — wrappers de `sysinfo` para CPU, RAM y procesos.
//
// Mantenemos un `System` único (Mutex global) entre llamadas para que el
// cálculo de CPU% sea correcto: `sysinfo` requiere dos refreshes con un
// pequeño delta de tiempo entre ellos para producir un porcentaje válido.
// Como el frontend hace polling cada ~1.5s, el delta natural ya es suficiente
// — sólo hacemos el "warm-up" extra en el primer arranque.

use crate::models::telemetry::ProcessInfo;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, System};

static SYSTEM: Lazy<Mutex<SysState>> = Lazy::new(|| Mutex::new(SysState::new()));

struct SysState {
    sys: System,
    warm: bool,
}

impl SysState {
    fn new() -> Self {
        SysState {
            sys: System::new(),
            warm: false,
        }
    }

    /// Hace dos refreshes con un sleep mínimo para que el primer snapshot
    /// devuelva CPU% válido (sysinfo necesita un delta mayor que
    /// MINIMUM_CPU_UPDATE_INTERVAL).
    fn warm_up(&mut self) {
        if self.warm {
            return;
        }
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        self.warm = true;
    }
}

pub struct CpuMemSnapshot {
    pub cpu_total_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub top_processes: Vec<ProcessInfo>,
}

pub fn snapshot() -> CpuMemSnapshot {
    let mut guard = SYSTEM.lock().expect("SYSTEM mutex poisoned");
    guard.warm_up();

    guard.sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    guard
        .sys
        .refresh_memory_specifics(MemoryRefreshKind::everything());
    guard.sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    let cpu_per_core: Vec<f32> = guard.sys.cpus().iter().map(|c| c.cpu_usage()).collect();
    let cpu_total_percent = if cpu_per_core.is_empty() {
        0.0
    } else {
        cpu_per_core.iter().sum::<f32>() / cpu_per_core.len() as f32
    };

    let ram_used_bytes = guard.sys.used_memory();
    let ram_total_bytes = guard.sys.total_memory();

    // Top 5 procesos por uso de CPU (suma simple — para "más activos" cuenta
    // tanto CPU como RAM, pero CPU es la métrica que el usuario percibe).
    let mut procs: Vec<ProcessInfo> = guard
        .sys
        .processes()
        .iter()
        .map(|(pid, p)| ProcessInfo {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
        })
        .collect();
    procs.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(5);

    CpuMemSnapshot {
        cpu_total_percent,
        cpu_per_core,
        ram_used_bytes,
        ram_total_bytes,
        top_processes: procs,
    }
}
