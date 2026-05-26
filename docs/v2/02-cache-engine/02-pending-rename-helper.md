# Paso 02 — Helper `PendingFileRenameOperations`

**Área**: 02-cache-engine
**Tiempo estimado**: 2-3 horas
**Dependencias**: ninguna

## Qué hacemos

Crear un módulo `platform::pending_rename` con tres funciones:
- `schedule_delete_on_reboot(path)` — programar borrado.
- `list_pending_renames()` — listar todos los pendientes.
- `cancel_pending_rename(path)` — eliminar uno concreto de la lista.

## Por qué

El v0.1 usa `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` directamente. Esto funciona pero es opaco: no podemos listar pendientes, no podemos cancelar, no podemos verificar si añadió bien. Necesitamos un wrapper limpio sobre la API + lectura/escritura directa del registry para visibility.

## Archivos que tocamos

- `src-tauri/src/platform/pending_rename.rs` (nuevo)
- `src-tauri/src/platform/mod.rs` (export)
- `src-tauri/src/models/pending_rename.rs` (nuevo, DTO)

## Cómo

### 1. Modelo

```rust
// src-tauri/src/models/pending_rename.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingRename {
    pub source: String,
    pub destination: String,  // vacío = delete; presente = rename
    pub is_delete: bool,      // shortcut: destination está vacío
}
```

### 2. Implementación

```rust
// src-tauri/src/platform/pending_rename.rs
use crate::core::{AppError, AppResult};
use crate::models::pending_rename::PendingRename;
use std::path::Path;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT,
    };
    use winreg::enums::*;
    use winreg::RegKey;

    const SESSION_MGR_KEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager";
    const PFRO_VAL: &str = "PendingFileRenameOperations";

    /// Programa un archivo para eliminarse en el próximo reboot.
    /// El borrado se ejecuta como SYSTEM antes de que ningún proceso de usuario arranque.
    pub fn schedule_delete_on_reboot(path: &Path) -> AppResult<()> {
        let path_wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let ok = unsafe {
            MoveFileExW(
                PCWSTR(path_wide.as_ptr()),
                PCWSTR::null(),
                MOVEFILE_DELAY_UNTIL_REBOOT,
            )
        };

        if ok.is_ok() {
            Ok(())
        } else {
            Err(AppError::Io(format!(
                "programando borrado para {}: error {:?}",
                path.display(),
                unsafe { windows::Win32::Foundation::GetLastError() }
            )))
        }
    }

    /// Lee la lista actual de pending renames del registry.
    /// Cada entry son DOS strings consecutivos (source + destination).
    /// Destination vacío = delete; con `\??\` prefix = move.
    pub fn list_pending_renames() -> AppResult<Vec<PendingRename>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = match hklm.open_subkey(SESSION_MGR_KEY) {
            Ok(k) => k,
            Err(e) => {
                return Err(AppError::Registry(format!(
                    "abriendo {}: {}",
                    SESSION_MGR_KEY, e
                )))
            }
        };

        // REG_MULTI_SZ — array de strings null-terminated
        let raw: Vec<String> = match key.get_value(PFRO_VAL) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),  // no key = no pendings
        };

        let mut out = Vec::new();
        let mut iter = raw.into_iter();
        while let Some(source) = iter.next() {
            let destination = iter.next().unwrap_or_default();
            let is_delete = destination.is_empty();

            // Limpiar prefijos NT
            let source_clean = strip_nt_prefix(&source).to_string();
            let dest_clean = if is_delete {
                String::new()
            } else {
                strip_nt_prefix(&destination).to_string()
            };

            out.push(PendingRename {
                source: source_clean,
                destination: dest_clean,
                is_delete,
            });
        }
        Ok(out)
    }

    /// Quita una entry concreta de la lista (re-escribe el valor sin esa fila).
    pub fn cancel_pending_rename(source_path: &Path) -> AppResult<()> {
        let target_normalized = normalize_path(source_path);
        let current = list_pending_renames()?;
        let filtered: Vec<PendingRename> = current
            .into_iter()
            .filter(|p| normalize_str_path(&p.source) != target_normalized)
            .collect();

        write_pending_renames(&filtered)
    }

    fn write_pending_renames(entries: &[PendingRename]) -> AppResult<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm
            .create_subkey(SESSION_MGR_KEY)
            .map_err(|e| AppError::Registry(format!("create {}: {}", SESSION_MGR_KEY, e)))?;

        if entries.is_empty() {
            key.delete_value(PFRO_VAL)
                .map_err(|e| AppError::Registry(format!("delete {}: {}", PFRO_VAL, e)))?;
            return Ok(());
        }

        let mut multi: Vec<String> = Vec::with_capacity(entries.len() * 2);
        for e in entries {
            multi.push(add_nt_prefix(&e.source));
            multi.push(if e.is_delete { String::new() } else { add_nt_prefix(&e.destination) });
        }
        key.set_value(PFRO_VAL, &multi)
            .map_err(|e| AppError::Registry(format!("set {}: {}", PFRO_VAL, e)))?;
        Ok(())
    }

    fn strip_nt_prefix(s: &str) -> &str {
        s.strip_prefix(r"\??\").unwrap_or(s)
    }

    fn add_nt_prefix(s: &str) -> String {
        if s.starts_with(r"\??\") {
            s.to_string()
        } else {
            format!(r"\??\{}", s)
        }
    }

    fn normalize_path(p: &Path) -> String {
        p.to_string_lossy().to_lowercase().replace('/', "\\")
    }

    fn normalize_str_path(s: &str) -> String {
        s.to_lowercase().replace('/', "\\")
    }
}

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
pub fn schedule_delete_on_reboot(_path: &Path) -> AppResult<()> {
    Err(AppError::NotImplemented("solo Windows".into()))
}

#[cfg(not(windows))]
pub fn list_pending_renames() -> AppResult<Vec<PendingRename>> {
    Ok(Vec::new())
}

#[cfg(not(windows))]
pub fn cancel_pending_rename(_path: &Path) -> AppResult<()> {
    Err(AppError::NotImplemented("solo Windows".into()))
}
```

### 3. Test manual

```rust
// test temporal
fn main() {
    // Lista actuales
    let pendings = cleartool::platform::pending_rename::list_pending_renames().unwrap();
    println!("Pendings actuales: {}", pendings.len());
    for p in &pendings {
        println!("  {} → {} (delete: {})", p.source, p.destination, p.is_delete);
    }
}
```

### 4. Caveat sobre permisos

`MoveFileExW` requiere SE_BACKUP_NAME y SE_RESTORE_NAME, que vienen con admin. Si no hay admin, devuelve error. Documentar esto en el toast: "Requiere administrador para programar borrado en reboot".

## Criterio de done

- [ ] `schedule_delete_on_reboot` funciona con un archivo de prueba (verificable listando después).
- [ ] `list_pending_renames` devuelve la lista actual del sistema.
- [ ] `cancel_pending_rename` quita una entrada y `list_pending_renames` lo refleja.
- [ ] Strip/add de prefijo `\??\` correcto.
- [ ] Sin admin, devuelve error de Permission identificable.
- [ ] Path normalization es case-insensitive (Windows file system).
