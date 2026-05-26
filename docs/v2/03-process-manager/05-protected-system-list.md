# Paso 05 — Lista de procesos protegidos

**Área**: 03-process-manager
**Tiempo estimado**: 1 hora
**Dependencias**: ninguna

## Qué hacemos

Mantener la blacklist de procesos del sistema que NUNCA se pueden matar/suspender desde ClearTool. Y exponer un check `is_protected(pid_or_name)`.

## Por qué

Matar `csrss.exe`, `lsass.exe` o `smss.exe` provoca pantalla azul instantánea. Es trivial proteger esto en el código.

## Archivos que tocamos

- `src-tauri/src/platform/processes.rs` (centralizar la lista)
- `src-tauri/src/models/process.rs` (la API expone `isProtected: bool`)

## Cómo

### 1. Lista canónica

```rust
// src-tauri/src/platform/processes.rs

/// Procesos críticos del sistema. Matarlos provoca BSOD o sistema inestable.
const PROTECTED_PROCESS_NAMES: &[&str] = &[
    // Kernel + boot
    "system", "registry", "smss.exe",
    // Subsystems
    "csrss.exe", "wininit.exe", "winlogon.exe", "lsass.exe",
    "services.exe", "fontdrvhost.exe",
    // Critical UI
    "dwm.exe", "memcompression", "lockapp.exe",
    // Antimalware (no matar el AV nunca)
    "msmpeng.exe", "nissrv.exe", "mssense.exe",
    // ClearTool mismo (suicide prevention)
    "cleartool.exe",
];

pub fn is_protected_by_name(name: &str) -> bool {
    let n = name.to_lowercase();
    PROTECTED_PROCESS_NAMES.iter().any(|&p| p == n.as_str())
}

pub fn is_system_protected_pid(pid: u32) -> bool {
    if pid == 0 || pid == 4 { return true; }
    // Para chequear por PID, hay que resolver el name primero
    if let Ok(name) = get_process_name(pid) {
        return is_protected_by_name(&name);
    }
    false
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> AppResult<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;

    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| AppError::Io(format!("OpenProcess: {:?}", e)))?;
        let mut buf = vec![0u16; 260];
        let len = GetModuleBaseNameW(h, None, &mut buf);
        let _ = CloseHandle(h);
        if len == 0 {
            return Err(AppError::Io("GetModuleBaseName failed".into()));
        }
        buf.truncate(len as usize);
        Ok(String::from_utf16_lossy(&buf))
    }
}
```

### 2. Aplicar en cada función destructiva

`kill_process`, `kill_process_tree`, `suspend_process`, `close_gracefully` ya hacen el check en pasos previos. Confirmar que TODAS lo hagan.

### 3. Marcar `isProtected` en el listado

En `list_processes_extended` (Paso 02), ya se llena `is_protected: is_system_protected(&name, pid_u32)`. La UI lo usa para desabilitar botones.

### 4. Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_protected_names() {
        assert!(is_protected_by_name("csrss.exe"));
        assert!(is_protected_by_name("CSRSS.EXE"));   // case insensitive
        assert!(is_protected_by_name("lsass.exe"));
        assert!(!is_protected_by_name("notepad.exe"));
    }

    #[test]
    fn pid_0_and_4_always_protected() {
        assert!(is_system_protected_pid(0));
        assert!(is_system_protected_pid(4));
    }
}
```

## Criterio de done

- [ ] Lista hard-coded con 12+ procesos protegidos.
- [ ] `is_protected_by_name` case-insensitive.
- [ ] `is_system_protected_pid` resuelve nombre y compara.
- [ ] Cualquier intento de `kill/suspend/close` sobre protegido devuelve `AppError::Permission`.
- [ ] UI desactiva botones con `process.isProtected === true` y muestra tooltip "Proceso protegido del sistema".
- [ ] Tests unitarios pasan.

## Bonus

Considerar mover la lista a un JSON externo `.claude/skills/process-manager/RESOURCES/protected-processes.json` para que la comunidad pueda contribuir. Pero **no permitir override** desde user-supplied JSON — la denylist solo se puede AMPLIAR, no reducir.
