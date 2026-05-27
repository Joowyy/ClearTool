# Fix 06 — `who_locks_path` solo compara contra `exe_path`

## Síntoma

`analyze_locations` siempre clasifica las ubicaciones como **"ready"** aunque haya
procesos con archivos abiertos dentro. Limpiar `%LOCALAPPDATA%\Spotify\Storage`
con Spotify abierto no se reporta como bloqueado.

## Causa raíz

[`src-tauri/src/platform/processes.rs:431-449`](../../src-tauri/src/platform/processes.rs#L431-L449)

```rust
pub fn who_locks_path(path: &std::path::Path) -> AppResult<Vec<LockingProcess>> {
    let path_str = path.to_string_lossy().to_lowercase();
    let mut locking = Vec::new();
    let processes = list_processes_extended()?;
    for p in &processes {
        if let Some(exe) = &p.exe_path {
            if exe.to_lowercase() == path_str {   // ← solo iguales
                locking.push(LockingProcess { ... });
            }
        }
    }
    Ok(locking)
}
```

Solo dispara cuando `path` es **exactamente** la ruta del `.exe` del proceso.
Para carpetas de caché eso nunca pasa. La spec
([`.claude/specs/v2/03-process-manager.md`](../../.claude/specs/v2/03-process-manager.md))
exige usar **Restart Manager API**.

## Fix propuesto

Reemplazar la implementación por Restart Manager:

```rust
use windows::Win32::System::RestartManager::{
    RmStartSession, RmEndSession, RmRegisterResources, RmGetList,
    CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RM_REBOOT_REASON_NONE,
};

pub fn who_locks_path(path: &Path) -> AppResult<Vec<LockingProcess>> {
    unsafe {
        let mut session: u32 = 0;
        let mut session_key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
        let rc = RmStartSession(&mut session, 0, PWSTR(session_key.as_mut_ptr()));
        if rc != 0 { return Err(...); }

        // Registrar la ruta (acepta archivos y directorios)
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let paths = [PCWSTR(wide.as_ptr())];
        let rc = RmRegisterResources(session, Some(&paths), None, None);
        if rc != 0 { let _ = RmEndSession(session); return Err(...); }

        // Pedir la lista
        let mut needed: u32 = 0;
        let mut count: u32 = 0;
        let mut reason: u32 = 0;
        let _ = RmGetList(session, &mut needed, &mut count, None, &mut reason);
        let mut buf: Vec<RM_PROCESS_INFO> = vec![Default::default(); needed as usize];
        count = needed;
        let rc = RmGetList(session, &mut needed, &mut count, Some(buf.as_mut_ptr()), &mut reason);
        let _ = RmEndSession(session);
        if rc != 0 { return Err(...); }

        Ok(buf.into_iter().take(count as usize).map(|info| LockingProcess {
            pid: info.Process.dwProcessId,
            name: String::from_utf16_lossy(&info.strAppName)
                       .trim_end_matches('\0').to_string(),
            path: None,
        }).collect())
    }
}
```

Para la **carpeta entera**, registrar todos los archivos top-level via
`RmRegisterResources` (Restart Manager acepta hasta ~10 K). Si la carpeta tiene
miles, usar muestreo: registrar los 200 archivos más grandes; si alguno está
bloqueado, marcar la carpeta como blocked.

## Coste

- **Feature flag** `Win32_System_RestartManager` ya está disponible en
  `windows = "0.58"`.
- Tiempo estimado: **0.5 día** (incluyendo test contra Spotify/Discord abiertos).
- Performance: Restart Manager hace lookup global; cachear el handle de sesión
  para múltiples carpetas en el mismo análisis.

## Archivos a tocar

- `src-tauri/src/platform/processes.rs` (reescribir `who_locks_path`).
- `src-tauri/Cargo.toml` (añadir feature `Win32_System_RestartManager`).
- `src-tauri/src/platform/processes.rs` (tests con Spotify proceso real).

## Aceptación

1. Con Spotify abierto, `who_locks_path(%LOCALAPPDATA%\\Spotify)` devuelve al
   menos `[{pid: <spotify.exe>, name: "Spotify"}]`.
2. Sin Spotify abierto, devuelve `[]`.
3. La sección "Bloqueadas" de Caché Cleaner muestra Spotify en el escenario 1.
