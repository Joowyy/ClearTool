# Paso 01 — Backend: list/cancel pending renames

**Área**: 06-boot-cleanup
**Tiempo estimado**: 1 hora
**Dependencias**: `02-cache-engine/02-pending-rename-helper.md` (ya implementó las funciones)

## Qué hacemos

Si el módulo cache-engine ya implementó `list_pending_renames` y `cancel_pending_rename`, este paso es solo confirmar y añadir helpers extra: contar bytes aproximados de los pendings.

## Archivos

- `src-tauri/src/platform/pending_rename.rs` (añadir helper `count_pending`)
- `src-tauri/src/domain/boot_cleanup.rs` (nuevo, orquestador)

## Cómo

```rust
// src-tauri/src/domain/boot_cleanup.rs

use crate::core::AppResult;
use crate::models::pending_rename::PendingRename;
use crate::platform::pending_rename;

pub fn list_all_pending() -> AppResult<Vec<PendingRename>> {
    pending_rename::list_pending_renames()
}

pub fn cancel_one(source_path: &str) -> AppResult<()> {
    pending_rename::cancel_pending_rename(std::path::Path::new(source_path))?;
    crate::domain::audit::write_entry(&crate::domain::audit::make_entry(
        "boot-cleanup",
        "cancel-pending",
        false,
        None,
        vec![source_path.to_string()],
        crate::models::restore::ReverseRecipe::Noop {
            reason: "El archivo ya no se eliminará en el próximo reboot.".into(),
        },
        "success",
        None,
    ))?;
    Ok(())
}

pub fn cancel_all_matching(prefix: &str) -> AppResult<u32> {
    let mut cancelled = 0u32;
    let all = list_all_pending()?;
    for p in all {
        if p.source.to_lowercase().starts_with(&prefix.to_lowercase()) {
            if pending_rename::cancel_pending_rename(std::path::Path::new(&p.source)).is_ok() {
                cancelled += 1;
            }
        }
    }
    Ok(cancelled)
}

pub fn count_pending() -> u32 {
    list_all_pending().map(|v| v.len() as u32).unwrap_or(0)
}
```

## Criterio de done

- [ ] `list_all_pending` devuelve datos reales del registry.
- [ ] `cancel_one(path)` quita una entrada.
- [ ] `cancel_all_matching(prefix)` quita múltiples.
- [ ] `count_pending` rápido (<50ms).
- [ ] Audit log entry por cada cancel.
