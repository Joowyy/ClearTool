# Paso 01 — Restart Manager API (`who_locks`)

**Área**: 02-cache-engine (o 03-process-manager si lo prefieres ahí)
**Tiempo estimado**: 4-6 horas
**Dependencias**: ninguna (es código nuevo de Rust + windows-rs)

## Qué hacemos

Implementar `platform::process_lock::who_locks(path) -> Vec<LockingProcess>` usando la API de Windows Restart Manager (RmStartSession + RmRegisterResources + RmGetList).

## Por qué

Es la única manera nativa y oficial de saber qué procesos tienen un archivo abierto. Alternativas (`handle.exe`, NtQueryObject) son peores: una requiere distribuir binario de Sysinternals, la otra es brittle entre versiones de Windows.

## Archivos que tocamos

- `src-tauri/Cargo.toml` (añadir features de windows-rs)
- `src-tauri/src/platform/process_lock.rs` (nuevo)
- `src-tauri/src/platform/mod.rs` (exportar el nuevo módulo)
- `src-tauri/src/models/process_lock.rs` (nuevo, para el DTO)

## Cómo

### 1. Cargo.toml — features de windows-rs

Asegúrate de que `windows` crate tiene esto:

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.59", features = [
    "Win32_System_RestartManager",
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
] }
```

(Si ya tienes otras features, añade estas. Si no, crea la sección).

### 2. Modelo `LockingProcess`

```rust
// src-tauri/src/models/process_lock.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockingProcess {
    pub pid: u32,
    pub name: String,
    pub kind: LockingKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LockingKind {
    MainWindow,    // Tiene ventana con UI
    OtherWindow,   // Ventana sin UI principal
    Service,       // Servicio del sistema
    Console,       // App de consola
    Critical,      // Proceso crítico del sistema (NO matar)
    Unknown,
}
```

Añadir export en `src-tauri/src/models/mod.rs`:
```rust
pub mod process_lock;
```

### 3. Implementación

```rust
// src-tauri/src/platform/process_lock.rs
use crate::core::{AppError, AppResult};
use crate::models::process_lock::{LockingKind, LockingProcess};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS, WIN32_ERROR};
    use windows::Win32::System::RestartManager::*;

    pub fn who_locks(path: &Path) -> AppResult<Vec<LockingProcess>> {
        let mut session_handle: u32 = 0;
        let mut session_key = [0u16; CCH_RM_SESSION_KEY as usize + 1];

        unsafe {
            // 1. Iniciar sesión
            let start_result = RmStartSession(
                &mut session_handle,
                0,
                PWSTR(session_key.as_mut_ptr()),
            );
            if start_result != ERROR_SUCCESS.0 {
                return Err(AppError::Io(format!(
                    "RmStartSession failed: code {}",
                    start_result
                )));
            }

            // Helper para cerrar siempre la sesión
            let result = (|| -> AppResult<Vec<LockingProcess>> {
                // 2. Registrar el recurso (el path)
                let path_wide: Vec<u16> = path
                    .as_os_str()
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let paths: Vec<PWSTR> = vec![PWSTR(path_wide.as_ptr() as *mut u16)];

                let register_result = RmRegisterResources(
                    session_handle,
                    Some(&paths),
                    None,
                    None,
                );
                if register_result != ERROR_SUCCESS.0 {
                    return Err(AppError::Io(format!(
                        "RmRegisterResources failed: code {} (¿el path existe?)",
                        register_result
                    )));
                }

                // 3. Obtener procesos. Loop con resize si necesita más buffer.
                let mut needed: u32 = 0;
                let mut count: u32 = 0;
                let mut reboot_reasons: u32 = 0;
                let mut affected: Vec<RM_PROCESS_INFO> = Vec::new();

                // Primera llamada: averiguar cuántos
                let mut buf_size: u32 = 64;
                affected.resize(buf_size as usize, std::mem::zeroed());

                loop {
                    count = buf_size;
                    let get_result = RmGetList(
                        session_handle,
                        &mut needed,
                        &mut count,
                        Some(affected.as_mut_ptr()),
                        &mut reboot_reasons,
                    );

                    if get_result == ERROR_SUCCESS.0 {
                        affected.truncate(count as usize);
                        break;
                    } else if get_result == ERROR_MORE_DATA.0 {
                        buf_size = needed.max(buf_size * 2);
                        affected.resize(buf_size as usize, std::mem::zeroed());
                        continue;
                    } else {
                        return Err(AppError::Io(format!(
                            "RmGetList failed: code {}",
                            get_result
                        )));
                    }
                }

                let processes: Vec<LockingProcess> = affected
                    .into_iter()
                    .map(|p| LockingProcess {
                        pid: p.Process.dwProcessId,
                        name: pwstr_to_string(&p.strAppName),
                        kind: classify_rm_app(p.ApplicationType),
                    })
                    .collect();

                Ok(processes)
            })();

            // 4. Siempre cerrar sesión
            let _ = RmEndSession(session_handle);
            result
        }
    }

    fn pwstr_to_string(buf: &[u16; CCH_RM_MAX_APP_NAME as usize + 1]) -> String {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }

    fn classify_rm_app(app_type: RM_APP_TYPE) -> LockingKind {
        match app_type {
            RmMainWindow => LockingKind::MainWindow,
            RmOtherWindow => LockingKind::OtherWindow,
            RmService => LockingKind::Service,
            RmConsole => LockingKind::Console,
            RmCritical => LockingKind::Critical,
            _ => LockingKind::Unknown,
        }
    }
}

