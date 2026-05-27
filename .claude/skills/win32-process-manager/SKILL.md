---
name: win32-process-manager
description: Enumerar y actuar sobre procesos Windows desde Rust (sysinfo + Win32) para ClearTool — listado extendido con categorización, kill/kill-tree/suspend/resume, cierre grácil vía WM_CLOSE con fallback a TerminateProcess, y "quién bloquea este archivo" con la Restart Manager API (RmStartSession/RmRegisterResources/RmGetList/RmEndSession). Uso obligatorio al implementar el módulo Process Manager o cuando el cache cleaner necesita resolver bloqueadores de archivos. Incluye la blacklist de procesos protegidos del sistema.
---

# Skill: win32-process-manager

Patrones para el módulo Procesos (M2, P0) y para el soporte interno del cache cleaner ("¿quién bloquea este archivo?" + "ciérralo por mí").

## Cuándo usar

- Implementas la pestaña Procesos o el preset "Liberar para limpieza".
- El cache engine necesita `who_locks_path` / `close_gracefully`.
- Cualquier acción que termine, suspenda o reanude un proceso.

## Regla de oro de seguridad

Antes de matar/suspender, comprobar SIEMPRE `is_system_protected`. Nunca tocar procesos del sistema ni el propio ClearTool. Confirmación de usuario obligatoria para grupos > 3 procesos, y restore point antes de "Liberar para limpieza".

```rust
fn is_system_protected(p: &ProcessInfo) -> bool {
    static PROTECTED: &[&str] = &[
        "system", "registry", "csrss.exe", "smss.exe", "wininit.exe",
        "winlogon.exe", "lsass.exe", "services.exe", "fontdrvhost.exe",
        "dwm.exe", "memcompression", "lockapp.exe",
    ];
    let n = p.name.to_lowercase();
    PROTECTED.contains(&n.as_str()) || p.pid == 0 || p.pid == 4 || is_self(p.pid)
}
```

La UI muestra estos procesos pero con botones deshabilitados y tooltip "Proceso protegido del sistema".

## Listado: sysinfo como base, Win32 para lo que falta

`sysinfo` da pid/parent/name/exe/cpu/memoria. Win32 aporta handle_count, command_line, SID de usuario y si es UWP. Reusar la instancia `sysinfo::System` entre llamadas (no recrearla en cada refresh de 2s).

```rust
use windows::Win32::System::Threading::*;
use windows::Win32::System::ProcessStatus::*;

pub fn list_processes_extended() -> AppResult<Vec<ProcessInfo>> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let mut out = Vec::with_capacity(sys.processes().len());
    for (pid, p) in sys.processes() {
        let extra = query_process_extra(pid.as_u32()).unwrap_or_default();
        out.push(ProcessInfo {
            pid: pid.as_u32(),
            parent_pid: p.parent().map(|x| x.as_u32()),
            name: p.name().to_string(),
            exe_path: p.exe().map(|x| x.to_string_lossy().into_owned()),
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
            handle_count: extra.handle_count,
            command_line: extra.command_line,
            user: extra.user_sid_resolved,
            is_uwp: extra.is_uwp,
            category: classify(p.name(), extra.exe_path.as_deref()),
            ..Default::default()
        });
    }
    Ok(out)
}
```

