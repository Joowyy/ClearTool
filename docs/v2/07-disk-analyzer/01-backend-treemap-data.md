# Paso 01 — Backend: `build_treemap_data`

**Área**: 07-disk-analyzer
**Tiempo estimado**: 3-4 horas
**Dependencias**: ninguna

## Qué hacemos

Implementar el comando que devuelve el árbol entero como `TreemapNode` serializable.

## Archivos

- `src-tauri/src/models/treemap.rs` (nuevo)
- `src-tauri/src/domain/disk_analyzer.rs` (nuevo)
- `src-tauri/src/ipc/disk_analyzer.rs` (nuevo)

## Cómo

### 1. Modelo

```rust
// src-tauri/src/models/treemap.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreemapNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub children: Vec<TreemapNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTreemapInput {
    pub root: String,
    pub max_depth: u32,        // default 4
    pub min_size_bytes: u64,   // default 10 MB
    pub follow_reparse_points: bool,
}
```

### 2. Build recursivo

```rust
// src-tauri/src/domain/disk_analyzer.rs
use crate::core::{AppError, AppResult};
use crate::models::treemap::{TreemapNode, BuildTreemapInput};
use std::path::Path;

pub fn build_treemap(input: &BuildTreemapInput) -> AppResult<TreemapNode> {
    let root_path = Path::new(&input.root);
    if !root_path.exists() {
        return Err(AppError::Io(format!("ruta no existe: {}", input.root)));
    }
    Ok(build_recursive(root_path, input.max_depth, input.min_size_bytes, input.follow_reparse_points, 0))
}

fn build_recursive(
    path: &Path,
    max_depth: u32,
    min_size: u64,
    follow_reparse: bool,
    current_depth: u32,
) -> TreemapNode {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    if !path.is_dir() || current_depth >= max_depth {
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        return TreemapNode {
            name,
            path: path.display().to_string(),
            size_bytes: size,
            is_dir: path.is_dir(),
            extension: path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()),
            children: Vec::new(),
        };
    }

    // Si es dir, recurse en hijos
    let mut children = Vec::new();
    let mut total_size: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        // Skip reparse points si no follow_reparse
        for entry in entries.filter_map(|r| r.ok()) {
            let entry_path = entry.path();

            // Detectar reparse point
            if !follow_reparse {
                if let Ok(md) = entry_path.symlink_metadata() {
                    if md.file_type().is_symlink() { continue; }
                }
            }

            let child_node = build_recursive(
                &entry_path,
                max_depth,
                min_size,
                follow_reparse,
                current_depth + 1,
            );
            total_size += child_node.size_bytes;

            // Filtrar hijos pequeños SI son archivos (los dirs grandes pueden tener muchos archivos pequeños)
            if child_node.size_bytes >= min_size || child_node.is_dir {
                children.push(child_node);
            }
        }
    }

    // Sort por tamaño desc
    children.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    TreemapNode {
        name,
        path: path.display().to_string(),
        size_bytes: total_size,
        is_dir: true,
        extension: None,
        children,
    }
}
```

### 3. IPC

```rust
// src-tauri/src/ipc/disk_analyzer.rs
use crate::core::AppResult;
use crate::domain::disk_analyzer;
use crate::models::treemap::{BuildTreemapInput, TreemapNode};

#[tauri::command]
pub async fn build_treemap_data(input: BuildTreemapInput) -> AppResult<TreemapNode> {
    disk_analyzer::build_treemap(&input)
}
```

Registrar en `lib.rs`.

### 4. Performance considerations

- `C:\` completo con depth=4 puede tener 100k+ nodos. Aplicar `min_size_bytes = 10MB` filtra muy bien.
- Tarda 10-30s en SSD para `C:\` depth 4 con filtros. Aceptable como operación one-shot.
- En el futuro: emitir eventos `treemap:progress` para mostrar progress bar mientras escanea.

### 5. Cancelación

Si el usuario navega fuera mientras escanea, hay que cancelar. Para v1.0, omitir (la operación termina sola en background y se descarta el result). Para v1.1, añadir `CancellationToken`.

## Criterio de done

- [ ] `build_treemap` devuelve árbol completo para `C:\Users\` con depth 3.
- [ ] `total_size` por nodo es correcto (suma de hijos).
- [ ] Sort por tamaño desc.
- [ ] Reparse points respetados con flag.
- [ ] `min_size_bytes` filtra correctamente archivos pequeños.
- [ ] IPC command registrado.
