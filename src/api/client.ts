// Cliente IPC con el backend Rust. Cada función es un envoltorio tipado
// sobre `invoke<T>(name, args)`. El resto del frontend NO debe importar
// `@tauri-apps/api/core` directamente.
//
// En modo web (navegador) se usa un stub que devuelve datos mock.

import type {
  ApplyTweakInput,
  AuditEntry,
  BloatwareEntry,
  BuildTreemapInput,
  CacheLocation,
  CacheScanReport,
  CleanCacheInput,
  CleanPlan,
  CleanReport,
  CleanReportV2,
  CleanResidualsReport,
  CreateRestorePointInput,
  DetectedPackage,
  DirectorySize,
  ExecutePlanOpts,
  InstalledApp,
  LockingProcess,
  PendingRename,
  ProcessInfo,
  RegistryTweak,
  ReleaseReport,
  RemoveBloatwareInput,
  RemoveReport,
  ResidualHints,
  RestorePoint,
  RestoreReport,
  ScanTreeHandle,
  ScanTreeInput,
  SelectedResiduals,
  StartupEntry,
  TreemapNode,
  TreeNode,
  Service,
  Settings,
  SystemSummary,
  TelemetrySnapshot,
  TweakState,
  UninstallCompleteReport,
  UninstallReport,
  VerifyReport,
} from "./types";

type InvokeFn = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

let cachedInvoke: InvokeFn | null = null;
let initPromise: Promise<void> | null = null;

