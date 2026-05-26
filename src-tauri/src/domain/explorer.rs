// domain/explorer.rs — exploración del árbol de directorios.
//
// Sirve dos casos:
//   1. `list_top_level(root)` — lista hijos directos con tamaño total por
//      subdirectorio (útil para el panel "qué pesa más en C:\").
//   2. `compute_size(path, follow)` — cómputo recursivo de tamaño.
//
// El streaming por eventos `explorer:node` se conecta desde `ipc/explorer.rs`.

use crate::core::AppResult;
use crate::models::tree::{DirectorySize, NodeKind, TreeNode};
use crate::platform::filesystem;
use std::path::Path;

pub fn list_top_level(root: &str, follow_reparse_points: bool) -> AppResult<Vec<TreeNode>> {
    let path = Path::new(root);
    let children = filesystem::list_children_with_sizes(path, follow_reparse_points)?;

    let nodes = children
        .into_iter()
        .map(|(p, name, is_dir, size)| TreeNode {
            path: p,
            name,
            kind: if is_dir { NodeKind::Dir } else { NodeKind::File },
            size_bytes: size,
            last_modified: None,
            children_count: None,
            is_protected: false,
            error: None,
        })
        .collect();

    Ok(nodes)
}

pub fn compute_size(path: &str, follow_reparse_points: bool) -> AppResult<DirectorySize> {
    let p = Path::new(path);
    let (bytes, files, dirs) = filesystem::directory_stats(p, follow_reparse_points)?;
    Ok(DirectorySize {
        path: path.to_string(),
        logical_bytes: bytes,
        physical_bytes: bytes, // TODO: tamaño físico real (clúster + compression) cuando aplique.
        file_count: files,
        dir_count: dirs,
    })
}
