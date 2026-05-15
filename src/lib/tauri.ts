import { invoke } from "@tauri-apps/api/core";
import type {
  SystemSummary,
  ScanTreeInput,
  ScanTreeHandle,
  DirectorySize,
  CacheLocation,
  CacheScanReport,
  CleanCacheInput,
  CleanReport,
  BloatwareEntry,
  DetectedPackage,
  RemoveBloatwareInput,
  RemoveReport,
  Service,
  RegistryTweak,
  TweakState,
  ApplyTweakInput,
  RestorePoint,
  CreateRestorePointInput,
  RestoreReport,
  AuditEntry,
} from "../bindings";

export const isElevated = () => invoke<boolean>("is_elevated");
export const systemSummary = () => invoke<SystemSummary>("system_summary");

export const scanTree = (input: ScanTreeInput) =>
  invoke<ScanTreeHandle>("scan_tree", { input });
export const cancelScan = (handle: ScanTreeHandle) =>
  invoke<void>("cancel_scan", { handle });
export const computeDirectorySize = (path: string, followReparsePints: boolean) =>
  invoke<DirectorySize>("compute_directory_size", { path, followReparsePints });

export const listCacheLocations = () =>
  invoke<CacheLocation[]>("list_cache_locations");
export const scanCacheLocations = (ids: string[]) =>
  invoke<CacheScanReport[]>("scan_cache_locations", { ids });
export const cleanCacheLocations = (input: CleanCacheInput) =>
  invoke<CleanReport>("clean_cache_locations", { input });

export const listBloatwareCatalog = () =>
  invoke<BloatwareEntry[]>("list_bloatware_catalog");
export const detectInstalledBloatware = () =>
  invoke<DetectedPackage[]>("detect_installed_bloatware");
export const removeBloatware = (input: RemoveBloatwareInput) =>
  invoke<RemoveReport>("remove_bloatware", { input });

export const listServices = () => invoke<Service[]>("list_services");
export const setServiceState = (name: string, startType: string, dryRun: boolean) =>
  invoke<void>("set_service_state", { name, startType, dryRun });
export const applyServicePreset = (preset: string, dryRun: boolean) =>
  invoke<void>("apply_service_preset", { preset, dryRun });

export const listRegistryTweaks = () => invoke<RegistryTweak[]>("list_registry_tweaks");
export const readRegistryTweakState = (id: string) =>
  invoke<TweakState>("read_registry_tweak_state", { id });
export const applyRegistryTweak = (input: ApplyTweakInput) =>
  invoke<void>("apply_registry_tweak", { input });
export const applyRegistryTweakBatch = (inputs: ApplyTweakInput[]) =>
  invoke<void>("apply_registry_tweak_batch", { inputs });
export const revertRegistryTweak = (id: string) =>
  invoke<void>("revert_registry_tweak", { id });

export const ensureRestoreEnabled = () => invoke<boolean>("ensure_restore_enabled");
export const createRestorePoint = (input: CreateRestorePointInput) =>
  invoke<RestoreReport>("create_restore_point", { input });
export const listRestorePoints = () => invoke<RestorePoint[]>("list_restore_points");
export const restoreToPoint = (sequenceNumber: number) =>
  invoke<void>("restore_to_point", { sequenceNumber });

export const listAuditLog = () => invoke<AuditEntry[]>("list_audit_log");
export const revertAuditEntry = (runId: string) =>
  invoke<void>("revert_audit_entry", { runId });
