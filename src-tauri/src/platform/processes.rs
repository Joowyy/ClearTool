use crate::core::{AppError, AppResult};
use crate::models::process::{ProcessCategory, ProcessInfo, ReleaseReport, LockingProcess};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::{Lazy, OnceCell};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

// System compartido para evitar el coste de ~30ms de new() por llamada.
static SYS: OnceCell<Mutex<System>> = OnceCell::new();

fn get_system() -> &'static Mutex<System> {
    SYS.get_or_init(|| Mutex::new(System::new()))
}

const PROTECTED_PROCESS_NAMES: &[&str] = &[
    // Núcleo del SO
    "system", "registry", "smss.exe",
    "csrss.exe", "wininit.exe", "winlogon.exe", "lsass.exe",
    "services.exe", "fontdrvhost.exe",
    "dwm.exe", "memcompression", "lockapp.exe",
    "msmpeng.exe", "nissrv.exe", "mssense.exe",
    "cleartool.exe",
    // Shell del usuario (incidente 2026-05-27: explorer.exe fue cerrado)
    "explorer.exe",
    "searchhost.exe",
    "searchapp.exe",
    "searchui.exe",
    "startmenuexperiencehost.exe",
    "shellexperiencehost.exe",
    "applicationframehost.exe",
    "runtimebroker.exe",
    "taskhostw.exe",
    "sihost.exe",
    "ctfmon.exe",
    // Antivirus / EDR — mejor seguro que sorry
    "avp.exe", "avgnt.exe", "ekrn.exe",
    "windefend.exe", "securityhealthservice.exe",
];

/// Caché en memoria de resultados de who_locks_path.
/// Los lockers no cambian en menos de 30 s así que evitamos syscalls repetidas.
#[cfg(windows)]
static LOCKS_CACHE: Lazy<Mutex<HashMap<std::path::PathBuf, (Instant, Vec<LockingProcess>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(windows)]
const LOCKS_CACHE_TTL: Duration = Duration::from_secs(30);

const COMMON_CACHE_APPS: &[&str] = &[
    "spotify.exe", "discord.exe", "claude.exe", "obsidian.exe",
    "chrome.exe", "msedge.exe", "brave.exe", "firefox.exe",
    "slack.exe", "teams.exe", "ms-teams.exe", "telegram.exe", "whatsapp.exe",
];

