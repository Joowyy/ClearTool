// Tipos TypeScript que reflejan los structs Rust del backend (src-tauri/src/models/).
//
// Convención: todos los campos en camelCase (los structs Rust usan
// `#[serde(rename_all = "camelCase")]`).

export type Risk = "low" | "medium" | "high";

export interface DriveInfo {
  letter: string;
  totalBytes: number;
  freeBytes: number;
  label: string;
}

export interface SystemSummary {
  osName: string;
  osVersion: string;
  buildNumber: string;
  username: string;
  isElevated: boolean;
  totalRamBytes: number;
  drives: DriveInfo[];
}

export type SizeStrategy = "Logical" | "Physical" | "Lazy";
export type NodeKind = "Dir" | "File" | "Symlink" | "Junction";

export interface ScanTreeInput {
  root: string;
  maxDepth: number;
  followReparsePoints: boolean;
  includeHidden: boolean;
  minSizeBytes: number | null;
  sizeStrategy: SizeStrategy;
}

export interface ScanTreeHandle {
  scanId: string;
}

export interface TreeNode {
  path: string;
  name: string;
  kind: NodeKind;
  sizeBytes: number;
  lastModified: string | null;
  childrenCount: number | null;
  isProtected: boolean;
  error: string | null;
}

export interface DirectorySize {
  path: string;
  logicalBytes: number;
  physicalBytes: number;
  fileCount: number;
  dirCount: number;
}

export interface CacheLocation {
  id: string;
  displayName: string;
  path: string;
  category?: string | null;
  requiresAdmin?: boolean;
  risk?: string | null;
  preconditions?: string[];
  filters?: {
    olderThanDays?: number;
    exclude?: string[];
  };
  consequences?: string[];
  averageSize?: string | null;
}

export interface CacheScanReport {
  id: string;
  resolvedPath: string;
  exists: boolean;
  bytes: number;
  fileCount: number;
  matchedAfterFilters: number;
  bytesAfterFilters: number;
}

export interface CleanCacheInput {
  ids: string[];
  dryRun: boolean;
  createRestorePoint: boolean;
  forceCloseProcesses: boolean;
}

export interface PerLocationResult {
  id: string;
  status: string;
  bytesFreed: number;
  filesDeleted: number;
  errors: string[];
}

export interface CleanReport {
  runId: string;
  startedAt: string;
  finishedAt: string;
  restorePointId: number | null;
  perLocation: PerLocationResult[];
  totalBytesFreed: number;
  totalFilesDeleted: number;
}

export interface BloatwareEntry {
  id: string;
  displayName: string;
  category?: string | null;
  removalStrategy?: string | null;
  packageNames?: string[];
  services?: string[];
  scheduledTasks?: string[];
  risk?: string | null;
  consequences?: string[];
  reversalMethod?: string | null;
  reversalDetails?: string | null;
  requiresElevation?: boolean;
}

export interface DetectedPackage {
  id: string;
  displayName: string;
  installedForUser: boolean;
  installedProvisioned: boolean;
  installLocation: string | null;
  sizeEstimateMb: number | null;
}

export interface RemoveBloatwareInput {
  entryIds: string[];
  dryRun: boolean;
  createRestorePoint: boolean;
  applyPolicies: boolean;
  disableServices: boolean;
}

export interface PerEntryResult {
  id: string;
  status: string;
  methodUsed: string | null;
  error: string | null;
}

export interface RemoveReport {
  runId: string;
  total: number;
  removed: number;
  failed: number;
  skipped: number;
  restorePointSeq: number | null;
  perEntry: PerEntryResult[];
}

export interface Service {
  name: string;
  displayName: string;
  description: string | null;
  state: string;
  startType: string;
  pid: number | null;
  canStop: boolean;
  canPause: boolean;
}

export interface RegistryTweak {
  id: string;
  displayName: string;
  description: string;
  category?: string | null;
  hive: string;
  path: string;
  valueName: string;
  enabledValue: unknown;
  disabledValue: unknown;
  requiresReboot?: boolean;
  risk?: string | null;
  consequences?: string[];
}

