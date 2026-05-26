# v2 · 03 — Process Manager (módulo nuevo, P0)

## Por qué este módulo es P0

El cache cleaner v2 depende de poder responder "quién bloquea este archivo" y "ciérralo por mí". Sin process manager, el bug crítico del cache no se resuelve. Por eso este módulo entra en P0 aunque sea "nuevo".

Además, el power user que usa ClearTool ya tiene un Task Manager en mente. Un process manager **bien diseñado** dentro de ClearTool es una feature diferenciadora porque puede:

1. Listar procesos con información que Task Manager no muestra fácilmente (archivos abiertos, handles, llamadas a red).
2. Suspender/reanudar (no sólo matar).
3. "Cerrar todo lo no esencial" — preset para liberar caches.
4. Detectar bloatware corriendo en background (procesos de Cortana, SearchHost, etc.) que el debloat va a tocar.

## Alcance v1.0

Tres usos primarios:

### A. Soporte interno al cache cleaner
"¿Quién tiene abierto este archivo?" → vía Restart Manager API. No expone UI propia.

### B. Pestaña "Procesos" (nueva)
Listado de procesos con acciones. Sustituye o complementa los "Top procesos" del Dashboard.

### C. Preset "Liberar para limpieza"
Cierra grácilmente los browsers, Spotify, Claude, Discord, etc. (cualquier app con UI activa que el usuario tenga corriendo). Pide confirmación antes de cerrar cada uno.

## Modelo de datos

```rust
// src-tauri/src/models/process.rs
#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub display_name: Option<String>,  // del .rc del exe
    pub exe_path: Option<String>,
    pub session_id: u32,
    pub start_time: String,            // RFC3339
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub thread_count: u32,
    pub handle_count: u32,
    pub command_line: Option<String>,
    pub user: Option<String>,          // SID resuelto
    pub is_uwp: bool,
    pub uwp_package_family: Option<String>,
    pub category: ProcessCategory,
}

#[derive(Serialize, ts_rs::TS)]
pub enum ProcessCategory {
    System,        // svchost, dwm, csrss, ...
    Service,       // hosted bajo svchost
    Browser,       // chrome, edge, brave, firefox
    Communication, // discord, teams, slack, whatsapp, telegram
    Media,         // spotify, vlc, music players
    Background,    // helpers, updaters, autostart
    UserApp,       // todo lo demás con ventana
}
```

## Backend

### Listado base
Usar la crate `sysinfo` (ya en deps para telemetría). Para datos extra (handles, parent PID, command line) usar Win32 directamente:

```rust
// src-tauri/src/platform/processes.rs
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
            thread_count: extra.thread_count,
            handle_count: extra.handle_count,
            command_line: extra.command_line,
            user: extra.user_sid_resolved,
            is_uwp: extra.is_uwp,
            uwp_package_family: extra.uwp_package_family,
            category: classify(p.name(), extra.exe_path.as_deref()),
            ..Default::default()
        });
    }
    Ok(out)
}
```

### Clasificación heurística

```rust
fn classify(name: &str, exe_path: Option<&str>) -> ProcessCategory {
    let n = name.to_lowercase();
    let path = exe_path.unwrap_or("").to_lowercase();

    if path.contains(r"\system32\") || matches!(n.as_str(),
        "svchost.exe" | "csrss.exe" | "dwm.exe" | "winlogon.exe" |
        "services.exe" | "lsass.exe" | "smss.exe" | "wininit.exe"
    ) {
        return ProcessCategory::System;
    }
    if matches!(n.as_str(),
        "chrome.exe" | "msedge.exe" | "brave.exe" | "firefox.exe" |
        "opera.exe" | "vivaldi.exe"
    ) {
        return ProcessCategory::Browser;
    }
    if matches!(n.as_str(),
        "discord.exe" | "teams.exe" | "slack.exe" |
        "whatsapp.exe" | "telegram.exe" | "skype.exe"
    ) {
        return ProcessCategory::Communication;
    }
    if matches!(n.as_str(),
        "spotify.exe" | "vlc.exe" | "musicbee.exe" | "wmplayer.exe"
    ) {
        return ProcessCategory::Media;
    }
    if path.contains(r"\packages\") {
        // UWP background helper
        return ProcessCategory::Background;
    }
    ProcessCategory::UserApp
}
```

### Acciones sobre procesos

```rust
pub fn kill_process(pid: u32) -> AppResult<()>;
pub fn kill_process_tree(pid: u32) -> AppResult<()>;    // mata padre + hijos
pub fn suspend_process(pid: u32) -> AppResult<()>;       // NtSuspendProcess
pub fn resume_process(pid: u32) -> AppResult<()>;        // NtResumeProcess
pub fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool>;
    // WM_CLOSE → espera → si sigue vivo, TerminateProcess
```

### "Cerrar grácilmente" para apps con UI

```rust
pub fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool> {
    // 1. Enumerar windows del proceso
    let windows = enum_windows_for_pid(pid)?;
    if windows.is_empty() {
        // sin ventanas → forzar kill
        return kill_process(pid).map(|_| false);
    }
    // 2. Enviar WM_CLOSE a la main window
    for hwnd in &windows {
        unsafe { PostMessageW(*hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))?; }
    }
    // 3. Esperar hasta timeout
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms as u64) {
        if !process_exists(pid) {
            return Ok(true);  // cerró bien
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // 4. Forzar
    kill_process(pid)?;
    Ok(false)
}
```

### Comandos IPC

