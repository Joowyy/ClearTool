# 09 — Pipeline de eventos enriquecidos del backend

> **Severidad:** 🟡 P1 — sin esto, el frontend no puede mostrar consola
> rica. Es la base sobre la que se montan los docs 10/11/12.
> **Modelo:** Sonnet 4.6.
> **Bloque:** 1 de 4 del set "consola de limpieza".

## 1. Problema

El backend actual emite `cache:progress` con sólo
`{ level, location, message, bytesFreed, filesDeleted }`. Con eso no se
puede:

- Calcular el porcentaje real de progreso.
- Mostrar ETA fiable.
- Saber en qué fase estamos (creando restore point, cerrando procesos,
  limpiando, verificando).
- Mostrar throughput (MB/s) en directo.
- Hacer un resumen final estructurado con duración total.

Además, los `emit` se hacen línea-a-línea sin throttle → con 50
ubicaciones y cientos de archivos, se saturan IPC y devtools.

## 2. Objetivo

Diseñar un **pipeline tipado de eventos** que dé al frontend toda la
información para una UI rica. Tres canales:

| Evento | Cuándo se emite | Payload |
|---|---|---|
| `cache:progress-v2` | Throttle de ≥150 ms entre emits; siempre al cambiar de ubicación | `CleanProgressPayload` |
| `cache:phase` | Cada vez que cambia la fase principal | `CleanPhaseEvent` |
| `cache:summary` | UNA vez al finalizar (éxito o fallo) | `CleanSummaryPayload` |

El evento legacy `cache:progress` **se mantiene** durante una versión
para compatibilidad — el componente nuevo escucha v2, el legacy puede
seguir alimentando logs viejos sin tocar nada.

## 3. Modelos nuevos

