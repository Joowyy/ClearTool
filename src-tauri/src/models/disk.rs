// models/disk.rs — modelos para Disk Analyzer (treemap).

use serde::{Deserialize, Serialize};

use super::tree::NodeKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreemapNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: NodeKind,
    pub extension: Option<String>,
    pub children: Vec<TreemapNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTreemapInput {
    pub root: String,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default = "default_min_size_mb")]
    pub min_size_mb: u64,
    #[serde(default)]
    pub follow_reparse_points: bool,
}

fn default_max_depth() -> u32 {
    4
}

fn default_min_size_mb() -> u64 {
    10
}