export interface TweakState {
  id: string;
  isEnabled: boolean | null;
  currentRaw: unknown;
}

export interface ApplyTweakInput {
  id: string;
  enable: boolean;
  dryRun: boolean;
}

export interface RestorePoint {
  sequenceNumber: number;
  description: string;
  creationTime: string;
  restorePointType: number;
  eventType: number;
}

export interface CreateRestorePointInput {
  description: string;
}

export interface RestoreReport {
  sequenceNumber: number;
  createdAt: string;
  description: string;
  bypassedThrottle: boolean;
}

export interface RegistryRevertOp {
  hive: string;
  key: string;
  name: string;
  kind: string;
  previousValue: unknown;
}

export type ReverseRecipe =
  | { kind: "Registry"; operations: RegistryRevertOp[] }
  | { kind: "Service"; serviceName: string; previousStartType: string; previousState: string }
  | { kind: "AppxReinstall"; packageFamilyName: string; storeUrl: string | null }
  | { kind: "Noop"; reason: string };

export interface AuditEntry {
  runId: string;
  timestamp: string;
  module: string;
  operation: string;
  dryRun: boolean;
  restorePointSeq: number | null;
  itemsAffected: string[];
  reverseRecipe: ReverseRecipe;
  status: string;
  error: string | null;
}

// ── Telemetría en vivo (CPU, RAM, GPU, procesos) ─────────────────────────

export interface TelemetryProcessInfo {
  pid: number;
  name: string;
  cpuPercent: number;
  memoryBytes: number;
}

export interface GpuInfo {
  index: number;
  name: string;
  vendor: string | null;
  usagePercent: number | null;
  memoryUsedBytes: number | null;
  memoryTotalBytes: number | null;
  tempCelsius: number | null;
}

export interface TelemetrySnapshot {
  cpuTotalPercent: number;
  cpuPerCore: number[];
  ramUsedBytes: number;
  ramTotalBytes: number;
  topProcesses: TelemetryProcessInfo[];
  gpus: GpuInfo[];
  timestamp: string;
}

// ── Process Manager ──────────────────────────────────────────────────────

export type ProcessCategory =
  | "system"
  | "service"
  | "browser"
  | "communication"
  | "media"
  | "development"
  | "background"
  | "user-app"
  | "unknown";

export interface ProcessInfo {
  pid: number;
  parentPid: number | null;
  name: string;
  displayName: string | null;
  exePath: string | null;
  sessionId: number;
  startTime: string | null;
  cpuPercent: number;
  memoryBytes: number;
  threadCount: number;
  handleCount: number;
  commandLine: string | null;
  userSid: string | null;
  userName: string | null;
  isUwp: boolean;
  uwpPackageFamily: string | null;
  category: ProcessCategory;
  isProtected: boolean;
}

export interface LockingProcess {
  pid: number;
  name: string;
  path: string | null;
}

export interface ReleaseReport {
  closedCount: number;
  failedCount: number;
  closedProcesses: string[];
}

/// Representación serializada de `AppError` del backend Rust.
export interface AppErrorPayload {
  kind:
    | "io"
    | "registry"
    | "powershell"
    | "permission"
    | "not-elevated"
    | "cancelled"
    | "restore-unavailable"
    | "external"
    | "not-implemented"
    | "catalog"
    | "json";
  message: string;
}

// ── Settings persistentes ────────────────────────────────────────────────

export interface AppearanceSettings {
  theme: string;
  language: string;
  density: string;
}

export interface SafetySettings {
  dryRunGlobal: boolean;
  autoCreateRestorePoint: boolean;
  requireConfirmBeforeBatch: boolean;
  bypassThrottlingRestore: boolean;
}

export interface BehaviorSettings {
  checkUpdatesOnStart: boolean;
  lastSeenAuditRunId: string | null;
  rememberWindowSize: boolean;
}

export interface AdvancedSettings {
  logLevel: string;
  auditLogMaxMb: number;
  diagnosticMode: boolean;
}

export interface Settings {
  settingsVersion: number;
  appearance: AppearanceSettings;
  safety: SafetySettings;
  behavior: BehaviorSettings;
  advanced: AdvancedSettings;
}