`src-tauri/src/models/cache.rs` — añadir:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanPhase {
    /// Iniciando: validando plan, etc.
    Preparing,
    /// Creando restore point (puede tardar 5-30 s).
    CreatingRestorePoint,
    /// Cerrando procesos bloqueantes (sólo si auto_close_blocking).
    ClosingProcesses,
    /// Limpiando archivos — fase principal.
    Cleaning,
    /// Programando bloqueados para borrado en reboot.
    SchedulingReboot,
    /// Verificando resultado tras la limpieza.
    Verifying,
    /// Limpieza terminada con éxito.
    Complete,
    /// Limpieza terminada con error.
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgressPayload {
    pub run_id: String,
    pub phase: CleanPhase,
    pub current_location_id: Option<String>,
    pub current_location_display_name: Option<String>,
    pub current_location_index: u32,
    pub total_locations: u32,
    pub bytes_freed: u64,
    pub bytes_scheduled: u64,
    pub total_estimated_bytes: u64,
    pub files_deleted: u32,
    pub files_scheduled: u32,
    pub files_failed: u32,
    pub elapsed_ms: u64,
    /// None mientras throughput < 50 MB (muestra insuficiente).
    pub eta_secs: Option<f32>,
    /// Bytes/s recientes (ventana móvil de 2 s).
    pub throughput_bytes_per_sec: u64,
    pub last_line: Option<CleanLogLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanLogLine {
    pub level: String,         // "info" | "warn" | "error" | "success"
    pub location: String,
    pub message: String,
    pub timestamp_ms: u64,     // ms desde epoch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanPhaseEvent {
    pub run_id: String,
    pub phase: CleanPhase,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanSummaryPayload {
    pub run_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub total_bytes_freed: u64,
    pub total_bytes_scheduled_reboot: u64,
    pub total_files_deleted: u32,
    pub total_files_scheduled_reboot: u32,
    pub total_files_failed: u32,
    pub locations_processed: u32,
    pub locations_with_errors: Vec<String>,
    pub restore_point_seq: Option<u32>,
    pub mean_throughput_bytes_per_sec: u64,
    /// El CleanReportV2 íntegro para clientes que quieran el detalle.
    pub report: CleanReportV2,
}
```

Añadir `ts-rs` derives (`#[derive(TS)] #[ts(export)]`) según convención
del proyecto para que se generen bindings TypeScript automáticos.

## 4. Cambios en `domain/cache.rs::execute_plan`

### 4.1 Cambiar la firma del callback

Hoy es `F: FnMut(&str, &str, &str, u64, u64)`. Sustituir por un
**emisor estructurado** que el IPC se encargue de mapear a eventos
Tauri.

```rust
pub trait CleanEmitter: Send + Sync {
    fn line(&mut self, line: CleanLogLine);
    fn progress(&mut self, payload: CleanProgressPayload);
    fn phase(&mut self, event: CleanPhaseEvent);
    fn summary(&mut self, payload: CleanSummaryPayload);
}

pub async fn execute_plan<E: CleanEmitter>(
    plan: CleanPlan,
    opts: ExecutePlanOpts,
    emitter: &mut E,
) -> AppResult<CleanReportV2> { /* ... */ }
```

Justificación de trait + `&mut` en vez de un cierre con 4 firmas:
serializa mejor la intención y permite stub en tests
(`MockCleanEmitter { lines: Vec<...>, ... }`).

### 4.2 Tracker interno

```rust
struct ProgressTracker {
    run_id: String,
    started: Instant,
    phase: CleanPhase,
    total_locations: u32,
    current_index: u32,
    bytes_freed: u64,
    bytes_scheduled: u64,
    total_estimated_bytes: u64,
    files_deleted: u32,
    files_scheduled: u32,
    files_failed: u32,
    /// Ventana móvil de muestras (bytes_freed, instant). Capacidad 60.
    samples: VecDeque<(u64, Instant)>,
    last_emit: Instant,
}

impl ProgressTracker {
    /// Mínimo entre emits para no saturar IPC.
    const THROTTLE: Duration = Duration::from_millis(150);

    fn record_bytes(&mut self, delta: u64) {
        self.bytes_freed += delta;
        self.samples.push_back((self.bytes_freed, Instant::now()));
        // Mantener sólo los últimos 2 s.
        while let Some((_, t)) = self.samples.front() {
            if t.elapsed() > Duration::from_secs(2) {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    fn throughput_bps(&self) -> u64 {
        match (self.samples.front(), self.samples.back()) {
            (Some((b0, t0)), Some((b1, t1))) if t1 > t0 => {
                let dt = t1.duration_since(*t0).as_secs_f64();
                if dt < 0.1 { return 0; }
                (((b1 - b0) as f64) / dt) as u64
            }
            _ => 0,
        }
    }

    fn eta_secs(&self) -> Option<f32> {
        let tp = self.throughput_bps();
        if tp < 1_000_000 { return None; } // muestra insuficiente
        let remaining = self.total_estimated_bytes
            .saturating_sub(self.bytes_freed);
        Some(remaining as f32 / tp as f32)
    }

    fn maybe_emit<E: CleanEmitter>(
        &mut self,
        emitter: &mut E,
        force: bool,
        current_id: Option<&str>,
        current_name: Option<&str>,
        last_line: Option<CleanLogLine>,
    ) {
        if !force && self.last_emit.elapsed() < Self::THROTTLE {
            return;
        }
        self.last_emit = Instant::now();
        emitter.progress(CleanProgressPayload {
            run_id: self.run_id.clone(),
            phase: self.phase.clone(),
            current_location_id: current_id.map(str::to_string),
            current_location_display_name: current_name.map(str::to_string),
            current_location_index: self.current_index,
            total_locations: self.total_locations,
            bytes_freed: self.bytes_freed,
            bytes_scheduled: self.bytes_scheduled,
            total_estimated_bytes: self.total_estimated_bytes,
            files_deleted: self.files_deleted,
            files_scheduled: self.files_scheduled,
            files_failed: self.files_failed,
            elapsed_ms: self.started.elapsed().as_millis() as u64,
            eta_secs: self.eta_secs(),
            throughput_bytes_per_sec: self.throughput_bps(),
            last_line,
        });
    }

    fn set_phase<E: CleanEmitter>(&mut self, emitter: &mut E, phase: CleanPhase) {
        self.phase = phase.clone();
        emitter.phase(CleanPhaseEvent {
            run_id: self.run_id.clone(),
            phase,
            elapsed_ms: self.started.elapsed().as_millis() as u64,
        });
    }
}
```

### 4.3 Reescribir el cuerpo de `execute_plan` por fases

Sustituir el flujo lineal actual por bloques delimitados:

```rust
pub async fn execute_plan<E: CleanEmitter>(
    plan: CleanPlan,
    opts: ExecutePlanOpts,
    emitter: &mut E,
) -> AppResult<CleanReportV2> {
    let run_id = Uuid::new_v4().to_string();
    let mut tracker = ProgressTracker {
        run_id: run_id.clone(),
        started: Instant::now(),
        phase: CleanPhase::Preparing,
        total_locations: plan.ready.len() as u32,
        current_index: 0,
        bytes_freed: 0,
        bytes_scheduled: 0,
        total_estimated_bytes: plan.total_estimated_bytes,
        files_deleted: 0,
        files_scheduled: 0,
        files_failed: 0,
        samples: VecDeque::with_capacity(64),
        last_emit: Instant::now() - ProgressTracker::THROTTLE * 2,
    };

    tracker.set_phase(emitter, CleanPhase::Preparing);
    tracker.maybe_emit(emitter, true, None, None, None);

    // 1. Restore point.
    let restore_point_seq = if !opts.dry_run && opts.create_restore_point {
        tracker.set_phase(emitter, CleanPhase::CreatingRestorePoint);
        tracker.maybe_emit(emitter, true, None, None, None);
        let desc = format!("ClearTool — caché ({} ubicaciones)", plan.ready.len());
        crate::platform::restore_point::create(&desc, 12, true).ok()
    } else {
        None
    };

    // 2. Cierre de procesos.
    let mut closed_pids: Vec<u32> = Vec::new();
    if opts.auto_close_blocking {
        tracker.set_phase(emitter, CleanPhase::ClosingProcesses);
        // ... loop existente, emitir line() por cada cierre ...
    }

    // 3. Limpieza.
    tracker.set_phase(emitter, CleanPhase::Cleaning);
    let mut per_location: Vec<LocationResult> = Vec::new();

    for (idx, ready_loc) in plan.ready.iter().enumerate() {
        tracker.current_index = (idx + 1) as u32;
        emitter.line(CleanLogLine {
            level: "info".into(),
            location: ready_loc.display_name.clone(),
            message: format!("Limpiando: {}", ready_loc.resolved_path),
            timestamp_ms: now_ms(),
        });
        tracker.maybe_emit(
            emitter, true,
            Some(&ready_loc.id),
            Some(&ready_loc.display_name),
            None,
        );

        let result = execute_one_location(ready_loc, &opts, &mut tracker, emitter).await;

        per_location.push(result);
    }

    // 4. Programar bloqueados.
    if opts.schedule_blocked_for_reboot && !opts.dry_run {
        tracker.set_phase(emitter, CleanPhase::SchedulingReboot);
        // ... loop existente ...
    }

    // 5. Verify.
    tracker.set_phase(emitter, CleanPhase::Verifying);
    let report = CleanReportV2 {
        plan_id: plan.plan_id.clone(),
        run_id: run_id.clone(),
        started_at: tracker.started_at_string(),
        finished_at: Utc::now().to_rfc3339(),
        restore_point_seq,
        per_location,
        total_bytes_freed: tracker.bytes_freed,
        total_bytes_scheduled_reboot: tracker.bytes_scheduled,
        total_bytes_failed: 0, // calcular como antes
        closed_processes: closed_pids,
    };

    // 6. Resumen + invalidación cache (igual que antes).
    let summary = build_summary(&tracker, &report, &plan);
    tracker.set_phase(emitter, CleanPhase::Complete);
    emitter.summary(summary);

    if !opts.dry_run {
        crate::domain::cache_background::invalidate();
        tauri::async_runtime::spawn(async {
            let _ = crate::domain::cache_background::recompute_and_store().await;
        });
    }

    // 7. Persistir muestra de throughput.
    let mean_tp = tracker.throughput_bps();
    let _ = crate::domain::throughput_stats::push_sample(mean_tp);

    Ok(report)
}
```

### 4.4 Adaptar `execute_one_location` y `walk_and_delete`

Aceptar `&mut ProgressTracker` y `&mut E` para llamar a `record_bytes()`
y `maybe_emit()` durante el walk. Cada vez que un archivo se borra:

```rust
tracker.record_bytes(size);
tracker.files_deleted += 1;
tracker.maybe_emit(emitter, false, Some(&loc.id), Some(&loc.display_name), None);
```

El `maybe_emit(force=false)` se autolimita a 150 ms — no añade
overhead perceptible.

## 5. Persistencia de throughput

Nuevo módulo `src-tauri/src/domain/throughput_stats.rs`:

```rust
//! Estadísticas de throughput de limpieza, persistidas en
//! %APPDATA%\ClearTool\throughput-stats.json. Buffer circular
//! de las últimas 20 ejecuciones.

use crate::core::AppResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThroughputStats {
    /// Bytes/s de cada ejecución pasada.
    pub samples: Vec<u64>,
    pub mean_bytes_per_sec: u64,
    pub p95_bytes_per_sec: u64,
}

const MAX_SAMPLES: usize = 20;
/// Default conservador si no hay muestras: 100 MB/s.
pub const DEFAULT_BPS: u64 = 100 * 1024 * 1024;

fn stats_path() -> PathBuf {
    let mut p = dirs::data_dir().unwrap_or_default();
    p.push("ClearTool");
    let _ = fs::create_dir_all(&p);
    p.push("throughput-stats.json");
    p
}

pub fn load() -> ThroughputStats {
    let path = stats_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn push_sample(bps: u64) -> AppResult<()> {
    if bps == 0 { return Ok(()); }
    let mut stats = load();
    stats.samples.push(bps);
    if stats.samples.len() > MAX_SAMPLES {
        stats.samples.drain(0..stats.samples.len() - MAX_SAMPLES);
    }
    recompute(&mut stats);
    let path = stats_path();
    fs::write(&path, serde_json::to_string_pretty(&stats)?)?;
    Ok(())
}

fn recompute(stats: &mut ThroughputStats) {
    if stats.samples.is_empty() {
        stats.mean_bytes_per_sec = DEFAULT_BPS;
        stats.p95_bytes_per_sec = DEFAULT_BPS;
        return;
    }
    let sum: u64 = stats.samples.iter().sum();
    stats.mean_bytes_per_sec = sum / stats.samples.len() as u64;
    let mut sorted = stats.samples.clone();
    sorted.sort_unstable();
    let p95_idx = ((sorted.len() as f32) * 0.95).floor() as usize;
    stats.p95_bytes_per_sec = sorted[p95_idx.min(sorted.len() - 1)];
}

pub fn current() -> ThroughputStats {
    let mut s = load();
    if s.samples.is_empty() {
        s.mean_bytes_per_sec = DEFAULT_BPS;
        s.p95_bytes_per_sec = DEFAULT_BPS;
    }
    s
}
```

(Si `dirs` no está, añadir al `Cargo.toml`: `dirs = "5"` o resolver
APPDATA con `std::env`.)

## 6. Cambios en `ipc/cache.rs`

```rust
#[derive(Debug, Clone, Serialize)]
pub struct TauriCleanEmitter {
    app: tauri::AppHandle,
}

impl crate::domain::cache::CleanEmitter for TauriCleanEmitter {
    fn line(&mut self, line: crate::models::cache::CleanLogLine) {
        let _ = self.app.emit("cache:line", line);
    }
    fn progress(&mut self, payload: crate::models::cache::CleanProgressPayload) {
        let _ = self.app.emit("cache:progress-v2", payload);
    }
    fn phase(&mut self, event: crate::models::cache::CleanPhaseEvent) {
        let _ = self.app.emit("cache:phase", event);
    }
    fn summary(&mut self, payload: crate::models::cache::CleanSummaryPayload) {
        let _ = self.app.emit("cache:summary", payload);
    }
}

#[tauri::command]
pub async fn execute_clean_plan(
    app: tauri::AppHandle,
    plan: CleanPlan,
    opts: ExecutePlanOpts,
) -> AppResult<CleanReportV2> {
    let mut emitter = TauriCleanEmitter { app };
    domain::cache::execute_plan(plan, opts, &mut emitter).await
}
```

Mantener `cache:debug` y `cache:progress` legacy emitiéndose dentro de
`emitter.line()` por compatibilidad de transición (una versión más se
puede retirar).

## 7. Comando IPC adicional para la estimación previa

```rust
#[tauri::command]
pub async fn get_throughput_stats() -> AppResult<ThroughputStats> {
    Ok(crate::domain::throughput_stats::current())
}
```

Lo consume el doc 12 (estimación previa al pulsar Limpiar).

## 8. Bindings TypeScript

`src/api/events.ts` — añadir:

```ts
export interface CleanLogLinePayload { /* mirror de CleanLogLine */ }
export interface CleanProgressV2Payload { /* mirror */ }
export interface CleanPhasePayload { /* mirror */ }
export interface CleanSummaryPayload { /* mirror */ }

export const TauriEvents = {
  // ... existentes ...
  CacheLine: "cache:line",
  CacheProgressV2: "cache:progress-v2",
  CachePhase: "cache:phase",
  CacheSummary: "cache:summary",
} as const;

export interface EventPayloads {
  // ... existentes ...
  "cache:line": CleanLogLinePayload;
  "cache:progress-v2": CleanProgressV2Payload;
  "cache:phase": CleanPhasePayload;
  "cache:summary": CleanSummaryPayload;
}
```

Si el proyecto ya usa `ts-rs` para generar tipos automáticos, los DTOs
de `models/cache.rs` se exportan a `src/api/types.ts` en el build de
backend; en ese caso, sólo registrar los nombres de evento aquí.

## 9. Tests

```rust
#[cfg(test)]
mod progress_tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockEmitter {
        lines: Vec<CleanLogLine>,
        progresses: Vec<CleanProgressPayload>,
        phases: Vec<CleanPhaseEvent>,
        summary: Option<CleanSummaryPayload>,
    }
    impl CleanEmitter for MockEmitter { /* ... */ }

    #[test]
    fn throttle_limits_emissions_to_150ms() {
        let mut tracker = ProgressTracker::new("test", 10, 100_000_000);
        let mut em = MockEmitter::default();
        for _ in 0..1000 {
            tracker.record_bytes(1_000);
            tracker.maybe_emit(&mut em, false, None, None, None);
        }
        // En menos de 1 ms se llaman 1000 veces, pero sólo se emiten <10.
        assert!(em.progresses.len() < 10);
    }

    #[test]
    fn eta_none_until_50mb_freed() {
        let mut tracker = ProgressTracker::new("t", 1, 1_000_000_000);
        tracker.record_bytes(1_000_000); // 1 MB
        assert!(tracker.eta_secs().is_none());
    }

    #[test]
    fn phase_event_fires_on_transition() {
        let mut em = MockEmitter::default();
        let mut tracker = ProgressTracker::new("t", 1, 0);
        tracker.set_phase(&mut em, CleanPhase::Cleaning);
        tracker.set_phase(&mut em, CleanPhase::Verifying);
        assert_eq!(em.phases.len(), 2);
        assert_eq!(em.phases[1].phase, CleanPhase::Verifying);
    }

    #[test]
    fn throughput_window_is_2_seconds() {
        // muestreo manual con instants forjados
    }
}
```

## 10. Criterio de done

- [ ] Modelos nuevos con `Serialize + Deserialize + Clone + Debug` y
      `ts-rs` (si aplica) en `models/cache.rs`.
- [ ] `CleanEmitter` trait + `TauriCleanEmitter` implementación.
- [ ] `execute_plan` reescrito por fases con `ProgressTracker`.
- [ ] Eventos `cache:line`, `cache:progress-v2`, `cache:phase`,
      `cache:summary` emitiéndose en orden coherente.
- [ ] Throttle ≥150 ms verificado en test.
- [ ] `domain/throughput_stats.rs` persiste a JSON, ring buffer 20.
- [ ] `get_throughput_stats` IPC disponible.
- [ ] 4 tests nuevos pasando (`throttle_limits_emissions_to_150ms`,
      `eta_none_until_50mb_freed`, `phase_event_fires_on_transition`,
      `throughput_window_is_2_seconds`).
- [ ] `cargo check` sin warnings.
- [ ] Eventos legacy (`cache:progress`, `cache:debug`) siguen
      funcionando.

## 11. Riesgos / efectos secundarios

- **Compatibilidad**: los oyentes legacy (`useCacheDebug` actual) siguen
  vivos. La consola nueva (doc 10) escucha v2; coexisten sin choques.
- **Performance**: throttle de 150 ms es el sweet spot (~6-7 frames/s)
  para una barra de progreso fluida sin saturar IPC.
- **Persistencia**: si el JSON está corrupto, `load()` cae a `default()`
  silenciosamente. Documentar y testear ese caso.
- **`tauri::AppHandle` en el callback**: ya estaba antes; nada nuevo.
- **`ts-rs`**: si no está habilitado para todos los structs, hay que
  declararlos manualmente en `src/api/events.ts`. No es trabajo grande.