async function initInvoke(): Promise<void> {
  if (cachedInvoke) return;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      cachedInvoke = invoke;
    } catch {
      // Web mode: mock data
      const mocks: Record<string, unknown> = {
        is_elevated: false,
        system_summary: {
          hostname: "DEMO-PC",
          os_version: "Windows 11 Pro (web demo)",
          total_ram_mb: 16384,
          cpu_name: "CPU Demo",
          gpu_names: ["GPU Demo"],
        },
        get_telemetry_snapshot: {
          cpu_usage: 25,
          ram_usage: 45,
          ram_total_mb: 16384,
          ram_used_mb: 7373,
          gpu_usage: 10,
          processes_count: 120,
        },
        scan_tree: {
          handle: "mock-handle",
          root: null,
          total_nodes: 0,
          scanned_nodes: 0,
        },
        list_dir: [],
        cancel_scan: undefined,
        compute_directory_size: {
          path: "",
          size_bytes: 0,
          file_count: 0,
        },
        list_cache_locations: [],
        scan_cache_locations: [],
        clean_cache_locations: {
          cleaned: 0,
          errors: 0,
          total_bytes: 0,
        },
        analyze_cache_locations: {
          planId: "",
          generatedAt: "",
          ready: [],
          blocked: [],
          permissionIssues: [],
          skipped: [],
          totalEstimatedBytes: 0,
          totalBlockedBytes: 0,
        },
        get_cache_plan_warm: null,
        refresh_cache_plan: {
          planId: "",
          generatedAt: "",
          ready: [],
          blocked: [],
          permissionIssues: [],
          skipped: [],
          totalEstimatedBytes: 0,
          totalBlockedBytes: 0,
        },
        execute_clean_plan: {
          planId: "",
          runId: "",
          startedAt: "",
          finishedAt: "",
          restorePointSeq: null,
          perLocation: [],
          totalBytesFreed: 0,
          totalBytesScheduledReboot: 0,
          totalBytesFailed: 0,
          closedProcesses: [],
        },
        verify_clean: {
          planId: "",
          verifiedAt: "",
          perLocation: [],
          totalActuallyFreed: 0,
          totalStillPresent: 0,
        },
        list_bloatware_catalog: [],
        detect_installed_bloatware: [],
        remove_bloatware: {
          removed: 0,
          errors: 0,
        },
        list_services: [],
        set_service_state: undefined,
        apply_service_preset: undefined,
        list_registry_tweaks: [],
        read_registry_tweak_state: {
          id: "",
          applied: false,
          current_value: null,
        },
        apply_registry_tweak: undefined,
        apply_registry_tweak_batch: undefined,
        revert_registry_tweak: undefined,
        ensure_restore_enabled: true,
        create_restore_point: {
          sequence_number: 0,
          description: "",
        },
        list_restore_points: [],
        restore_to_point: undefined,
        list_audit_log: [],
        revert_audit_entry: undefined,
        get_settings: {
          settingsVersion: 1,
          appearance: { theme: "dark", language: "system", density: "normal" },
          safety: { dryRunGlobal: false, autoCreateRestorePoint: true, requireConfirmBeforeBatch: true, bypassThrottlingRestore: true },
          behavior: { checkUpdatesOnStart: true, lastSeenAuditRunId: null, rememberWindowSize: true },
          advanced: { logLevel: "info", auditLogMaxMb: 10, diagnosticMode: false },
        },
        update_settings: undefined,
        reset_settings_to_defaults: undefined,
        settings_file_path: "",
        open_settings_file: undefined,
        relaunch_as_admin: false,
        app_version: { version: "0.1.0", buildDate: "unknown", gitCommit: null },
        list_processes: [],
        kill_process: undefined,
        kill_process_tree: undefined,
        suspend_process: undefined,
        resume_process: undefined,
        close_gracefully: true,
        who_locks_path: [],
        release_caches: {
          closedCount: 0,
          failedCount: 0,
          closedProcesses: [],
        },
        list_startup: [],
        disable_startup: undefined,
        enable_startup: undefined,
        build_treemap_data: {
          name: "C:\\",
          path: "C:\\",
          sizeBytes: 0,
          kind: "Dir",
          extension: null,
          children: [],
        },
        flush_dns: undefined,
        renew_ip: undefined,
        reset_winsock: undefined,
        reset_tcpip: undefined,
        reset_proxy: undefined,
        restore_hosts_file: undefined,
        list_pending_renames: [],
        cancel_pending_rename: undefined,
        clear_all_pending_renames: 0,
        list_installed_apps: [],
        compute_residual_hints: { appdataRoaming: [], appdataLocal: [], programdata: [], registryKeys: [], startMenuShortcuts: [], desktopShortcuts: [], scheduledTasks: [], services: [], firewallRules: [] },
        uninstall_app: { appId: "", displayName: "", dryRun: true, success: false, methodUsed: "", error: null, durationMs: 0 },
        clean_residuals: { appId: "", dryRun: true, pathsDeleted: [], registryKeysDeleted: [], shortcutsDeleted: [], errors: [] },
        uninstall_app_complete: { appId: "", displayName: "", dryRun: true, restorePointSeq: null, uninstall: { appId: "", displayName: "", dryRun: true, success: false, methodUsed: "", error: null, durationMs: 0 }, residuals: { appId: "", dryRun: true, pathsDeleted: [], registryKeysDeleted: [], shortcutsDeleted: [], errors: [] }, auditRunId: "" },
      };

      cachedInvoke = async <T>(command: string, _args?: Record<string, unknown>): Promise<T> => {
        if (command in mocks) return mocks[command] as T;
        throw new Error(`Unknown command in web mode: ${command}`);
      };
    }
  })();

  return initPromise;
}

// Initialize immediately
void initInvoke();

const invoke: InvokeFn = async (command, args) => {
  await initInvoke();
  return cachedInvoke!(command, args);
};

// ── system_info ──────────────────────────────────────────────────────────
export const isElevated = () => invoke<boolean>("is_elevated");
export const systemSummary = () => invoke<SystemSummary>("system_summary");
export const relaunchAsAdmin = () => invoke<boolean>("relaunch_as_admin");
export const appVersion = () => invoke<{ version: string; buildDate: string; gitCommit: string | null }>("app_version");

