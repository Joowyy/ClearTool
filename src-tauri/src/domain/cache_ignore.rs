// domain/cache_ignore.rs — lista de rutas ignoradas por el usuario.
//
// Persiste en %APPDATA%\ClearTool\cache-ignore.json como ring buffer de 100 entradas.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct IgnoreList {
    paths: Vec<String>,
}

fn ignore_file_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("ClearTool").join("cache-ignore.json"))
}

fn load_raw() -> IgnoreList {
    let path = match ignore_file_path() {
        Some(p) => p,
        None => return IgnoreList::default(),
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return IgnoreList::default(),
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn save(list: &IgnoreList) -> crate::core::AppResult<()> {
    let path = ignore_file_path()
        .ok_or_else(|| crate::core::AppError::External("No se pudo obtener %APPDATA%".into()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(list)?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Añade `path` al ignore list (ring buffer de MAX_ENTRIES).
pub fn add_ignored(path: &str) -> crate::core::AppResult<()> {
    let mut list = load_raw();
    // Dedup.
    if list.paths.iter().any(|p| p == path) {
        return Ok(());
    }
    list.paths.push(path.to_string());
    // Mantener ring buffer.
    if list.paths.len() > MAX_ENTRIES {
        let drain = list.paths.len() - MAX_ENTRIES;
        list.paths.drain(..drain);
    }
    save(&list)
}

/// Devuelve `true` si `path` está en la lista de ignorados.
pub fn is_ignored(path: &str) -> bool {
    let list = load_raw();
    list.paths.iter().any(|p| p == path)
}

/// Devuelve todas las rutas ignoradas.
pub fn list_ignored() -> Vec<String> {
    load_raw().paths
}
