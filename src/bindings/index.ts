// Tipos TypeScript que reflejan los structs Rust del backend (src-tauri/src/models/)

export type Risk = "Low" | "Medium" | "High" | "low" | "medium" | "high";

export interface DriveInfo {
  letter: string;
  total_bytes: number;
  free_bytes: number;
  label: string;
}

export interface SystemSummary {
  os_name: string;
  os_version: string;
  build_number: string;
  username: string;
  is_elevated: boolean;
  total_ram_bytes: number;
  drives: DriveInfo[];
}

export type SizeStrategy = "Logical" | "Physical" | "Lazy";
export type NodeKind = "Dir" | "File" | "Symlink" | "Junction";

export interface ScanTreeInput {
  root: string;
  max_depth: number;
  follow_reparse_points: boolean;
  include_hidden: boolean;
  min_size_bytes: number | null;
  size_strategy: SizeStrategy;
}

export interface ScanTreeHandle {
  scan_id: string;
}

export interface TreeNode {
  path: string;
  name: string;
  kind: NodeKind;
  size_bytes: number;
  last_modified: string | null;
  children_count: number | null;
  is_protected: boolean;
  error: string | null;
}

export interface DirectorySize {
  path: string;
  logical_bytes: number;
  physical_bytes: number;
  file_count: number;
  dir_count: number;
}

export interface CacheLocation {
  id: string;
  display_name: string;
  category: string;
  path_template: string;
  requires_admin: boolean;
  risk: string;
  consequences: string[];
  average_size: string | null;
}

export interface CacheScanReport {
  id: string;
  resolved_path: string;
  exists: boolean;
  bytes: number;
  file_count: number;
  matched_after_filters: number;
  bytes_after_filters: number;
}

export interface CleanCacheInput {
  ids: string[];
  dry_run: boolean;
  create_restore_point: boolean;
  force_close_processes: boolean;
}

export interface PerLocationResult {
  id: string;
  status: string;
  bytes_freed: number;
  files_deleted: number;
  errors: string[];
}

export interface CleanReport {
  run_id: string;
  started_at: string;
  finished_at: string;
  restore_point_id: number | null;
  per_location: PerLocationResult[];
  total_bytes_freed: number;
  total_files_deleted: number;
}

export interface BloatwareEntry {
  id: string;
  display_name: string;
  category: string;
  removal_strategy: string;
  package_names: string[];
  services: string[];
  scheduled_tasks: string[];
  risk: string;
  consequences: string[];
  reversal_method: string;
  reversal_details: string;
  requires_elevation: boolean;
}

export interface DetectedPackage {
  entry_id: string;
  installed_for_user: boolean;
  installed_all_users: boolean;
  provisioned: boolean;
  package_full_name: string | null;
  install_location: string | null;
  size_estimate_bytes: number | null;
}

export interface RemoveBloatwareInput {
  entry_ids: string[];
  dry_run: boolean;
  create_restore_point: boolean;
  apply_policies: boolean;
  disable_services: boolean;
}

export interface StepLog {
  kind: string;
  target: string;
  ok: boolean;
  stderr: string | null;
  duration_ms: number;
}

export interface PerEntryResult {
  entry_id: string;
  status: string;
  steps: StepLog[];
}

export interface RemoveReport {
  run_id: string;
  started_at: string;
  finished_at: string;
  restore_point_id: number | null;
  per_entry: PerEntryResult[];
  total_removed: number;
  total_skipped: number;
  total_failed: number;
}

export interface Service {
  name: string;
  display_name: string;
  description: string | null;
  state: string;
  start_type: string;
  pid: number | null;
  can_stop: boolean;
  can_pause: boolean;
}

export interface RegistryTweak {
  id: string;
  display_name: string;
  description: string;
  category: string;
  hive: string;
  path: string;
  value_name: string;
  enabled_value: unknown;
  disabled_value: unknown;
  requires_reboot: boolean;
  risk: string;
  consequences: string[];
}

export interface TweakState {
  id: string;
  is_enabled: boolean | null;
  current_raw: unknown;
}

export interface ApplyTweakInput {
  id: string;
  enable: boolean;
  dry_run: boolean;
}

export interface RestorePoint {
  sequence_number: number;
  description: string;
  restore_point_type: number;
  event_type: number;
  created_at: string;
}

export interface CreateRestorePointInput {
  description: string;
}

export interface RestoreReport {
  run_id: string;
  sequence_number: number;
  success: boolean;
  message: string;
}

export interface AuditEntry {
  run_id: string;
  timestamp: string;
  module: string;
  operation: string;
  restore_point_id: number | null;
  items_count: number;
  success: boolean;
  reverse_recipe: string | null;
}
