// ipc/disk.rs — comandos Tauri para Disk Analyzer.

use crate::core::AppResult;
use crate::domain::disk;
use crate::models::disk::{BuildTreemapInput, TreemapNode};

#[tauri::command]
pub async fn build_treemap_data(input: BuildTreemapInput) -> AppResult<TreemapNode> {
    // Validar que el root no sea path traversal
    if input.root.contains("..") {
        return Err(crate::core::AppError::Permission(
            "Invalid root path".to_string(),
        ));
    }

    disk::build_treemap(
        &input.root,
        input.max_depth,
        input.min_size_mb,
        input.follow_reparse_points,
    )
}
