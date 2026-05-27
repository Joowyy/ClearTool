# 01 — El "Cerrar procesos bloqueantes" cerró el Explorador

> **Severidad:** 🔴 P0 — incidente reproducido por el usuario el 2026-05-27.
> Tras pulsar "Limpiar" con la opción "Cerrar procesos bloqueantes
> automáticamente" activa, ClearTool cerró `explorer.exe`. El usuario
> tuvo que reiniciar el PC porque la barra de tareas, el menú de inicio
> y el escritorio desaparecieron.

## 1. Problema

Cuando el usuario activa la casilla **"Cerrar procesos bloqueantes
automáticamente"** en la vista de plan, ClearTool:

1. Para cada caché bloqueada, llama a `who_locks_path()` que usa
   **Restart Manager** y devuelve TODOS los procesos con handles abiertos
   en cualquier archivo del directorio (muestrea hasta 200 archivos por
   tamaño).
2. Para cachés como `%LOCALAPPDATA%\Microsoft\Windows\Explorer\`
   (thumbcache_\*.db), `INetCache`, `iconcache_*.db`, etc.,
   **`explorer.exe` siempre tiene handles abiertos**.
3. El plan marca esa ubicación con `BlockedAction::CloseProcess { pid:
   <explorer_pid>, ... }`.
4. Al ejecutar, `close_gracefully(pid, 5000)` envía `WM_CLOSE` a las
   ventanas de Explorer. Si no se cierra solo en 5s, llama directamente
   a `kill_process(pid)`.
5. `kill_process` solo aborta si el PID está en `PROTECTED_PROCESS_NAMES`
   (`src-tauri/src/platform/processes.rs:15-22`). **`explorer.exe` NO
   está en esa lista.**
6. → Explorer muere. Windows debería relanzarlo por la policy
   `AutoRestartShell` (HKLM\…\Winlogon), pero si esa clave está en 0 o
   el servicio Shell Experience Host también muere por colateral, el
   shell queda inactivo hasta el siguiente login.

## 2. Causa raíz — referencias al código

- `src-tauri/src/platform/processes.rs:15-22` — `PROTECTED_PROCESS_NAMES`
  incluye `csrss`, `lsass`, `dwm`, `cleartool` pero **no `explorer.exe`,
  ni `searchhost.exe`, ni `startmenuexperiencehost.exe`, ni
  `shellexperiencehost.exe`, ni `runtimebroker.exe`, ni `taskhostw.exe`**.
- `src-tauri/src/platform/processes.rs:191-220` — `classify()` clasifica
  como `ProcessCategory::System` solo si el exe vive en
  `C:\windows\system32\`. `explorer.exe` vive en `C:\Windows\explorer.exe`
  → cae a `UserApp`.
- `src-tauri/src/platform/processes.rs:375-427` — `close_gracefully`
  llama a `kill_process` como fallback sin pedir confirmación.
- `src-tauri/src/domain/cache.rs:558-568` — `suggest_action()` propone
  `CloseProcess` para el primer locker sin filtrar por sensibilidad.
- `src-tauri/src/domain/cache.rs:648-664` — `execute_plan` ejecuta
  `close_gracefully` sin verificar la naturaleza del proceso.

## 3. Fix propuesto

### 3.1 Ampliar `PROTECTED_PROCESS_NAMES`

`src-tauri/src/platform/processes.rs:15`:

```rust
const PROTECTED_PROCESS_NAMES: &[&str] = &[
    // Núcleo del SO (existente)
    "system", "registry", "smss.exe",
    "csrss.exe", "wininit.exe", "winlogon.exe", "lsass.exe",
    "services.exe", "fontdrvhost.exe",
    "dwm.exe", "memcompression", "lockapp.exe",
    "msmpeng.exe", "nissrv.exe", "mssense.exe",
    "cleartool.exe",
    // Shell del usuario (NUEVO — incidente 2026-05-27)
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
    // Antivirus / EDR comerciales (mejor seguro que sorry)
    "avp.exe", "avgnt.exe", "ekrn.exe",
    "windefend.exe", "securityhealthservice.exe",
];
```

### 3.2 Cambiar `suggest_action` para que el shell NUNCA se cierre

`src-tauri/src/domain/cache.rs:558`:

```rust
fn suggest_action(lockers: &[LockingProcess], _category: &str) -> BlockedAction {
    // Si CUALQUIER locker es del shell o sistema, jamás proponer cerrar.
    let any_critical = lockers.iter().any(|l| {
        crate::platform::processes::is_protected_by_name(&l.name)
    });

    if any_critical {
        return BlockedAction::ScheduleReboot;
    }

    if let Some(main) = lockers.first() {
        return BlockedAction::CloseProcess {
            pid: main.pid,
            process_name: main.name.clone(),
        };
    }
    BlockedAction::SkipOnly {
        reason: "Bloqueado por proceso del sistema sin UI".into(),
    }
}
```

### 3.3 Defensa en profundidad en `execute_plan`

`src-tauri/src/domain/cache.rs:648-664` — antes de llamar a
`close_gracefully`, revalidar:

```rust
if opts.auto_close_blocking {
    for blocked in &plan.blocked {
        if let BlockedAction::CloseProcess { pid, process_name } = &blocked.suggested_action {
            // Defensa en profundidad: nunca cerrar procesos protegidos
            // aunque el plan los traiga (catálogo desactualizado, plan
            // antiguo, etc.).
            if crate::platform::processes::is_protected_by_name(process_name)
                || crate::platform::processes::is_system_protected_pid_lookup(*pid)
            {
                emit_progress(
                    "warn",
                    &blocked.display_name,
                    &format!("Omitido cierre de {} (PID {}) — proceso protegido", process_name, pid),
                    0,
                    0,
                );
                continue;
            }

            emit_progress(
                "info",
                &blocked.display_name,
                &format!("Cerrando proceso PID {}", pid),
                0,
                0,
            );
            let ok = crate::platform::processes::close_gracefully(*pid, 5000).await;
            if ok.unwrap_or(false) {
                closed_pids.push(*pid);
            }
        }
    }
}
```

### 3.4 Tests Rust

Añadir a `processes.rs` el módulo `tests`:

```rust
#[test]
fn shell_processes_are_protected() {
    assert!(is_protected_by_name("explorer.exe"));
    assert!(is_protected_by_name("ShellExperienceHost.exe"));
    assert!(is_protected_by_name("StartMenuExperienceHost.exe"));
    assert!(is_protected_by_name("RuntimeBroker.exe"));
    assert!(is_protected_by_name("SearchHost.exe"));
    assert!(is_protected_by_name("dwm.exe"));
}
```

Añadir a `cache.rs` un test de `suggest_action`:

```rust
#[test]
fn suggest_action_never_closes_shell() {
    let lockers = vec![
        LockingProcess { pid: 1234, name: "explorer.exe".into(), path: None },
        LockingProcess { pid: 5678, name: "spotify.exe".into(), path: None },
    ];
    match suggest_action(&lockers, "user-cache") {
        BlockedAction::ScheduleReboot => {}
        other => panic!("Esperado ScheduleReboot, obtuve {:?}", other),
    }
}
```

### 3.5 UI — desactivar la casilla por defecto y warning explícito

`src/features/cache-cleaner/cache-page.tsx`, el toggle "Cerrar procesos
bloqueantes automáticamente":

- **Ya está desactivado por defecto** (correcto, ver screenshot).
- **Cambiar copy** a: "Cerrar apps de usuario que estén bloqueando (no
  toca el sistema)".
- **Tooltip explicando**: "ClearTool nunca cierra el Explorador, la
  barra de tareas ni procesos del sistema. Sólo cierra navegadores,
  reproductores y apps de mensajería si están abiertas."

## 4. Criterio de done

- [ ] `is_protected_by_name("explorer.exe")` → `true`
- [ ] `suggest_action` con cualquier locker crítico devuelve `ScheduleReboot`
- [ ] `execute_plan` no llama nunca a `close_gracefully` sobre un PID
      protegido aunque el plan lo traiga
- [ ] Test `shell_processes_are_protected` pasa
- [ ] Test `suggest_action_never_closes_shell` pasa
- [ ] Verificación manual en VM Windows 11 limpia:
  1. Marcar las cachés `windows-explorer-thumbcache`, `inetcache`,
     `iconcache`.
  2. Pulsar "Analizar".
  3. Comprobar que aparecen en "Bloqueadas" con acción "Se limpiarán al
     reiniciar" (no "Cerrar Explorer").
  4. Activar "Cerrar procesos bloqueantes" + "Limpiar".
  5. Verificar que Explorer sigue abierto al terminar.

## 5. Riesgos / efectos secundarios

- **Cobertura de la lista de protegidos**: si llega un proceso nuevo
  crítico de Windows 12+, hay que añadirlo. Mitigación: tests + revisión
  trimestral.
- **Cachés que requieren cerrar Explorer para limpiarse**: thumbcache,
  iconcache. Solución: programar para reboot (que es lo que el sistema
  hace nativamente con `cleanmgr` cuando lo pides). Es más lento pero
  seguro.
- **Anti-debugger / EDR**: algunos antivirus comerciales (Kaspersky,
  ESET) cuelgan `OpenProcess(PROCESS_TERMINATE)` desde un proceso no
  privilegiado. Ya está cubierto por las protecciones existentes y la
  ampliación.
- **Auditoría**: el cambio toca código destructivo. Antes de mergear,
  `security-auditor` debe revisar este doc + el diff.