// ── telemetry (CPU/RAM/GPU/procesos) ─────────────────────────────────────
export const getTelemetrySnapshot = () =>
  invoke<TelemetrySnapshot>("get_telemetry_snapshot");

// ── explorer ────────────────────────────────────────────────────────────
export const scanTree = (input: ScanTreeInput) =>
  invoke<ScanTreeHandle>("scan_tree", { input });

export const listDir = (path: string, followReparsePoints = false) =>
  invoke<TreeNode[]>("list_dir", { path, followReparsePoints });

export const cancelScan = (handle: ScanTreeHandle) =>
  invoke<void>("cancel_scan", { handle });

export const computeDirectorySize = (path: string, followReparsePoints: boolean) =>
  invoke<DirectorySize>("compute_directory_size", { path, followReparsePoints });

// ── cache ───────────────────────────────────────────────────────────────
export const listCacheLocations = () =>
  invoke<CacheLocation[]>("list_cache_locations");

export const scanCacheLocations = (ids: string[]) =>
  invoke<CacheScanReport[]>("scan_cache_locations", { ids });

export const cleanCacheLocations = (input: CleanCacheInput) =>
  invoke<CleanReport>("clean_cache_locations", { input });

// ── cache v2: analyze, execute_plan, verify ─────────────────────────────
export const analyzeCacheLocations = (ids: string[]) =>
  invoke<CleanPlan>("analyze_cache_locations", { ids });

export const executeCleanPlan = (plan: CleanPlan, opts: ExecutePlanOpts) =>
  invoke<CleanReportV2>("execute_clean_plan", { plan, opts });

export const verifyClean = (plan: CleanPlan, report: CleanReportV2) =>
  invoke<VerifyReport>("verify_clean", { plan, report });

export const getCachePlanWarm = () =>
  invoke<CleanPlan | null>("get_cache_plan_warm");

export const refreshCachePlan = () =>
  invoke<CleanPlan>("refresh_cache_plan");

// ── debloat ─────────────────────────────────────────────────────────────
export const listBloatwareCatalog = () =>
  invoke<BloatwareEntry[]>("list_bloatware_catalog");

export const detectInstalledBloatware = () =>
  invoke<DetectedPackage[]>("detect_installed_bloatware");

export const removeBloatware = (input: RemoveBloatwareInput) =>
  invoke<RemoveReport>("remove_bloatware", { input });

// ── services ────────────────────────────────────────────────────────────
export const listServices = () => invoke<Service[]>("list_services");

export const setServiceState = (name: string, startType: string, dryRun: boolean) =>
  invoke<void>("set_service_state", { name, startType, dryRun });

export const applyServicePreset = (preset: string, dryRun: boolean) =>
  invoke<void>("apply_service_preset", { preset, dryRun });

// ── registry ────────────────────────────────────────────────────────────
export const listRegistryTweaks = () =>
  invoke<RegistryTweak[]>("list_registry_tweaks");

export const readRegistryTweakState = (id: string) =>
  invoke<TweakState>("read_registry_tweak_state", { id });

export const applyRegistryTweak = (input: ApplyTweakInput) =>
  invoke<void>("apply_registry_tweak", { input });

export const applyRegistryTweakBatch = (inputs: ApplyTweakInput[]) =>
  invoke<void>("apply_registry_tweak_batch", { inputs });

export const revertRegistryTweak = (id: string) =>
  invoke<void>("revert_registry_tweak", { id });

// ── restore ─────────────────────────────────────────────────────────────
export const ensureRestoreEnabled = () => invoke<boolean>("ensure_restore_enabled");

export const createRestorePoint = (input: CreateRestorePointInput) =>
  invoke<RestoreReport>("create_restore_point", { input });

export const listRestorePoints = () => invoke<RestorePoint[]>("list_restore_points");