pub fn list_processes_extended() -> AppResult<Vec<ProcessInfo>> {
    let mut sys = get_system()
        .lock()
        .map_err(|_| AppError::Permission("system mutex envenenado".into()))?;
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    let mut out = Vec::with_capacity(sys.processes().len());
    for (pid, p) in sys.processes() {
        let pid_u32 = pid.as_u32();
        let exe_path = p.exe().map(|x| x.to_string_lossy().into_owned());
        let extra = query_process_extra(pid_u32).unwrap_or_default();
        let name = p.name().to_string_lossy().into_owned();
        let is_uwp = exe_path
            .as_deref()
            .map(|x| x.to_lowercase().contains(r"\packages\"))
            .unwrap_or(false);

        let uwp_pf = if is_uwp {
            get_uwp_package_family(pid_u32).ok()
        } else {
            None
        };

        out.push(ProcessInfo {
            pid: pid_u32,
            parent_pid: p.parent().map(|x| x.as_u32()),
            display_name: extra.file_description,
            name: name.clone(),
            exe_path: exe_path.clone(),
            session_id: extra.session_id,
            start_time: Some(chrono::DateTime::from_timestamp(
                p.start_time() as i64, 0,
            ).map(|dt| dt.to_rfc3339()).unwrap_or_default()),
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
            thread_count: extra.thread_count,
            handle_count: extra.handle_count,
            command_line: p.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ").into(),
            user_sid: extra.user_sid,
            user_name: extra.user_name,
            is_uwp,
            uwp_package_family: uwp_pf,
            category: classify(&name, exe_path.as_deref()),
            is_protected: is_system_protected_pid(pid_u32, &name),
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
}

#[cfg(windows)]
fn query_process_extra(pid: u32) -> Option<ProcessExtra> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?
    };

    let mut session_id: u32 = 0;
    unsafe { let _ = ProcessIdToSessionId(pid, &mut session_id); }

    let mut buf = vec![0u16; 260];
    let len = unsafe { GetModuleBaseNameW(handle, None, &mut buf) };
    let file_description = if len > 0 {
        buf.truncate(len as usize);
        Some(String::from_utf16_lossy(&buf))
    } else {
        None
    };

    let thread_count = get_thread_count_via_snapshot(pid);

    unsafe { let _ = CloseHandle(handle); }

    Some(ProcessExtra {
        file_description,
        session_id,
        thread_count,
        handle_count: 0,
        ..Default::default()
    })
}

#[cfg(windows)]
fn get_thread_count_via_snapshot(pid: u32) -> u32 {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok();
        if let Some(snapshot) = snapshot {
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32ProcessID == pid {
                        let _ = CloseHandle(snapshot);
                        return entry.cntThreads;
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
    }
    0
}

#[cfg(not(windows))]
fn query_process_extra(_pid: u32) -> Option<ProcessExtra> { None }

#[cfg(windows)]
fn get_uwp_package_family(pid: u32) -> AppResult<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::core::PWSTR;

    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|_| AppError::Io(std::io::Error::other("OpenProcess UWP failed")))?;
        let mut len: u32 = 256;
        let mut buf = vec![0u16; 256];
        let result = GetPackageFamilyName(h, &mut len, PWSTR(buf.as_mut_ptr()));
        let _ = CloseHandle(h);
        if result.is_ok() && len > 1 {
            buf.truncate((len - 1) as usize);
            Ok(String::from_utf16_lossy(&buf))
        } else {
            Err(AppError::Io(std::io::Error::other("Not a UWP process")))
        }
    }
}

#[cfg(not(windows))]
fn get_uwp_package_family(_pid: u32) -> AppResult<String> {
    Err(AppError::NotImplemented)
}

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

pub fn is_protected_by_name(name: &str) -> bool {
    let n = name.to_lowercase();
    PROTECTED_PROCESS_NAMES.iter().any(|&p| p == n.as_str())
}

pub fn is_system_protected_pid(pid: u32, name: &str) -> bool {
    if pid == 0 || pid == 4 { return true; }
    is_protected_by_name(name)
}

pub fn is_system_protected_pid_lookup(pid: u32) -> bool {
    if pid == 0 || pid == 4 { return true; }
    if let Ok(name) = get_process_name(pid) {
        return is_protected_by_name(&name);
    }
    false
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> AppResult<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|_| AppError::Io(std::io::Error::other("OpenProcess failed")))?;
        let mut buf = vec![0u16; 260];
        let len = GetModuleBaseNameW(h, None, &mut buf);
        let _ = CloseHandle(h);
        if len == 0 {
            return Err(AppError::Io(std::io::Error::other("GetModuleBaseName failed")));
        }
        buf.truncate(len as usize);
        Ok(String::from_utf16_lossy(&buf))
    }
}

#[cfg(not(windows))]
fn get_process_name(_pid: u32) -> AppResult<String> {
    Err(AppError::NotImplemented)
}

fn win32_err(msg: &str) -> AppError {
    AppError::Io(std::io::Error::other(msg.to_string()))
}

#[cfg(windows)]
pub fn kill_process(pid: u32) -> AppResult<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    if is_system_protected_pid_lookup(pid) {
        return Err(AppError::Permission(format!("PID {} es proceso protegido del sistema", pid)));
    }
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|e| AppError::Permission(format!("OpenProcess PID {}: {:?}", pid, e)))?;
        let result = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        result.map_err(|e| win32_err(&format!("TerminateProcess PID {}: {:?}", pid, e)))?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn kill_process_tree(pid: u32) -> AppResult<()> {
    let children = find_children(pid)?;
    for child_pid in children {
        let _ = kill_process_tree(child_pid);
    }
    kill_process(pid)
}

#[cfg(windows)]
fn find_children(parent_pid: u32) -> AppResult<Vec<u32>> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };

    let mut children = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| win32_err(&format!("snapshot: {:?}", e)))?;
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

#[cfg(windows)]
pub fn suspend_process(pid: u32) -> AppResult<()> {
    if is_system_protected_pid_lookup(pid) {
        return Err(AppError::Permission(format!("PID {} protegido", pid)));
    }
    nt_action(pid, "NtSuspendProcess")
}

