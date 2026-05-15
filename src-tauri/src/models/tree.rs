// models/tree.rs — árbol de directorios para el explorador.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeStrategy {
    Logical,
    Physical,
    Lazy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Dir,
    File,
    Symlink,
    Junction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTreeInput {
    pub root: String,
    pub max_depth: u32,
    pub follow_reparse_points: bool,
    pub include_hidden: bool,
    pub min_size_bytes: Option<u64>,
    pub size_strategy: SizeStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTreeHandle {
    pub scan_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub path: String,
    pub name: String,
    pub kind: NodeKind,
    pub size_bytes: u64,
    pub last_modified: Option<String>,
    pub children_count: Option<u32>,
    pub is_protected: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorySize {
    pub path: String,
    pub logical_bytes: u64,
    pub physical_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
}
