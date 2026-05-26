# Paso 03 — Disable/Enable reversible

**Área**: 05-startup-manager
**Tiempo estimado**: 3-4 horas
**Dependencias**: Paso 02

## Qué hacemos

Implementar `disable_startup(id)` y `enable_startup(id)` para cada origen, de forma reversible (renombrar con sufijo, no borrar).

## Por qué

Si "borramos" la entrada, el usuario no puede revertir. Si la renombramos con `.disabled` (registry: backup + delete; folder: rename .lnk → .lnk.disabled; task: Disable-ScheduledTask), siempre se puede reactivar.

## Archivos

- `src-tauri/src/platform/startup.rs` (añadir funciones disable/enable)
- `src-tauri/src/domain/startup.rs` (orquestador con audit log)

## Cómo

### Registry: backup + delete

```rust
pub fn disable_registry_entry(hive: &str, key: &str, name: &str) -> AppResult<()> {
    let root = match hive {
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return Err(AppError::Validation(format!("hive: {}", hive))),
    };
    let run_key = root.open_subkey_with_flags(key, KEY_ALL_ACCESS)?;
    let current_value: String = run_key.get_value(name)
        .map_err(|e| AppError::Registry(e.to_string()))?;

    // Mover a "StartupApproved" o crear sub-key custom
    // Convención Windows: HKCU\...\StartupApproved\Run con value name + 12 bytes (8 bytes ts + 4 bytes flags)
    let approved_key = match hive {
        "HKCU" => r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
        "HKLM" => r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
        _ => unreachable!(),
    };
    let (approved, _) = root.create_subkey(approved_key)?;
    // Flag 03 = disabled. ts = 0.
    let disabled_bytes: Vec<u8> = vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    approved.set_raw_value(name, &winreg::RegValue {
        bytes: disabled_bytes,
        vtype: REG_BINARY,
    }).map_err(|e| AppError::Registry(e.to_string()))?;

    // Backup en nuestro propio dir
    let _ = backup_run_value(hive, key, name, &current_value);

    Ok(())
}

pub fn enable_registry_entry(hive: &str, key: &str, name: &str) -> AppResult<()> {
    // Reverso: marcar StartupApproved con flag 02 (enabled)
    // ...similar
}
```

### Folder: rename .lnk → .lnk.disabled

```rust
pub fn disable_lnk_entry(path: &std::path::Path) -> AppResult<()> {
    let new_path = path.with_extension("lnk.disabled");
    std::fs::rename(path, &new_path)
        .map_err(|e| AppError::Io(format!("rename {}: {}", path.display(), e)))
}

pub fn enable_lnk_entry(disabled_path: &std::path::Path) -> AppResult<()> {
    let original = disabled_path.with_extension("");  // remove .disabled
    let original = original.with_extension("lnk");
    std::fs::rename(disabled_path, &original)
        .map_err(|e| AppError::Io(format!("rename: {}", e)))
}
```

### Scheduled task: Disable-ScheduledTask

```rust
pub fn disable_scheduled_task(task_path_name: &str) -> AppResult<()> {
    let escaped = task_path_name.replace('"', "");
    let script = format!(
        r#"Disable-ScheduledTask -TaskPath '{}' -TaskName '{}'"#,
        std::path::Path::new(task_path_name).parent().map(|p| p.display().to_string()).unwrap_or_default(),
        std::path::Path::new(task_path_name).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()
    );
    let out = crate::platform::powershell::run_script_owned(&script)?;
    if !out.status.success() {
        return Err(AppError::Powershell(String::from_utf8_lossy(&out.stderr).into()));
    }
    Ok(())
}

pub fn enable_scheduled_task(task_path_name: &str) -> AppResult<()> {
    // Same con Enable-ScheduledTask
}
```

### Service: cambiar StartType a Disabled / Manual

```rust
pub fn disable_service(service_name: &str) -> AppResult<()> {
    // Reusar set_service_state existente
    crate::platform::services::set_start_type(service_name, "Disabled")
}

pub fn enable_service(service_name: &str) -> AppResult<()> {
    crate::platform::services::set_start_type(service_name, "Automatic")
}
```

### Orquestador con audit

```rust
// src-tauri/src/domain/startup.rs

pub fn disable_startup(id: &str) -> AppResult<()> {
    let entry = find_entry(id)?;
    let prev_state_for_audit = serialize_state(&entry);

    match &entry.origin {
        StartupOrigin::Registry { hive, key, name } =>
            platform::startup::disable_registry_entry(hive, key, name)?,
        StartupOrigin::StartupFolder { lnk_path } =>
            platform::startup::disable_lnk_entry(std::path::Path::new(lnk_path))?,
        StartupOrigin::ScheduledTask { task_path } =>
            platform::startup::disable_scheduled_task(task_path)?,
        StartupOrigin::Service { service_name } =>
            platform::startup::disable_service(service_name)?,
        _ => return Err(AppError::NotImplemented("UWP autostart aún no soportado".into())),
    }

    // Audit log entry
    crate::domain::audit::write_entry(&crate::domain::audit::make_entry(
        "startup",
        "disable",
        false,
        None,
        vec![id.to_string()],
        crate::models::restore::ReverseRecipe::Noop {
            reason: format!("Reactivar manualmente desde Arranque o usar 'Enable startup'"),
        },
        "success",
        None,
    ))?;

    Ok(())
}
```

## Criterio de done

- [ ] `disable_startup(id)` para los 4 orígenes funciona.
- [ ] `enable_startup(id)` revierte.
- [ ] Cada disable genera entry en audit log.
- [ ] Test manual: deshabilitar Spotify autostart → reiniciar → no arranca. Reactivar → arranca.
