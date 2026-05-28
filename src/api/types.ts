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

export type NodeKind = "Dir" | "File" | "Symlink" | "Junction";

export type DriveType =
  | "fixed"
  | "removable"
  | "network"
  | "cdRom"
  | "ramDisk"
  | "unknown";

export interface DriveListing {
  letter: string;
  rootPath: string;
  label: string;
  filesystem: string;
  driveType: DriveType;
  totalBytes: number;
  freeBytes: number;
  isReady: boolean;
}

export type ExtCategory =
  | "media"
  | "image"
  | "code"
  | "docs"
  | "archive"
  | "executable"
  | "database"
  | "font"
  | "threeD"
  | "other";

export interface TreemapNode {
  name: string;
  path: string;
  sizeBytes: number;
  kind: NodeKind;
  extension: string | null;
  fileCount: number;
  dirCount: number;
  lastModified: string | null;
  percentOfParent: number;
  percentOfRoot: number;
  children: TreemapNode[];
  truncated: boolean;
  error: string | null;
}

export interface BuildTreemapInput {
  root: string;
  scanId: string;
  maxDepthEmit?: number;
  minSizeMb?: number;
  followReparsePoints?: boolean;
  includeHidden?: boolean;
  includeSystem?: boolean;
}

export interface ExtensionStat {
  extension: string;
  category: ExtCategory;
  bytes: number;
  fileCount: number;
  percent: number;
}

export interface FolderStat {
  path: string;
  name: string;
  bytes: number;
  fileCount: number;
  percent: number;
}

export interface FileStat {
  path: string;
  name: string;
  bytes: number;
  extension: string | null;
  lastModified: string | null;
}

export interface AgeBucket {
  bytes: number;
  fileCount: number;
}

export interface AgeDistribution {
  last7Days: AgeBucket;
  last30Days: AgeBucket;
  last90Days: AgeBucket;
  last1Year: AgeBucket;
  last5Years: AgeBucket;
  older: AgeBucket;
}

export interface DiskAnalysisReport {
  scanId: string;
  root: string;
  scannedAt: string;
  durationMs: number;
  totalBytes: number;
  totalFiles: number;
  totalDirs: number;
  freeBytes: number;
  driveTotalBytes: number;
  topExtensions: ExtensionStat[];
  largestFolders: FolderStat[];
  largestFiles: FileStat[];
  ageDistribution: AgeDistribution;
  errors: string[];
}

export interface DiskAnalysisResult {
  report: DiskAnalysisReport;
  root: TreemapNode;
}

export interface DiskScanProgressPayload {
  scanId: string;
  bytesScanned: number;
  filesScanned: number;
  dirsScanned: number;
  currentPath: string;
  elapsedMs: number;
}

