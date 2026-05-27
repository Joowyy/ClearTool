// models/cache.rs — DTOs de cachés y reportes de limpieza.

use serde::{Deserialize, Serialize};
use crate::models::process::LockingProcess;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    pub entries: Vec<CacheLocation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheFilters {
    #[serde(default)]
    pub older_than_days: Option<u64>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheLocation {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub path: String,
    pub category: String,
    #[serde(default)]
    pub requires_admin: bool,
    pub risk: String,
    #[serde(default)]
    pub consequences: Vec<String>,
    #[serde(default)]
    pub average_size: Option<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub filters: CacheFilters,
    #[serde(default)]
    pub min_windows_build: Option<u32>,
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub locked_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheScanReport {
    pub id: String,
    pub resolved_path: String,
    pub exists: bool,
    pub bytes: u64,
    pub file_count: u64,
    pub matched_after_filters: u64,
    pub bytes_after_filters: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheInput {
    pub ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub force_close_processes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerLocationResult {
    pub id: String,
    pub status: String,
    pub bytes_freed: u64,
    pub files_deleted: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub restore_point_id: Option<u32>,
    pub per_location: Vec<PerLocationResult>,
    pub total_bytes_freed: u64,
    pub total_files_deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgressEvent {
    pub run_id: String,
    pub level: String,
    pub location_display: String,
    pub message: String,
    pub bytes_delta: u64,
    pub files_delta: u64,
}

// ── NUEVO en v2: CleanPlan y tipos asociados ──

/// Plan de limpieza generado por `analyze_locations`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanPlan {
    pub plan_id: String,
    pub generated_at: String,
    pub ready: Vec<ReadyLocation>,
    pub blocked: Vec<BlockedLocation>,
    pub permission_issues: Vec<PermissionLocation>,
    pub skipped: Vec<SkippedLocation>,
    pub total_estimated_bytes: u64,
    pub total_blocked_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyLocation {
    pub id: String,
    pub display_name: String,
    pub resolved_path: String,
    pub bytes: u64,
    pub file_count: u32,
    pub strategy: CleanStrategy,
    pub age_oldest_file: Option<String>,
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
    CloseProcess { pid: u32, process_name: String },
    ScheduleReboot,
    SkipOnly { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionLocation {
    pub id: String,
    pub display_name: String,
    pub resolved_path: String,
    pub bytes: u64,
    pub reason: String,
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
    DisallowedByAllowlist,
    Precondition { name: String },
    UserIgnored,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CleanStrategy {
    DirectDelete,
    UwpAppAware { package_family_name: String },
    BrowserAware,
    ProcessLocked,
    SystemRestartRequired,
    TakeOwnershipAndDelete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanOpts {
    pub plan_id: String,
    pub auto_close_blocking: bool,
    pub schedule_blocked_for_reboot: bool,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub timeout_per_location_secs: u64,
}

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
    pub closed_processes: Vec<u32>,
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
    Cleaned,
    PartialReboot,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReport {
    pub plan_id: String,
    pub verified_at: String,
    pub per_location: Vec<VerifyLocationResult>,
    pub total_actually_freed: u64,
    pub total_still_present: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyLocationResult {
    pub id: String,
    pub display_name: String,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub bytes_actually_freed: u64,
    pub files_pending_reboot: u32,
    pub success_percent: f32,
    #[serde(default)]
    pub residuals: Vec<ResidualEntry>,
}

// ── Doc 15: clasificación de bytes residuales ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResidualEntry {
    pub path: String,
    pub bytes: u64,
    pub reason: ResidualReason,
    /// Hasta 3 rutas de muestra para que el usuario reconozca el contenido.
    pub sample_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ResidualReason {
    LockedBySystem { suggested_action: String },
    PendingReboot,
    AccessDenied,
    FilteredOut,
    ReparsePoint,
    Unknown,
}

impl Default for ResidualReason {
    fn default() -> Self {
        Self::Unknown
    }
}

// ── Doc 09: pipeline de eventos enriquecidos ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CleanPhase {
    Preparing,
    CreatingRestorePoint,
    ClosingProcesses,
    Cleaning,
    SchedulingReboot,
    Verifying,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanLogLine {
    pub level: String,
    pub location: String,
    pub message: String,
    pub timestamp_ms: u64,
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
    pub eta_secs: Option<f32>,
    /// true cuando el ETA viene de la ventana móvil (throughput estable).
    /// false cuando es estimación amplia (acumulado o fallback).
    #[serde(default)]
    pub eta_is_precise: bool,
    pub throughput_bytes_per_sec: u64,
    pub last_line: Option<CleanLogLine>,
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
    pub report: CleanReportV2,
    #[serde(default)]
    pub cancelled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheDenylist {
    pub patterns: Vec<String>,
    #[serde(default)]
    pub exact_paths: Vec<String>,
    #[serde(default)]
    pub ids: Vec<String>,
}