```rust
#[tauri::command] pub async fn list_processes() -> Result<Vec<ProcessInfo>>;
#[tauri::command] pub async fn kill_process(pid: u32) -> Result<()>;
#[tauri::command] pub async fn kill_process_tree(pid: u32) -> Result<()>;
#[tauri::command] pub async fn suspend_process(pid: u32) -> Result<()>;
#[tauri::command] pub async fn resume_process(pid: u32) -> Result<()>;
#[tauri::command] pub async fn close_gracefully(pid: u32, timeout_ms: u32) -> Result<bool>;
#[tauri::command] pub async fn who_locks_path(path: String) -> Result<Vec<ProcessInfo>>;
#[tauri::command] pub async fn release_caches() -> Result<ReleaseReport>;
   // ↑ preset: cierra browsers/spotify/discord/claude/etc según categoría
```

## Frontend — pestaña Procesos

```
┌─────────────────────────────────────────────────────────────────────┐
│ Procesos                          ☐ Mostrar System  ☐ Sólo con UI  │
│ 124 procesos · 27 categorías                                        │
│                                                                     │
│ [Filtrar...]                                  [Liberar para limpiar]│
│                                                                     │
│ ▼ Browsers (3)                                                      │
│   chrome.exe          PID 4521  CPU 12%  RAM 1.2GB  Handles 1840    │
│     └─ chrome.exe         5012        2%        180MB    320         │
│     └─ chrome.exe         5024        0%        120MB    240         │
│                                              [Cerrar tab] [Kill]    │
│   msedge.exe          PID 7820  CPU 1%   RAM 200MB  Handles 410     │
│                                                                     │
│ ▼ Media (1)                                                         │
│   Spotify.exe         PID 9100  CPU 3%   RAM 400MB  Handles 920     │
│                                              [Cerrar] [Suspender]   │
│                                                                     │
│ ▶ System (45)                                                       │
│ ▶ Background (28)                                                   │
└─────────────────────────────────────────────────────────────────────┘
```

### Comportamiento

- Click en proceso → panel lateral con detalles (cmdline, exe path, parent, archivos abiertos).
- Botón "Liberar para limpiar" → confirma:
  > "Esto cerrará Chrome, Edge, Spotify, Discord, Claude. ¿Continuar?"
- Estado loading mientras cierra cada uno.
- Tras cerrar, redirige a la pestaña Caché y dispara escaneo automático.

## Seguridad

### NO hacer:
- Matar procesos del sistema (System, csrss, smss, wininit, lsass) → blacklist hard-coded.
- Matar el propio ClearTool.
- Operar sobre procesos de otra sesión sin elevación.

### Sí hacer:
- Confirmar antes de cerrar grupo grande (>3 procesos a la vez).
- Audit log de cada kill/suspend (módulo `process-manager`, operación `kill`/`suspend`).
- Restore point ANTES de "Liberar para limpiar" — porque podría cerrar algo que el usuario tenía sin guardar.

### Heurística "esto es un proceso del sistema"
```rust
fn is_system_protected(p: &ProcessInfo) -> bool {
    let n = p.name.to_lowercase();
    static PROTECTED: &[&str] = &[
        "system", "registry", "csrss.exe", "smss.exe", "wininit.exe",
        "winlogon.exe", "lsass.exe", "services.exe", "fontdrvhost.exe",
        "dwm.exe", "memcompression", "lockapp.exe",
    ];
    PROTECTED.contains(&n.as_str()) || p.pid == 0 || p.pid == 4
}
```

UI: estos procesos aparecen pero los botones están deshabilitados con tooltip "Proceso protegido del sistema".

## Performance

`list_processes()` corre cada 2s cuando la pestaña está activa. Para mantenerlo fluido:
- En Rust, reusar `sysinfo::System` entre calls (no recrearlo).
- Cache de `display_name`/`uwp_package_family` por PID durante la vida del proceso.
- Si el usuario filtra, hacer el filtrado en JS, no en Rust (el array entero ya cabe en memoria).

Para PCs con 300+ procesos, considerar virtualización de la tabla (react-virtuoso).

## Cómo se integra con Cache Cleaner

```ts
// src/features/cache-cleaner/use-clean-plan.ts
async function presentBlockedLocation(loc, lockedProcesses) {
  return await toast.warning(`Spotify bloquea ${formatSize(loc.bytes)} de caché`, {
    action: {
      label: "Cerrar Spotify",
      onClick: async () => {
        await closeGracefully(lockedProcesses[0].pid, 5000);
        await rescanLocation(loc.id);
      },
    },
  });
}
```

## Criterio de "done"

- [ ] `list_processes()` devuelve los 100+ procesos típicos en <100ms.
- [ ] `kill_process` mata cualquier proceso no-protegido.
- [ ] `close_gracefully(Spotify, 5000)` cierra Spotify limpiamente sin perder cola.
- [ ] `who_locks_path("...Spotify\\LocalCache")` devuelve Spotify.exe.
- [ ] Pestaña Procesos en UI agrupa por categoría y permite filtrar.
- [ ] "Liberar para limpiar" cierra 5 apps comunes con confirmación.
- [ ] Procesos protegidos del sistema están bloqueados con tooltip.

## Futuro post-1.0

- Estadísticas de uso histórico (CPU/RAM ring buffer por proceso).
- Detección de procesos sospechosos (firmas no válidas, paths raros).
- Network connections por proceso (`netstat -b` equivalente nativo).
- Sandbox/AppContainer info por proceso UWP.
