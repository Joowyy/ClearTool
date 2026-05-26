# Paso 03 — Modelos: `CleanPlan`, `LocationStatus`, `BlockedLocation`

**Área**: 02-cache-engine
**Tiempo estimado**: 1.5 horas
**Dependencias**: Paso 01 (LockingProcess), Paso 02 (PendingRename)

## Qué hacemos

Definir los DTOs que la fase de pre-flight devuelve al frontend. Estos modelos guían toda la UI nueva.

## Por qué

Cambiamos el contrato de "limpia y reporta" a "analiza primero, presenta plan, ejecuta tras confirmación". El plan es lo que viaja entre backend y frontend.

## Archivos que tocamos

- `src-tauri/src/models/cache.rs` (modificado o expandido)

## Cómo

### 1. Reescribir `models/cache.rs`

Añadir (sin borrar lo existente que esté en uso):

```rust
// src-tauri/src/models/cache.rs
use serde::{Deserialize, Serialize};
use crate::models::process_lock::LockingProcess;

// ── Modelos existentes (mantener) ──
// pub struct CacheLocation { ... }
// pub struct CacheScanReport { ... }
// pub struct CleanCacheInput { ... }
// pub struct CleanReport { ... }

// ── NUEVO en v2 ──

/// Plan de limpieza generado por `analyze_locations`.
/// Es lo que el frontend recibe para presentar al usuario antes de ejecutar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanPlan {
    pub plan_id: String,                              // UUID, usado en verify
    pub generated_at: String,                         // RFC3339
    pub ready: Vec<ReadyLocation>,                    // listas para limpiar
    pub blocked: Vec<BlockedLocation>,                // bloqueadas por procesos
    pub permission_issues: Vec<PermissionLocation>,   // requieren ownership / TI
    pub skipped: Vec<SkippedLocation>,                // no existen / vacías
    pub total_estimated_bytes: u64,                   // suma de ready
    pub total_blocked_bytes: u64,                     // suma de blocked
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyLocation {
    pub id: String,                  // del catálogo
    pub display_name: String,
    pub resolved_path: String,
    pub bytes: u64,
    pub file_count: u32,
    pub strategy: CleanStrategy,
    pub age_oldest_file: Option<String>,  // RFC3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedLocation {
    pub id: String,
    pub display_name: String,
    pub resolved_path: String,
    pub bytes: u64,
    pub locked_by: Vec<LockingProcess>,
    pub suggested_action: BlockedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BlockedAction {
    /// "Cerrar Spotify y reintentar"
    CloseProcess { pid: u32, process_name: String },
    /// "Programar para próximo reinicio"
    ScheduleReboot,
    /// "Saltar (no se puede automatizar)"
    SkipOnly { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionLocation {
    pub id: String,
    pub display_name: String,
    pub resolved_path: String,
    pub bytes: u64,
    pub reason: String,    // "requires TrustedInstaller", "elevation needed"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLocation {
    pub id: String,
    pub display_name: String,
    pub reason: SkipReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SkipReason {
    DoesNotExist,
    Empty,
    DisallowedByAllowlist,    // ej. Windows Terminal
    Precondition { name: String },
}

/// Estrategia de limpieza por tipo de path. Lleva el catálogo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CleanStrategy {
    /// `remove_dir_all` directo. Caches de sistema.
    DirectDelete,
    /// Detecta si la app UWP está corriendo. Si sí: ofrece cerrar.
    UwpAppAware { package_family_name: String },
    /// Para Chrome/Edge/Firefox: detecta el proceso del navegador.
    BrowserAware,
    /// Detecta procesos con handle abierto vía RestartManager.
    ProcessLocked,
    /// Requiere reboot (drivers, kernel cache).
    SystemRestartRequired,
    /// `takeown` + `icacls` antes de delete (TrustedInstaller paths).
    TakeOwnershipAndDelete,
}

/// Opciones para `execute_plan`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanOpts {
    pub plan_id: String,
    pub auto_close_blocking: bool,          // si true, cierra procesos sin preguntar
    pub schedule_blocked_for_reboot: bool,  // si true, los blocked → MoveFileEx
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub timeout_per_location_secs: u64,
}

/// Resultado de execute_plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReportV2 {
    pub plan_id: String,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub restore_point_seq: Option<u32>,
    pub per_location: Vec<LocationResult>,
    pub total_bytes_freed: u64,
    pub total_bytes_scheduled_reboot: u64,
    pub total_bytes_failed: u64,
    pub closed_processes: Vec<u32>,  // PIDs que se cerraron
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationResult {
    pub id: String,
    pub status: LocationStatus,
    pub bytes_freed: u64,
    pub bytes_scheduled: u64,
    pub files_deleted: u32,
    pub files_scheduled: u32,
    pub files_failed: u32,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum LocationStatus {
    Cleaned,           // todo borrado
    PartialReboot,     // parte borrada, parte schedule
    Skipped,
    Failed,
}
```

### 2. Generar bindings TypeScript (si usas ts-rs)

Si el proyecto usa `ts-rs` o similar, añadir derive `#[derive(ts_rs::TS)]` y exportar. Si genera bindings on-build, ya está hecho. Si no, copiar manualmente al `src/api/types.ts`:

```ts
// src/api/types.ts (añadir)
export interface CleanPlan {
  planId: string;
  generatedAt: string;
  ready: ReadyLocation[];
  blocked: BlockedLocation[];
  permissionIssues: PermissionLocation[];
  skipped: SkippedLocation[];
  totalEstimatedBytes: number;
  totalBlockedBytes: number;
}

export interface ReadyLocation {
  id: string;
  displayName: string;
  resolvedPath: string;
  bytes: number;
  fileCount: number;
  strategy: CleanStrategy;
  ageOldestFile: string | null;
}

export interface BlockedLocation {
  id: string;
  displayName: string;
  resolvedPath: string;
  bytes: number;
  lockedBy: LockingProcess[];
  suggestedAction: BlockedAction;
}

export type BlockedAction =
  | { kind: "closeProcess"; pid: number; processName: string }
  | { kind: "scheduleReboot" }
  | { kind: "skipOnly"; reason: string };

export type CleanStrategy =
  | "direct-delete"
  | { uwpAppAware: { packageFamilyName: string } }
  | "browser-aware"
  | "process-locked"
  | "system-restart-required"
  | "take-ownership-and-delete";

// ... resto de tipos
```

### 3. Verifica que compila

```bash
cd src-tauri && cargo check
cd .. && npx tsc --noEmit
```

## Criterio de done

- [ ] Todos los structs/enums definidos en `models/cache.rs`.
- [ ] `cargo check` pasa sin errores.
- [ ] Tipos TypeScript correspondientes en `src/api/types.ts`.
- [ ] `npx tsc --noEmit` pasa sin errores.
- [ ] Discriminantes de enums correctos (`kebab-case` para CleanStrategy, `camelCase` para BlockedAction kind).
