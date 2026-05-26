// ipc/explorer.rs — exploración del árbol de directorios.

use crate::core::AppResult;
use crate::domain;
use crate::models::tree::{DirectorySize, ScanTreeHandle, ScanTreeInput};
use uuid::Uuid;

/// Lista hijos directos del `root` con tamaño total por subdirectorio.
/// La UI ya está preparada para escuchar `explorer:node` y `explorer:done`,
/// pero por simplicidad esta primera versión devuelve el resultado completo
/// y emite los eventos al terminar para no romper el contrato.
#[tauri::command]
pub async fn scan_tree(
    app: tauri::AppHandle,
    input: ScanTreeInput,
) -> AppResult<ScanTreeHandle> {
    use tauri::Emitter;

    let scan_id = Uuid::new_v4().to_string();
    let nodes = domain::explorer::list_top_level(&input.root, input.follow_reparse_points)?;

    for node in &nodes {
        let _ = app.emit("explorer:node", node);
    }
    let _ = app.emit("explorer:done", &scan_id);

    Ok(ScanTreeHandle { scan_id })
}

/// Lista los hijos directos de `path` y los devuelve sin emitir eventos.
/// Usado por el frontend para carga lazy al expandir un nodo del árbol.
#[tauri::command]
pub async fn list_dir(
    path: String,
    follow_reparse_points: bool,
) -> AppResult<Vec<crate::models::tree::TreeNode>> {
    domain::explorer::list_top_level(&path, follow_reparse_points)
}

#[tauri::command]
pub async fn cancel_scan(_handle: ScanTreeHandle) -> AppResult<()> {
    // Sin cancelación real todavía: la versión actual es síncrona.
    Ok(())
}

#[tauri::command]
pub async fn compute_directory_size(
    path: String,
    follow_reparse_points: bool,
) -> AppResult<DirectorySize> {
    domain::explorer::compute_size(&path, follow_reparse_points)
}
