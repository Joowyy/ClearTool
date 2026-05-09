---
name: directory-tree-explorer
description: Patrón para recorrer eficientemente el sistema de archivos Windows como árbol con cálculo de tamaños, manejo de reparse points/symlinks/junctions, paths >MAX_PATH, archivos en uso y permisos denegados. Streaming al frontend en lotes para evitar bloquear la UI con árboles grandes.
---

# Skill: directory-tree-explorer

Para el módulo "Explorador" de ClearTool.

## Reglas

1. **No bloquear el thread async de Tauri.** Walks pesados en `tokio::task::spawn_blocking`.
2. **Streaming.** El árbol completo de C:\ puede tener millones de nodos; emite resultados en lotes de 500 por evento, no esperes a terminar.
3. **Manejar errores por nodo, no abortar.** Permission denied en un subdir no aborta el escaneo.
4. **Reparse points.** Por defecto NO los sigues (evita ciclos); el usuario puede activar "Seguir junctions" con warning.
5. **Paths largos.** Usa el prefijo `\\?\` cuando sea necesario para soportar > 260 caracteres.

## Estructura del DTO

```rust
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub path: String,
    pub name: String,
    pub kind: NodeKind,                 // file | directory | reparse-point | unknown
    pub size_bytes: u64,
    pub child_count: Option<u32>,       // None hasta que se expande
    pub last_modified: Option<i64>,     // unix epoch
    pub readable: bool,                 // false si permission denied
    pub is_hidden: bool,
    pub is_system: bool,
}
```

## Patrón con streaming

```rust
// services/filesystem.rs

pub async fn scan_tree(
    root: PathBuf,
    options: ScanOptions,
    app: tauri::AppHandle,
) -> Result<ScanSummary, AppError> {
    validate_within_safe_roots(&root)?;

    tokio::task::spawn_blocking(move || -> Result<_, AppError> {
        let mut batch = Vec::with_capacity(500);
        let mut total_files = 0u64;
        let mut total_bytes = 0u64;

        let walker = walkdir::WalkDir::new(&root)
            .max_depth(options.max_depth.unwrap_or(usize::MAX))
            .follow_links(options.follow_reparse)
            .into_iter()
            .filter_entry(|e| !options.is_excluded(e.path()));

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    // Permission denied / locked: nodo "no legible".
                    if let Some(path) = err.path() {
                        batch.push(TreeNode::unreadable(path));
                    }
                    continue;
                }
            };

            let node = TreeNode::from_entry(&entry);
            total_files += 1;
            total_bytes += node.size_bytes;
            batch.push(node);

            if batch.len() >= 500 {
                app.emit("explorer:batch", &batch).ok();
                batch.clear();
            }
        }

        if !batch.is_empty() {
            app.emit("explorer:batch", &batch).ok();
        }
        Ok(ScanSummary { total_files, total_bytes })
    }).await?
}
```

## Tamaño "real"

Distinguir:

- `size`: tamaño lógico del archivo.
- `size_on_disk`: tamaño asignado por NTFS (cluster size matters), opcional, solo si el usuario lo pide.
- `size_total_recursive`: cálculo bajo demanda al expandir un directorio en la UI.

El cálculo recursivo NO se hace en el primer scan (caro); se hace cuando el usuario expande un nodo.

## Reparse points

```rust
fn is_reparse_point(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}
```

## Atributos hidden/system

```rust
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;

fn is_hidden(meta: &std::fs::Metadata) -> bool {
    meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0
}
```

## Paths largos

En Rust, `std::fs` soporta paths largos pero algunas APIs Win32 requieren prefijo `\\?\`. Usa `dunce::simplified` para presentar al usuario sin prefijo, mantén interno con prefijo cuando profundices.

## Frontend

```ts
import { listen } from '@tauri-apps/api/event';
import type { TreeNode } from '@/bindings';

listen<TreeNode[]>('explorer:batch', (e) => {
  store.appendNodes(e.payload);
});
```

UI usa `react-arborist` o `@tanstack/react-virtual` para virtualizar (1M nodos no se renderizan a la vez).

## Performance objetivos

- Escaneo de C:\ a profundidad 3: < 5s en SSD.
- Escaneo recursivo total de C:\Users\<user>: < 60s para 100k archivos.
- RAM consumida: lineal con # nodos visibles, no con # nodos totales (gracias a streaming).