#[cfg(windows)]
pub use windows_impl::who_locks;

#[cfg(not(windows))]
pub fn who_locks(_path: &Path) -> AppResult<Vec<LockingProcess>> {
    Err(AppError::NotImplemented(
        "who_locks sólo disponible en Windows".into(),
    ))
}
```

Añadir export en `src-tauri/src/platform/mod.rs`:
```rust
pub mod process_lock;
```

### 4. Test manual

Crea un test rápido en el bin (`src/main.rs`) temporalmente:

```rust
// SOLO PARA TEST, BORRAR DESPUÉS
fn main() {
    let path = std::path::Path::new(r"C:\Users\joels\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState");
    match cleartool::platform::process_lock::who_locks(path) {
        Ok(procs) => {
            println!("Procesos que bloquean: {:?}", procs);
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

Compila con `cargo build` y ejecuta. Si Terminal está abierta, debe imprimir su PID + nombre.

Borra el test después.

### 5. Hacer público vía IPC

(Opcional en este paso; se hará formalmente en `03-process-manager` o en `02-cache-engine/08`).

Por ahora dejarlo como API interna de Rust.

## Criterio de done

- [ ] `src-tauri/src/platform/process_lock.rs` compila sin warnings.
- [ ] `windows` crate tiene la feature `Win32_System_RestartManager`.
- [ ] Test manual: pasar el path de Windows Terminal con la app abierta devuelve mínimo 1 proceso.
- [ ] Pasar un path inexistente devuelve `Err(...)` con mensaje claro.
- [ ] Pasar un path libre (ej. `C:\Windows\notepad.exe` sin notepad abierto) devuelve `Ok(vec![])`.
- [ ] Sesión Rm siempre se cierra incluso en path de error (verificable con leaks tool si hay duda).

## Referencias

- [RmStartSession docs](https://learn.microsoft.com/en-us/windows/win32/api/restartmanager/nf-restartmanager-rmstartsession)
- [Restart Manager overview](https://learn.microsoft.com/en-us/windows/win32/rstmgr/about-restart-manager)
- [Ejemplo C oficial](https://learn.microsoft.com/en-us/windows/win32/rstmgr/using-restart-manager-with-a-secondary-installer)