#[cfg(windows)]
pub fn resume_process(pid: u32) -> AppResult<()> {
    if is_system_protected_pid_lookup(pid) {
        return Err(AppError::Permission(format!("PID {} protegido", pid)));
    }
    nt_action(pid, "NtResumeProcess")
}

#[cfg(windows)]
fn nt_action(pid: u32, fn_name: &str) -> AppResult<()> {
    use windows::core::PCSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let module = GetModuleHandleA(PCSTR(b"ntdll.dll\0".as_ptr()))
            .map_err(|e| win32_err(&format!("ntdll: {:?}", e)))?;
        let mut name_z = fn_name.as_bytes().to_vec();
        name_z.push(0);
        let proc_addr = GetProcAddress(module, PCSTR(name_z.as_ptr()))
            .ok_or_else(|| win32_err(&format!("{} not found", fn_name)))?;

        type NtFn = unsafe extern "system" fn(HANDLE) -> i32;
        let nt: NtFn = std::mem::transmute(proc_addr);

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| AppError::Permission(format!("OpenProcess: {:?}", e)))?;
        let status = nt(handle);
        let _ = CloseHandle(handle);

        if status >= 0 {
            Ok(())
        } else {
            Err(win32_err(&format!("{} status 0x{:X}", fn_name, status as u32)))
        }
    }
}