Categorías: `System | Service | Browser | Communication | Media | Background | UserApp` (heurística por nombre + path; los de `\system32\` y `\packages\` se tratan especial).

## Acciones

```rust
pub fn kill_process(pid: u32) -> AppResult<()>;       // TerminateProcess tras OpenProcess(PROCESS_TERMINATE)
pub fn kill_process_tree(pid: u32) -> AppResult<()>;  // recorre hijos por parent_pid, mata hojas→raíz
pub fn suspend_process(pid: u32) -> AppResult<()>;    // NtSuspendProcess
pub fn resume_process(pid: u32) -> AppResult<()>;     // NtResumeProcess
pub fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool>;
```

Toda acción: validar `!is_system_protected`, registrar en el audit log (`module: process-manager`, `operation: kill|suspend|...`), y mapear errores a `AppError::Process(...)` (ver skill `error-model`: usa `kind = "process"` o `"access-denied-process"`).

### Cierre grácil (apps con UI)

```rust
pub fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool> {
    let windows = enum_windows_for_pid(pid)?;          // EnumWindows + GetWindowThreadProcessId
    if windows.is_empty() {
        return kill_process(pid).map(|_| false);       // sin ventanas → kill directo
    }
    for hwnd in &windows {
        unsafe { PostMessageW(*hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))?; }
    }
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(timeout_ms as u64) {
        if !process_exists(pid) { return Ok(true); }   // cerró limpio (no perdió cola, p.ej. Spotify)
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    kill_process(pid)?;                                // timeout → forzar
    Ok(false)
}
```

## "¿Quién bloquea este archivo?" — Restart Manager API

Es la pieza que desbloquea el cache cleaner v2. Secuencia: `RmStartSession` → `RmRegisterResources` (los paths) → `RmGetList` (procesos que los usan) → `RmEndSession`. `RmGetList` se llama dos veces: la primera para obtener `pnProcInfoNeeded`, la segunda con el buffer dimensionado.

```rust
use windows::Win32::System::RestartManager::*;
use windows::core::PCWSTR;

pub fn who_locks_path(path: &str) -> AppResult<Vec<ProcessInfo>> {
    unsafe {
        let mut session: u32 = 0;
        let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
        let rc = RmStartSession(&mut session, 0, windows::core::PWSTR(key.as_mut_ptr()));
        if rc.0 != 0 { return Err(AppError::Process(format!("RmStartSession: {}", rc.0))); }

        // RAII: cerrar la sesión pase lo que pase
        let _guard = scopeguard::guard(session, |s| { let _ = RmEndSession(s); });

        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let files = [PCWSTR(wide.as_ptr())];
        let rc = RmRegisterResources(session, Some(&files), None, None);
        if rc.0 != 0 { return Err(AppError::Process(format!("RmRegisterResources: {}", rc.0))); }

        let mut needed: u32 = 0;
        let mut count: u32 = 0;
        let mut reasons: u32 = 0;
        // 1ª llamada: descubrir cuántos
        let _ = RmGetList(session, &mut needed, &mut count, None, &mut reasons);
        if needed == 0 { return Ok(vec![]); }            // nadie lo bloquea

        let mut infos = vec![RM_PROCESS_INFO::default(); needed as usize];
        count = needed;
        let rc = RmGetList(session, &mut needed, &mut count, Some(infos.as_mut_ptr()), &mut reasons);
        if rc.0 != 0 { return Err(AppError::Process(format!("RmGetList: {}", rc.0))); }

        infos.truncate(count as usize);
        Ok(infos.iter().map(|i| resolve_process_info(i.Process.dwProcessId)).collect())
    }
}
```

Notas:
- `RM_PROCESS_INFO.Process` es un `RM_UNIQUE_PROCESS` (PID + `ProcessStartTime`); valida ambos para evitar PID reciclado.
- Mapear `i.ApplicationType` (RmMainWindow, RmService, RmExplorer, RmCritical...) ayuda a decidir si se puede cerrar grácilmente o no.
- No uses `RmShutdown` para cerrar por nosotros: preferimos `close_gracefully` controlado desde nuestra propia lógica + confirmación.

`Cargo.toml`: feature `Win32_System_RestartManager` de `windows`, y opcional `scopeguard` para el RAII de la sesión.

## Integración con cache cleaner

```ts
// al detectar una location bloqueada
toast.warning(`Spotify bloquea ${formatSize(loc.bytes)} de caché`, {
  action: { label: "Cerrar Spotify", onClick: async () => {
    await closeGracefully(locked[0].pid, 5000);
    await rescanLocation(loc.id);
  }},
});
```

## Performance

- `list_processes()` cada 2s solo con la pestaña activa; reusar `System`.
- Cachear `display_name`/`uwp_package_family` por PID durante la vida del proceso.
- Filtrado en JS (el array entero cabe en memoria); virtualizar la tabla (react-virtuoso) con 300+ procesos.

## Tests obligatorios

- `who_locks_path` sobre un archivo abierto por un proceso de prueba devuelve ese PID; sobre uno libre devuelve `[]`.
- `close_gracefully` cierra un proceso con ventana sin matarlo; si supera timeout, lo mata y devuelve `false`.
- `kill_process` sobre un protegido devuelve `AppError::Process` y NO mata nada.
- La sesión de Restart Manager se cierra siempre (incluso en el path de error) — verificar que no quedan sesiones colgadas.
