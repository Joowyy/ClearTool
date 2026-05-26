# Paso 02 — IPC + helpers de schedule

**Área**: 06-boot-cleanup
**Tiempo estimado**: 1.5 horas
**Dependencias**: Paso 01

## Qué hacemos

Exponer 4 comandos IPC + helpers TypeScript.

## Archivos

- `src-tauri/src/ipc/boot_cleanup.rs` (nuevo)
- `src-tauri/src/lib.rs` (registrar)
- `src/api/client.ts` (wrappers)

## Cómo

### Rust

```rust
// src-tauri/src/ipc/boot_cleanup.rs
use crate::core::AppResult;
use crate::domain::boot_cleanup;
use crate::models::pending_rename::PendingRename;

#[tauri::command]
pub async fn list_pending_renames() -> AppResult<Vec<PendingRename>> {
    boot_cleanup::list_all_pending()
}

#[tauri::command]
pub async fn cancel_pending_rename(source_path: String) -> AppResult<()> {
    boot_cleanup::cancel_one(&source_path)
}

#[tauri::command]
pub async fn cancel_pending_renames_matching(prefix: String) -> AppResult<u32> {
    boot_cleanup::cancel_all_matching(&prefix)
}

#[tauri::command]
pub async fn count_pending_renames() -> AppResult<u32> {
    Ok(boot_cleanup::count_pending())
}

#[tauri::command]
pub async fn schedule_delete_on_reboot(path: String) -> AppResult<()> {
    crate::platform::pending_rename::schedule_delete_on_reboot(std::path::Path::new(&path))
}
```

Registrar en `lib.rs`.

### TS

```ts
// src/api/client.ts
export const listPendingRenames = () => invoke<PendingRename[]>("list_pending_renames");
export const cancelPendingRename = (sourcePath: string) =>
  invoke<void>("cancel_pending_rename", { sourcePath });
export const cancelPendingMatching = (prefix: string) =>
  invoke<number>("cancel_pending_renames_matching", { prefix });
export const countPendingRenames = () => invoke<number>("count_pending_renames");
export const scheduleDeleteOnReboot = (path: string) =>
  invoke<void>("schedule_delete_on_reboot", { path });
```

## Criterio de done

- [ ] 5 comandos registrados.
- [ ] Wrappers TS funcionan.
- [ ] Permisos: todos requieren admin (verificable porque el registry SYSTEM\... lo requiere).