export const restoreToPoint = (sequenceNumber: number) =>
  invoke<void>("restore_to_point", { sequenceNumber });

// ── audit ───────────────────────────────────────────────────────────────
export const listAuditLog = () => invoke<AuditEntry[]>("list_audit_log");

export const revertAuditEntry = (runId: string) =>
  invoke<void>("revert_audit_entry", { runId });

// ── settings ─────────────────────────────────────────────────────────────
export const getSettings = () => invoke<Settings>("get_settings");

export const updateSettings = (settings: Settings) =>
  invoke<void>("update_settings", { updated: settings });

export const resetSettingsToDefaults = () => invoke<Settings>("reset_settings_to_defaults");

export const settingsFilePath = () => invoke<string>("settings_file_path");

export const openSettingsFile = () => invoke<void>("open_settings_file");

// ── processes ─────────────────────────────────────────────────────────────
export const listProcesses = () => invoke<ProcessInfo[]>("list_processes");
export const killProcess = (pid: number) => invoke<void>("kill_process", { pid });
export const killProcessTree = (pid: number) => invoke<void>("kill_process_tree", { pid });
export const suspendProcess = (pid: number) => invoke<void>("suspend_process", { pid });
export const resumeProcess = (pid: number) => invoke<void>("resume_process", { pid });
export const closeGracefully = (pid: number, timeoutMs: number) =>
  invoke<boolean>("close_gracefully", { pid, timeoutMs });
export const whoLocksPath = (path: string) =>
  invoke<LockingProcess[]>("who_locks_path", { path });
export const releaseCaches = () => invoke<ReleaseReport>("release_caches");

// ── startup ─────────────────────────────────────────────────────────────
export const listStartup = () => invoke<StartupEntry[]>("list_startup");
export const disableStartup = (id: string) => invoke<void>("disable_startup", { id });
export const enableStartup = (id: string) => invoke<void>("enable_startup", { id });

// ── disk analyzer ───────────────────────────────────────────────────────
export const buildTreemapData = (input: BuildTreemapInput) =>
  invoke<TreemapNode>("build_treemap_data", { input });

// ── network utilities ───────────────────────────────────────────────────
export const flushDns = (dryRun: boolean) => invoke<void>("flush_dns", { dryRun });
export const renewIp = (dryRun: boolean) => invoke<void>("renew_ip", { dryRun });
export const resetWinsock = (dryRun: boolean) => invoke<void>("reset_winsock", { dryRun });
export const resetTcpip = (dryRun: boolean) => invoke<void>("reset_tcpip", { dryRun });
export const resetProxy = (dryRun: boolean) => invoke<void>("reset_proxy", { dryRun });
export const restoreHostsFile = (dryRun: boolean) => invoke<void>("restore_hosts_file", { dryRun });

// ── boot-time cleanup (pending renames) ─────────────────────────────────
export const listPendingRenames = () => invoke<PendingRename[]>("list_pending_renames");
export const cancelPendingRename = (source: string) =>
  invoke<void>("cancel_pending_rename", { source });
export const clearAllPendingRenames = () => invoke<number>("clear_all_pending_renames");

// ── universal app inventory ──────────────────────────────────────────────
export const listInstalledApps = () => invoke<InstalledApp[]>("list_installed_apps");
export const computeResidualHints = (appId: string, app: InstalledApp) =>
  invoke<ResidualHints>("compute_residual_hints", { appId, app });
export const uninstallApp = (app: InstalledApp, dryRun: boolean) =>
  invoke<UninstallReport>("uninstall_app", { app, dryRun });
export const cleanResiduals = (appId: string, selected: SelectedResiduals, dryRun: boolean) =>
  invoke<CleanResidualsReport>("clean_residuals", { appId, selected, dryRun });
export const uninstallAppComplete = (app: InstalledApp, dryRun: boolean) =>
  invoke<UninstallCompleteReport>("uninstall_app_complete", { app, dryRun });
