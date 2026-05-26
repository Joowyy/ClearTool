# Paso 02 — Listado: `list_processes_extended`

**Área**: 03-process-manager
**Tiempo estimado**: 4-5 horas
**Dependencias**: Paso 01

## Qué hacemos

Implementar la función que devuelve todos los procesos con su info extendida. Combina `sysinfo` (datos básicos) con Win32 directamente (handles, command line, UWP info).

## Archivos que tocamos

- `src-tauri/Cargo.toml` (añadir features windows-rs si faltan)
- `src-tauri/src/platform/processes.rs` (modificado o nuevo)

## Cómo

### 1. Cargo features

```toml
[target.'cfg(windows)'.dependencies]
sysinfo = "0.32"
windows = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_Security",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_RemoteDesktop",
] }
```

### 2. Implementación

```rust
// src-tauri/src/platform/processes.rs
use crate::core::{AppError, AppResult};
use crate::models::process::{ProcessInfo, ProcessCategory};
use sysinfo::{ProcessesToUpdate, System};

pub fn list_processes_extended() -> AppResult<Vec<ProcessInfo>> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut out = Vec::with_capacity(sys.processes().len());
    for (pid, p) in sys.processes() {
        let pid_u32 = pid.as_u32();
        let exe_path = p.exe().map(|x| x.to_string_lossy().into_owned());

        // Extras via Win32
        let extra = query_process_extra(pid_u32).unwrap_or_default();

        let name = p.name().to_string_lossy().into_owned();
        let is_uwp = exe_path
            .as_deref()
            .map(|x| x.to_lowercase().contains(r"\packages\"))
            .unwrap_or(false);

        out.push(ProcessInfo {
            pid: pid_u32,
            parent_pid: p.parent().map(|x| x.as_u32()),
            display_name: extra.file_description,
            name: name.clone(),
            exe_path: exe_path.clone(),
            session_id: extra.session_id,
            start_time: p.start_time().to_string().into(), // simplificar a RFC3339 si quieres
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
            thread_count: extra.thread_count,
            handle_count: extra.handle_count,
            command_line: p.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ").into(),
            user_sid: extra.user_sid,
            user_name: extra.user_name,
            is_uwp,
            uwp_package_family: extra.uwp_package_family,
            category: classify(&name, exe_path.as_deref()),
            is_protected: is_system_protected(&name, pid_u32),
        });
    }
    Ok(out)
}

#[derive(Default)]
struct ProcessExtra {
    file_description: Option<String>,
    session_id: u32,
    thread_count: u32,
    handle_count: u32,
    user_sid: Option<String>,
    user_name: Option<String>,
    uwp_package_family: Option<String>,
}

#[cfg(windows)]
fn query_process_extra(pid: u32) -> Option<ProcessExtra> {
    use windows::Win32::System::Threading::*;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::GetProcessHandleCount;

    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?
    };

    let mut handle_count: u32 = 0;
    unsafe { let _ = GetProcessHandleCount(handle, &mut handle_count); }

    // Session ID
    let mut session_id: u32 = 0;
    unsafe { windows::Win32::System::RemoteDesktop::ProcessIdToSessionId(pid, &mut session_id).ok()?; }

    unsafe { let _ = CloseHandle(handle); }

    Some(ProcessExtra {
        handle_count,
        session_id,
        ..Default::default()
    })
}

#[cfg(not(windows))]
fn query_process_extra(_pid: u32) -> Option<ProcessExtra> { None }

fn classify(name: &str, exe: Option<&str>) -> ProcessCategory {
    let n = name.to_lowercase();
    let p = exe.unwrap_or("").to_lowercase();

    if p.starts_with(r"c:\windows\system32\") || matches!(n.as_str(),
        "system" | "registry" | "csrss.exe" | "smss.exe" | "wininit.exe" |
        "winlogon.exe" | "lsass.exe" | "services.exe" | "fontdrvhost.exe" |
        "dwm.exe" | "memcompression"
    ) {
        return ProcessCategory::System;
    }
    if n == "svchost.exe" { return ProcessCategory::Service; }
    if matches!(n.as_str(),
        "chrome.exe" | "msedge.exe" | "brave.exe" | "firefox.exe" |
        "opera.exe" | "vivaldi.exe" | "thorium.exe"
    ) { return ProcessCategory::Browser; }
    if matches!(n.as_str(),
        "discord.exe" | "teams.exe" | "ms-teams.exe" | "slack.exe" |
        "whatsapp.exe" | "telegram.exe" | "skype.exe"
    ) { return ProcessCategory::Communication; }
    if matches!(n.as_str(),
        "spotify.exe" | "vlc.exe" | "musicbee.exe" | "wmplayer.exe"
    ) { return ProcessCategory::Media; }
    if matches!(n.as_str(),
        "code.exe" | "rustrover.exe" | "idea64.exe" | "pycharm64.exe" |
        "devenv.exe" | "powershell.exe" | "cmd.exe" | "wt.exe"
    ) { return ProcessCategory::Development; }
    if p.contains(r"\packages\") { return ProcessCategory::Background; }
    ProcessCategory::UserApp
}

fn is_system_protected(name: &str, pid: u32) -> bool {
    if pid == 0 || pid == 4 { return true; }
    let n = name.to_lowercase();
    const PROTECTED: &[&str] = &[
        "system", "registry", "csrss.exe", "smss.exe", "wininit.exe",
        "winlogon.exe", "lsass.exe", "services.exe", "fontdrvhost.exe",
        "memcompression", "lockapp.exe",
    ];
    PROTECTED.contains(&n.as_str())
}
```

### 3. UWP package family detection (opcional pero deseable)

Para detectar el `uwpPackageFamily`, usar `GetPackageFamilyName` desde `windows::Win32::Storage::Packaging::Appx`:

```rust
fn get_uwp_pfn(pid: u32) -> Option<String> {
    use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut len: u32 = 256;
        let mut buf = vec![0u16; 256];
        let result = GetPackageFamilyName(h, &mut len, Some(buf.as_mut_ptr()));
        if result.is_ok() {
            buf.truncate(len as usize - 1);
            return Some(String::from_utf16_lossy(&buf));
        }
        None
    }
}
```

(Si no es UWP, devuelve error; tratarlo como `None`).

### 4. Performance

Si el sistema tiene 300+ procesos, `query_process_extra` corriendo serial para cada uno tarda 200-500ms. Para v1.0 está bien. Si se ve lag en UI:
- Cache `query_process_extra` por PID durante 5 segundos (la mayoría de procesos viven más).
- Lazy: solo computar `command_line`/`handle_count` cuando se expanda esa fila en UI.

## Criterio de done

- [ ] `list_processes_extended` devuelve lista no vacía en una máquina con apps abiertas.
- [ ] Procesos del sistema (svchost, csrss) tienen `category: "system"` o `"service"`.
- [ ] Procesos UWP tienen `isUwp: true`.
- [ ] Procesos protegidos tienen `isProtected: true`.
- [ ] Tiempo de respuesta <300ms para 200 procesos.
- [ ] `cargo check` y `tsc` sin errores.
