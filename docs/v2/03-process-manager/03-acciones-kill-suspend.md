# Paso 03 — Acciones: kill, suspend, resume

**Área**: 03-process-manager
**Tiempo estimado**: 3 horas
**Dependencias**: Paso 01 (modelos)

## Qué hacemos

Implementar las funciones de manipulación de procesos básicas: matar (forzado), matar árbol completo, suspender, reanudar.

## Archivos que tocamos

- `src-tauri/src/platform/processes.rs` (añadir funciones)

## Cómo

```rust
// src-tauri/src/platform/processes.rs (añadir)

use crate::core::{AppError, AppResult};

#[cfg(windows)]
mod actions {
    use super::*;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Threading::{
        OpenProcess, TerminateProcess,
        PROCESS_TERMINATE, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };

    pub fn kill_process(pid: u32) -> AppResult<()> {
        if super::super::is_system_protected_pid(pid) {
            return Err(AppError::Permission(
                format!("PID {} es proceso protegido del sistema", pid),
            ));
        }
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
                .map_err(|e| AppError::Permission(format!("OpenProcess PID {}: {:?}", pid, e)))?;
            let result = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
            result.map_err(|e| AppError::Io(format!("TerminateProcess PID {}: {:?}", pid, e)))?;
        }
        Ok(())
    }

    pub fn kill_process_tree(pid: u32) -> AppResult<()> {
        let children = find_children(pid)?;
        for child_pid in children {
            let _ = kill_process_tree(child_pid);  // recursivo
        }
        kill_process(pid)
    }

    fn find_children(parent_pid: u32) -> AppResult<Vec<u32>> {
        let mut children = Vec::new();
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|e| AppError::Io(format!("snapshot: {:?}", e)))?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32ParentProcessID == parent_pid {
                        children.push(entry.th32ProcessID);
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
        Ok(children)
    }

    // Suspend/Resume usan NtSuspendProcess / NtResumeProcess — no expuestos en windows-rs.
    // Usar GetProcAddress dinámico de ntdll.dll.
    pub fn suspend_process(pid: u32) -> AppResult<()> {
        if super::super::is_system_protected_pid(pid) {
            return Err(AppError::Permission(format!("PID {} protegido", pid)));
        }
        nt_action(pid, "NtSuspendProcess")
    }

    pub fn resume_process(pid: u32) -> AppResult<()> {
        nt_action(pid, "NtResumeProcess")
    }

    fn nt_action(pid: u32, fn_name: &str) -> AppResult<()> {
        use windows::core::PCSTR;
        use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

        unsafe {
            let module = GetModuleHandleA(PCSTR(b"ntdll.dll\0".as_ptr()))
                .map_err(|e| AppError::Io(format!("ntdll: {:?}", e)))?;
            let mut name_z = fn_name.as_bytes().to_vec();
            name_z.push(0);
            let proc_addr = GetProcAddress(module, PCSTR(name_z.as_ptr()))
                .ok_or_else(|| AppError::Io(format!("{} not found", fn_name)))?;

            type NtFn = unsafe extern "system" fn(HANDLE) -> i32;
            let nt: NtFn = std::mem::transmute(proc_addr);

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
                .map_err(|e| AppError::Permission(format!("OpenProcess: {:?}", e)))?;
            let status = nt(handle);
            let _ = CloseHandle(handle);

            if status >= 0 {
                Ok(())
            } else {
                Err(AppError::Io(format!("{} status 0x{:X}", fn_name, status as u32)))
            }
        }
    }
}

#[cfg(windows)] pub use actions::*;

#[cfg(not(windows))]
pub fn kill_process(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented("solo Windows".into()))
}
#[cfg(not(windows))]
pub fn kill_process_tree(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented("solo Windows".into()))
}
#[cfg(not(windows))]
pub fn suspend_process(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented("solo Windows".into()))
}
#[cfg(not(windows))]
pub fn resume_process(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented("solo Windows".into()))
}

// Helper expuesto al módulo
pub(super) fn is_system_protected_pid(pid: u32) -> bool {
    // Reusa la lógica del Paso 05; placeholder por ahora.
    pid == 0 || pid == 4
}
```

### Permisos necesarios

- `kill_process`: requiere `PROCESS_TERMINATE` access right. Sin admin no puede matar procesos de otros usuarios o procesos elevated.
- `suspend_process`: necesita `PROCESS_SUSPEND_RESUME` (parte de QUERY_LIMITED en admin).
- Procesos protegidos (PPL — Protected Process Light) NUNCA se pueden matar desde user mode. Documentar.

## Criterio de done

- [ ] `kill_process(pid)` mata un proceso de prueba (ej. abrir notepad y matarlo).
- [ ] `kill_process_tree(pid)` mata padre + hijos (test: chrome.exe).
- [ ] `suspend_process` y `resume_process` funcionan (verificable: suspend → Task Manager muestra el proceso pausado).
- [ ] Matar PID 0/4 devuelve error de Permission.
- [ ] Funciones devuelven errores claros con código de Win32 cuando fallan.