#[cfg(windows)]
pub async fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool> {
    use std::time::{Duration, Instant};
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, WM_CLOSE,
    };

    if is_system_protected_pid_lookup(pid) {
        return Err(AppError::Permission(format!("PID {} protegido", pid)));
    }

    let hwnds = {
        let mut hwnds: Vec<isize> = Vec::new();
        struct EnumCtx { target_pid: u32, hwnds: *mut Vec<isize> }

        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let ctx = &mut *(lparam.0 as *mut EnumCtx);
            let mut proc_pid: u32 = 0;
            let _ = GetWindowThreadProcessId(hwnd, Some(&mut proc_pid));
            if proc_pid == ctx.target_pid && IsWindowVisible(hwnd).as_bool() {
                (*ctx.hwnds).push(hwnd.0 as isize);
            }
            BOOL::from(true)
        }

        let mut ctx = EnumCtx { target_pid: pid, hwnds: &mut hwnds };
        unsafe {
            let _ = EnumWindows(Some(enum_callback), LPARAM(&mut ctx as *mut _ as isize));
        }
        hwnds
    };

    if hwnds.is_empty() {
        return kill_process(pid).map(|_| false);
    }

    for hwnd_val in &hwnds {
        unsafe {
            let _ = PostMessageW(HWND(*hwnd_val as *mut c_void), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms as u64) {
        if !process_exists(pid) {
            return Ok(true);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    kill_process(pid)?;
    Ok(false)
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => { let _ = CloseHandle(h); true }
            Err(_) => false,
        }
    }
}

/// Devuelve los procesos que tienen handles abiertos sobre `path` (archivo o carpeta).
/// Usa Windows Restart Manager API — detecta correctamente procesos que tienen
/// archivos dentro de un directorio abiertos, no solo el .exe literal.
/// Fallback al método anterior si RmStartSession falla.
/// Los resultados se cachean 30 s para evitar syscalls repetidas en el mismo análisis.
#[cfg(windows)]
pub fn who_locks_path(path: &std::path::Path) -> AppResult<Vec<LockingProcess>> {
    use windows::Win32::System::RestartManager::{
        RmEndSession, RmGetList, RmRegisterResources, RmStartSession,
        RM_PROCESS_INFO,
    };
    use windows::core::PCWSTR;

    // Devolver resultado cacheado si sigue fresco.
    let cache_key = path.to_path_buf();
    {
        if let Ok(guard) = LOCKS_CACHE.lock() {
            if let Some((ts, cached)) = guard.get(&cache_key) {
                if ts.elapsed() < LOCKS_CACHE_TTL {
                    return Ok(cached.clone());
                }
            }
        }
    }

    // Si el path es un directorio, muestrear archivos representativos.
    // Para directorios con >5000 archivos (INetCache, Temp) bajamos a 50.
    let paths_to_register: Vec<std::path::PathBuf> = if path.is_dir() {
        collect_sample_files(path)
    } else {
        vec![path.to_path_buf()]
    };

    if paths_to_register.is_empty() {
        return Ok(Vec::new());
    }

    // Codificar rutas como wide strings (cada una null-terminada).
    let wide_paths: Vec<Vec<u16>> = paths_to_register
        .iter()
        .map(|p| {
            use std::os::windows::ffi::OsStrExt;
            p.as_os_str()
                .encode_wide()
                .chain(std::iter::once(0u16))
                .collect()
        })
        .collect();
    let pcwstr_ptrs: Vec<PCWSTR> = wide_paths
        .iter()
        .map(|w| PCWSTR(w.as_ptr()))
        .collect();

    use windows::Win32::Foundation::WIN32_ERROR;
    const ERROR_SUCCESS: WIN32_ERROR = WIN32_ERROR(0);
    const ERROR_MORE_DATA: WIN32_ERROR = WIN32_ERROR(234);

    let result = unsafe {
        let mut session: u32 = 0;
        // session_key debe ser un buffer mutable wide de CCH_RM_SESSION_KEY + 1 chars.
        let mut session_key = [0u16; 64];
        let rc = RmStartSession(
            &mut session,
            0,
            windows::core::PWSTR(session_key.as_mut_ptr()),
        );
        if rc != ERROR_SUCCESS {
            log::warn!("RmStartSession falló ({:?}), usando fallback", rc);
            return who_locks_path_fallback(path);
        }

        let result = (|| -> AppResult<Vec<LockingProcess>> {
            let rc = RmRegisterResources(
                session,
                Some(pcwstr_ptrs.as_slice()),
                None,
                None,
            );
            if rc != ERROR_SUCCESS {
                return Ok(Vec::new());
            }

            let mut needed: u32 = 0;
            let mut count: u32 = 0;
            let mut reason: u32 = 0;
            let _ = RmGetList(session, &mut needed, &mut count, None, &mut reason);

            if needed == 0 {
                return Ok(Vec::new());
            }

            let mut retries = 0u32;
            loop {
                let buf_size = needed;
                let mut buf: Vec<RM_PROCESS_INFO> =
                    vec![std::mem::zeroed(); buf_size as usize];
                count = buf_size;
                let rc = RmGetList(
                    session,
                    &mut needed,
                    &mut count,
                    Some(buf.as_mut_ptr()),
                    &mut reason,
                );
                if rc == ERROR_SUCCESS {
                    let lockers = buf
                        .into_iter()
                        .take(count as usize)
                        .filter(|info| !is_low_pid(info.Process.dwProcessId))
                        .map(|info| {
                            let name =
                                String::from_utf16_lossy(&info.strAppName)
                                    .trim_end_matches('\0')
                                    .to_string();
                            LockingProcess {
                                pid: info.Process.dwProcessId,
                                name,
                                path: None,
                            }
                        })
                        .collect();
                    return Ok(lockers);
                }
                if rc == ERROR_MORE_DATA && retries < 3 {
                    retries += 1;
                    continue;
                }
                return Ok(Vec::new());
            }
        })();

        let _ = RmEndSession(session);
        result
    };

    // Cachear resultado para evitar syscalls repetidas en el mismo análisis.
    if let Ok(lockers) = &result {
        if let Ok(mut guard) = LOCKS_CACHE.lock() {
            guard.insert(cache_key, (Instant::now(), lockers.clone()));
        }
    }
    result
}

#[cfg(windows)]
fn collect_sample_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    use walkdir::WalkDir;
    // Recogemos hasta 5001 archivos; si hay más, es un directorio grande
    // (INetCache, Temp) y usamos 50 en lugar de 200 para mantener Restart Manager rápido.
    let mut files: Vec<(u64, std::path::PathBuf)> = WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let size = e.metadata().ok()?.len();
            Some((size, e.into_path()))
        })
        .take(5_001)
        .collect();
    let max = if files.len() > 5_000 { 50 } else { 200 };
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files.into_iter().take(max).map(|(_, p)| p).collect()
}

#[cfg(windows)]
fn is_low_pid(pid: u32) -> bool {
    pid == 0 || pid == 4
}