export interface CacheLocation {
  id: string;
  displayName: string;
  path: string;
  category?: string | null;
  strategy?: string | null;
  lockedBy?: string[];
  requiresAdmin?: boolean;
  risk?: string | null;
  preconditions?: string[];
  filters?: {
    olderThanDays?: number;
    exclude?: string[];
    include?: string[];
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

// ── Cache Engine v2: CleanPlan ───────────────────────────────────────────

export type CleanStrategy =
  | "direct-delete"
  | "uwp-app-aware"
  | "browser-aware"
  | "process-locked"
  | "system-restart-required"
  | "take-ownership-and-delete";

export interface ReadyLocation {
  id: string;
  displayName: string;
  resolvedPath: string;
  bytes: number;
  fileCount: number;
  strategy: CleanStrategy;
  ageOldestFile: string | null;
}

export type BlockedAction =
  | { kind: "closeProcess"; pid: number; processName: string }
  | { kind: "scheduleReboot" }
  | { kind: "skipOnly"; reason: string };

export interface BlockedLocation {
  id: string;
  displayName: string;
  resolvedPath: string;
  bytes: number;
  lockedBy: LockingProcess[];
  suggestedAction: BlockedAction;
}

export interface PermissionLocation {
  id: string;
  displayName: string;
  resolvedPath: string;
  bytes: number;
  reason: string;
}

export type SkipReason =
  | "does-not-exist"
  | "empty"
  | "disallowed-by-allowlist"
  | { precondition: { name: string } };

export interface SkippedLocation {
  id: string;
  displayName: string;
  reason: SkipReason;
}

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

export interface ExecutePlanOpts {
  planId: string;
  autoCloseBlocking: boolean;
  scheduleBlockedForReboot: boolean;
  dryRun: boolean;
  createRestorePoint: boolean;
  timeoutPerLocationSecs: number;
}

export type LocationStatus = "cleaned" | "partial-reboot" | "skipped" | "failed";

export interface LocationResult {
  id: string;
  status: LocationStatus;
  bytesFreed: number;
  bytesScheduled: number;
  filesDeleted: number;
  filesScheduled: number;
  filesFailed: number;
  error: string | null;
  durationMs: number;
}

export interface CleanReportV2 {
  planId: string;
  runId: string;
  startedAt: string;
  finishedAt: string;
  restorePointSeq: number | null;
  perLocation: LocationResult[];
  totalBytesFreed: number;
  totalBytesScheduledReboot: number;
  totalBytesFailed: number;
  closedProcesses: number[];
}

export interface PendingRename {
  source: string;
  destination: string;
  isDelete: boolean;
}

export interface VerifyReport {
  planId: string;
  verifiedAt: string;
  perLocation: VerifyLocationResult[];
  totalActuallyFreed: number;
  totalStillPresent: number;
}

export interface VerifyLocationResult {
  id: string;
  displayName: string;
  bytesBefore: number;
  bytesAfter: number;
  bytesActuallyFreed: number;
  filesPendingReboot: number;
  successPercent: number;
}

// ── Universal App Inventory ──────────────────────────────────────────────

export type AppxKind = "user" | "provisioned" | "framework" | "bundle";

export type AppSource =
  | { kind: "appxPackage"; fullName: string; familyName: string; appxKind: AppxKind }
  | { kind: "appxProvisioned"; fullName: string }
  | { kind: "win32Uninstaller"; registryKey: string; hive: string }
  | { kind: "steam"; appId: number; libraryPath: string }
  | { kind: "epicGames"; catalogItemId: string; manifestPath: string }
  | { kind: "gog"; gameId: number }
  | { kind: "xbox"; packageFamilyName: string; msstoreId: string | null }
  | { kind: "winget"; id: string };

export type UninstallMethod =
  | { kind: "appxRemove" }
  | { kind: "appxProvisionedRemove" }
  | { kind: "uninstallString"; exe: string; args: string[]; requiresAdmin: boolean }
  | { kind: "quietUninstallString"; exe: string; args: string[]; requiresAdmin: boolean }
  | { kind: "msiUninstall"; productCode: string }
  | { kind: "steamUninstall"; appId: number }
  | { kind: "epicUninstall"; catalogItemId: string }
  | { kind: "gogUninstall"; exe: string }
  | { kind: "noUninstaller" };

export interface CatalogMatch {
  catalogId: string;
  requiresDisclaimer: boolean;
  risk: string;
  category: string;
}

export interface ResidualHints {
  appdataRoaming: string[];
  appdataLocal: string[];
  programdata: string[];
  registryKeys: [string, string][];
  startMenuShortcuts: string[];
  desktopShortcuts: string[];
  scheduledTasks: string[];
  services: string[];
  firewallRules: string[];
}

export interface InstalledApp {
  id: string;
  displayName: string;
  publisher: string | null;
  version: string | null;
  source: AppSource;
  installLocation: string | null;
  installDate: string | null;
  sizeBytes: number | null;
  uninstallMethod: UninstallMethod;
  isSystemCritical: boolean;
  catalogMatch: CatalogMatch | null;
  residualHints: ResidualHints;
}

export interface UninstallReport {
  appId: string;
  displayName: string;
  dryRun: boolean;
  success: boolean;
  methodUsed: string;
  error: string | null;
  durationMs: number;
}

export interface CleanResidualsReport {
  appId: string;
  dryRun: boolean;
  pathsDeleted: string[];
  registryKeysDeleted: [string, string][];
  shortcutsDeleted: string[];
  errors: string[];
}

export interface UninstallCompleteReport {
  appId: string;
  displayName: string;
  dryRun: boolean;
  restorePointSeq: number | null;
  uninstall: UninstallReport;
  residuals: CleanResidualsReport;
  auditRunId: string;
}

export interface SelectedResiduals {
  appdataRoaming: string[];
  appdataLocal: string[];
  programdata: string[];
  registryKeys: [string, string][];
  shortcuts: string[];
}

// ── Startup Manager ────────────────────────────────────────────────────

export type StartupImpact = "unknown" | "low" | "medium" | "high";
export type StartupCategory =
  | "updater"
  | "launcher"
  | "widget"
  | "cloud-sync"
  | "communication"
  | "media"
  | "security"
  | "driver"
  | "user-app"
  | "system"
  | "unknown";

export type StartupOrigin =
  | { kind: "registry"; hive: string; key: string; name: string }
  | { kind: "startupFolder"; lnkPath: string }
  | { kind: "scheduledTask"; taskPath: string }
  | { kind: "service"; serviceName: string }
  | { kind: "uwpAutoStart"; packageFamilyName: string; taskId: string };

export interface StartupEntry {
  id: string;
  origin: StartupOrigin;
  displayName: string;
  command: string;
  exePath: string | null;
  iconPath: string | null;
  publisher: string | null;
  signatureValid: boolean | null;
  impact: StartupImpact;
  lastModified: string | null;
  enabled: boolean;
  category: StartupCategory;
}
