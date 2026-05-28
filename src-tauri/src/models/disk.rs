// models/disk.rs — modelos para Disk Analyzer (treemap + stats).
//
// Antes vivía aquí solo `TreemapNode` y `BuildTreemapInput`. Tras el
// borrado del módulo Explorer, también absorbe `NodeKind` (que era
// común a Explorer y Disk; ahora solo lo usa este módulo).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeKind {
    Dir,
    File,
    Symlink,
    Junction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DriveType {
    Fixed,
    Removable,
    Network,
    CdRom,
    RamDisk,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveListing {
    pub letter: String,
    pub root_path: String,
    pub label: String,
    pub filesystem: String,
    pub drive_type: DriveType,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub is_ready: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ExtCategory {
    Media,
    Image,
    Code,
    Docs,
    Archive,
    Executable,
    Database,
    Font,
    #[serde(rename = "threeD")]
    ThreeD,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreemapNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: NodeKind,
    pub extension: Option<String>,
    pub file_count: u64,
    pub dir_count: u64,
    pub last_modified: Option<String>,
    pub percent_of_parent: f32,
    pub percent_of_root: f32,
    pub children: Vec<TreemapNode>,
    pub truncated: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTreemapInput {
    pub root: String,
    #[serde(default = "default_max_depth_emit")]
    pub max_depth_emit: u32,
    #[serde(default = "default_min_size_mb")]
    pub min_size_mb: u64,
    #[serde(default)]
    pub follow_reparse_points: bool,
    #[serde(default = "default_true")]
    pub include_hidden: bool,
    #[serde(default = "default_true")]
    pub include_system: bool,
    #[serde(default)]
    pub scan_id: String,
}

fn default_max_depth_emit() -> u32 {
    6
}

fn default_min_size_mb() -> u64 {
    10
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionStat {
    pub extension: String,
    pub category: ExtCategory,
    pub bytes: u64,
    pub file_count: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderStat {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub file_count: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStat {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub extension: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgeBucket {
    pub bytes: u64,
    pub file_count: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgeDistribution {
    pub last_7_days: AgeBucket,
    pub last_30_days: AgeBucket,
    pub last_90_days: AgeBucket,
    pub last_1_year: AgeBucket,
    pub last_5_years: AgeBucket,
    pub older: AgeBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskAnalysisReport {
    pub scan_id: String,
    pub root: String,
    pub scanned_at: String,
    pub duration_ms: u64,
    pub total_bytes: u64,
    pub total_files: u64,
    pub total_dirs: u64,
    pub free_bytes: u64,
    pub drive_total_bytes: u64,
    pub top_extensions: Vec<ExtensionStat>,
    pub largest_folders: Vec<FolderStat>,
    pub largest_files: Vec<FileStat>,
    pub age_distribution: AgeDistribution,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskAnalysisResult {
    pub report: DiskAnalysisReport,
    pub root: TreemapNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskScanProgressPayload {
    pub scan_id: String,
    pub bytes_scanned: u64,
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub current_path: String,
    pub elapsed_ms: u64,
}