#[cfg(windows)]
fn who_locks_path_fallback(path: &std::path::Path) -> AppResult<Vec<LockingProcess>> {
    let path_str = path.to_string_lossy().to_lowercase();
    let processes = list_processes_extended()?;
    Ok(processes
        .into_iter()
        .filter(|p| {
            p.exe_path
                .as_deref()
                .map(|e| e.to_lowercase() == path_str)
                .unwrap_or(false)
        })
        .map(|p| LockingProcess {
            pid: p.pid,
            name: p.name,
            path: p.exe_path,
        })
        .collect())
}

pub async fn release_common_apps_for_cleanup() -> AppResult<ReleaseReport> {
    let processes = list_processes_extended()?;
    let mut closed = 0u32;
    let mut failed = 0u32;
    let mut closed_names = Vec::new();

    for p in processes {
        let name_lower = p.name.to_lowercase();
        if !COMMON_CACHE_APPS.contains(&name_lower.as_str()) { continue; }
        if p.is_protected { continue; }

        match close_gracefully(p.pid, 5000).await {
            Ok(_) => {
                closed += 1;
                closed_names.push(p.name);
            }
            Err(_) => failed += 1,
        }
    }

    Ok(ReleaseReport {
        closed_count: closed,
        failed_count: failed,
        closed_processes: closed_names,
    })
}

#[cfg(not(windows))]
pub fn kill_process(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
#[cfg(not(windows))]
pub fn kill_process_tree(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
#[cfg(not(windows))]
pub fn suspend_process(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
#[cfg(not(windows))]
pub fn resume_process(_pid: u32) -> AppResult<()> {
    Err(AppError::NotImplemented)
}
#[cfg(not(windows))]
pub async fn close_gracefully(_pid: u32, _timeout_ms: u32) -> AppResult<bool> {
    Err(AppError::NotImplemented)
}
#[cfg(not(windows))]
pub fn who_locks_path(_path: &std::path::Path) -> AppResult<Vec<LockingProcess>> {
    Err(AppError::NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_protected_names() {
        assert!(is_protected_by_name("csrss.exe"));
        assert!(is_protected_by_name("CSRSS.EXE"));
        assert!(is_protected_by_name("lsass.exe"));
        assert!(!is_protected_by_name("notepad.exe"));
    }

    #[test]
    fn shell_processes_are_protected() {
        assert!(is_protected_by_name("explorer.exe"));
        assert!(is_protected_by_name("Explorer.EXE"));
        assert!(is_protected_by_name("shellexperiencehost.exe"));
        assert!(is_protected_by_name("ShellExperienceHost.exe"));
        assert!(is_protected_by_name("startmenuexperiencehost.exe"));
        assert!(is_protected_by_name("StartMenuExperienceHost.exe"));
        assert!(is_protected_by_name("runtimebroker.exe"));
        assert!(is_protected_by_name("RuntimeBroker.exe"));
        assert!(is_protected_by_name("searchhost.exe"));
        assert!(is_protected_by_name("SearchHost.exe"));
        assert!(is_protected_by_name("dwm.exe"));
        // Aseguramos que apps de usuario normales NO están protegidas
        assert!(!is_protected_by_name("spotify.exe"));
        assert!(!is_protected_by_name("discord.exe"));
        assert!(!is_protected_by_name("chrome.exe"));
    }

    #[test]
    fn pid_0_and_4_always_protected() {
        assert!(is_system_protected_pid(0, "idle"));
        assert!(is_system_protected_pid(4, "System"));
    }

    #[test]
    fn classify_known_processes() {
        assert_eq!(classify("chrome.exe", Some(r"C:\Program Files\Google\Chrome\chrome.exe")), ProcessCategory::Browser);
        assert_eq!(classify("spotify.exe", Some(r"C:\Users\test\AppData\Roaming\Spotify\spotify.exe")), ProcessCategory::Media);
        // svchost vive en System32 → el check de path gana sobre el de nombre
        assert_eq!(classify("svchost.exe", Some(r"C:\Windows\System32\svchost.exe")), ProcessCategory::System);
        assert_eq!(classify("notepad.exe", Some(r"C:\Windows\System32\notepad.exe")), ProcessCategory::System);
        assert_eq!(classify("myapp.exe", Some(r"C:\Program Files\MyApp\myapp.exe")), ProcessCategory::UserApp);
    }
}
