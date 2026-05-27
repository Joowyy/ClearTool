// domain/disk.rs — Disk Analyzer: construye estructura de treemap.
//
// Reutiliza `filesystem::directory_stats` y `filesystem::list_children_with_sizes`
// para construir un árbol serializable con tamaños por nodo.

use crate::core::AppResult;
use crate::models::disk::TreemapNode;
use crate::models::tree::NodeKind;
use crate::platform::filesystem;
use std::path::Path;

pub fn build_treemap(
    root: &str,
    max_depth: u32,
    min_size_mb: u64,
    follow_reparse_points: bool,
) -> AppResult<TreemapNode> {
    let path = Path::new(root);
    let min_size_bytes = min_size_mb * 1024 * 1024;

    let root_node = build_node_recursive(path, max_depth, min_size_bytes, follow_reparse_points, 0)?;
    Ok(root_node)
}

fn build_node_recursive(
    path: &Path,
    max_depth: u32,
    min_size_bytes: u64,
    follow_reparse_points: bool,
    current_depth: u32,
) -> AppResult<TreemapNode> {
    let meta = path.symlink_metadata().ok();
    let is_symlink = meta.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false);
    let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let extension = if is_dir {
        None
    } else {
        path.extension()
            .map(|e| e.to_string_lossy().to_string().to_lowercase())
    };

    let kind = if is_symlink {
        NodeKind::Symlink
    } else if is_dir {
        NodeKind::Dir
    } else {
        NodeKind::File
    };

    let size_bytes = if is_dir && current_depth < max_depth {
        filesystem::directory_stats(path, follow_reparse_points)
            .map(|(b, _, _)| b)
            .unwrap_or(0)
    } else if !is_dir {
        meta.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let children = if is_dir && current_depth < max_depth {
        let entries = filesystem::list_children_with_sizes(path, follow_reparse_points)
            .unwrap_or_default();

        let mut child_nodes = Vec::new();
        for (child_path, _child_name, is_child_dir, child_size) in entries {
            // Filtrar por tamaño mínimo (solo para directorios o archivos grandes)
            if !is_child_dir && child_size < min_size_bytes {
                continue;
            }

            let child_path_obj = Path::new(&child_path);
            match build_node_recursive(
                child_path_obj,
                max_depth,
                min_size_bytes,
                follow_reparse_points,
                current_depth + 1,
            ) {
                Ok(node) => child_nodes.push(node),
                Err(_) => {
                    // Si falla un hijo, añadir placeholder con error
                    child_nodes.push(TreemapNode {
                        name: _child_name,
                        path: child_path,
                        size_bytes: 0,
                        kind: NodeKind::Dir,
                        extension: None,
                        children: vec![],
                    });
                }
            }
        }
        child_nodes.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        child_nodes
    } else {
        vec![]
    };

    Ok(TreemapNode {
        name,
        path: path.to_string_lossy().to_string(),
        size_bytes,
        kind,
        extension,
        children,
    })
}
