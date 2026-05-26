# Paso 04 — Close gracefully con WM_CLOSE

**Área**: 03-process-manager
**Tiempo estimado**: 3-4 horas
**Dependencias**: Paso 03

## Qué hacemos

Implementar `close_gracefully(pid, timeout_ms)`: enviar `WM_CLOSE` a las ventanas del proceso, esperar a que cierre, si no cierra en el timeout → forzar `TerminateProcess`.

## Por qué

Cuando el cache cleaner dice "Cerrar Spotify", no queremos matarlo brutalmente (perdería la cola, sesión, settings sin guardar). Queremos pedirle educadamente que cierre. Si no hace caso en X segundos, entonces lo forzamos.

## Archivos que tocamos

- `src-tauri/src/platform/processes.rs` (añadir función)

## Cómo

```rust
// src-tauri/src/platform/processes.rs (añadir)

#[cfg(windows)]
pub async fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool> {
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, IsWindowVisible,
        WM_CLOSE,
    };
    use std::time::{Duration, Instant};

    if is_system_protected_pid(pid) {
        return Err(AppError::Permission(format!("PID {} protegido", pid)));
    }

    // 1) Enumerar windows del proceso
    struct EnumCtx { target_pid: u32, hwnds: Vec<HWND> }
    let mut ctx = EnumCtx { target_pid: pid, hwnds: Vec::new() };

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> windows::Win32::Foundation::BOOL {
        let ctx = &mut *(lparam.0 as *mut EnumCtx);
        let mut pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == ctx.target_pid && IsWindowVisible(hwnd).as_bool() {
            ctx.hwnds.push(hwnd);
        }
        true.into()
    }

    unsafe {
        let _ = EnumWindows(Some(callback), LPARAM(&mut ctx as *mut _ as isize));
    }

    if ctx.hwnds.is_empty() {
        // No tiene ventanas → no se puede cerrar grácilmente. Forzar.
        return super::kill_process(pid).map(|_| false);
    }

    // 2) Enviar WM_CLOSE a cada window
    for hwnd in &ctx.hwnds {
        unsafe {
            let _ = PostMessageW(*hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    // 3) Esperar a que el proceso muera, hasta timeout
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms as u64) {
        if !process_exists(pid) {
            return Ok(true);  // cerró bien
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 4) Si sigue vivo, forzar
    super::kill_process(pid)?;
    Ok(false)  // false = forzado
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => {
                let _ = CloseHandle(h);
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(not(windows))]
pub async fn close_gracefully(_pid: u32, _timeout_ms: u32) -> AppResult<bool> {
    Err(AppError::NotImplemented("solo Windows".into()))
}
```

### Detalles de UX

- **Botón "Cerrar"** en UI → llama `close_gracefully(pid, 5000)`.
- Si devuelve `Ok(true)` → toast verde "X cerrada".
- Si devuelve `Ok(false)` → toast amarillo "X no respondió, se forzó cierre".
- Si error → toast rojo con el detalle.

### Edge cases

1. **Proceso sin window visible** (servicio, background helper): no se le puede mandar WM_CLOSE → directamente kill.
2. **Process con diálogo modal abierto** ("¿Guardar cambios?"): WM_CLOSE puede no funcionar porque el proceso está en modal loop. Si el timeout vence → kill forzado, pierdes los cambios sin guardar. Documentar en UI con disclaimer en el botón cuando aplique.
3. **Multi-window** (Chrome con muchas tabs): WM_CLOSE va a TODAS las visible windows. Chrome cierra todas las tabs y luego el proceso.

### Test manual

```
1. Abre Notepad con texto sin guardar.
2. Llama close_gracefully(notepad_pid, 3000).
3. Espera 3s.
4. Notepad debería mostrar diálogo "¿Guardar?" → no responde → kill.
```

vs.

```
1. Abre Spotify (sin estado dirty).
2. Llama close_gracefully(spotify_pid, 5000).
3. Spotify cierra limpiamente. La función devuelve Ok(true).
```

## Criterio de done

- [ ] `close_gracefully(pid, 5000)` cierra Spotify limpiamente.
- [ ] Si proceso no responde, fuerza kill tras timeout.
- [ ] Devuelve `Ok(true)` si fue grácil, `Ok(false)` si fue forzado.
- [ ] Proceso protegido del sistema → error de Permission.
- [ ] Proceso sin window visible → kill directo.
