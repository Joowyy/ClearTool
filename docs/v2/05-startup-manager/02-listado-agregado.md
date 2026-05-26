# Paso 02 — Listado: agregador de los 5 orígenes

**Área**: 05-startup-manager
**Tiempo estimado**: 5-6 horas
**Dependencias**: Paso 01

## Qué hacemos

Implementar `list_startup_entries()` que combina los 5 orígenes en un solo array.

## Archivos

- `src-tauri/src/platform/startup.rs` (nuevo)
- `src-tauri/src/domain/startup.rs` (nuevo, agregador)

## Cómo

### 1. Registry Run/RunOnce

```rust
// src-tauri/src/platform/startup.rs

use winreg::{enums::*, RegKey};

const RUN_KEYS: &[(&str, &str)] = &[
    ("HKLM", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
    ("HKLM", r"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce"),
    ("HKLM", r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run"),
    ("HKCU", r"Software\Microsoft\Windows\CurrentVersion\Run"),
    ("HKCU", r"Software\Microsoft\Windows\CurrentVersion\RunOnce"),
];

pub fn list_registry_run_entries() -> Vec<StartupEntry> {
    let mut out = Vec::new();
    for (hive_s, path) in RUN_KEYS {
        let root = match *hive_s {
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            _ => continue,
        };
        let key = match root.open_subkey(path) { Ok(k) => k, Err(_) => continue };
        for value_r in key.enum_values() {
            if let Ok((name, value)) = value_r {
                let cmd: String = match value.to_string() {
                    s if !s.is_empty() => s,
                    _ => continue,
                };
                let exe_path = extract_exe_path(&cmd);
                let display = exe_path.as_deref()
                    .and_then(|p| std::path::Path::new(p).file_stem())
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| name.clone());

                out.push(StartupEntry {
                    id: format!("registry:{}:{}:{}", hive_s, path, name),
                    origin: StartupOrigin::Registry {
                        hive: hive_s.to_string(),
                        key: path.to_string(),
                        name: name.clone(),
                    },
                    display_name: display,
                    command: cmd,
                    exe_path,
                    icon_path: None,
                    publisher: None,
                    signature_valid: None,
                    impact: StartupImpact::Unknown,
                    last_modified: None,
                    enabled: true,
                    category: StartupCategory::Unknown,
                });
            }
        }
        // RunOnce-Disabled siblings podrían tener entries deshabilitadas; capturar también si quieres
    }
    out
}

fn extract_exe_path(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();
    if trimmed.starts_with('"') {
        trimmed[1..].split('"').next().map(String::from)
    } else {
        trimmed.split_whitespace().next().map(String::from)
    }
}
```

### 2. Startup folder

```rust
pub fn list_startup_folder_entries() -> Vec<StartupEntry> {
    let mut out = Vec::new();
    let dirs = [
        std::env::var("APPDATA").map(|a|
            format!(r"{}\Microsoft\Windows\Start Menu\Programs\Startup", a)).ok(),
        std::env::var("PROGRAMDATA").map(|p|
            format!(r"{}\Microsoft\Windows\Start Menu\Programs\Startup", p)).ok(),
    ];
    for dir_opt in dirs.iter().flatten() {
        let dir = std::path::Path::new(dir_opt);
        if !dir.exists() { continue; }
        let entries = match std::fs::read_dir(dir) { Ok(it) => it, Err(_) => continue };
        for entry in entries.filter_map(|r| r.ok()) {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("lnk") { continue; }
            let display = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
            out.push(StartupEntry {
                id: format!("folder:{}", p.display()),
                origin: StartupOrigin::StartupFolder { lnk_path: p.display().to_string() },
                display_name: display.clone(),
                command: p.display().to_string(),
                exe_path: None,
                icon_path: None,
                publisher: None,
                signature_valid: None,
                impact: StartupImpact::Unknown,
                last_modified: None,
                enabled: true,
                category: StartupCategory::Unknown,
            });
        }
    }
    out
}
```

### 3. Scheduled tasks at logon

```rust
const TASKS_AT_LOGON_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Get-ScheduledTask | Where-Object {
  $_.Triggers | Where-Object { $_.CimClass.CimClassName -eq 'MSFT_TaskLogonTrigger' }
} | ForEach-Object {
  [PSCustomObject]@{
    Name = $_.TaskName
    Path = $_.TaskPath
    State = $_.State
    Action = ($_.Actions[0].Execute + ' ' + $_.Actions[0].Arguments)
  }
} | ConvertTo-Json -Compress -Depth 3
"#;

pub fn list_logon_scheduled_tasks() -> Vec<StartupEntry> {
    let out = match crate::platform::powershell::run_script(TASKS_AT_LOGON_SCRIPT) {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    if !out.status.success() { return Vec::new(); }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(stdout.trim())
        .unwrap_or_default();
    parsed.into_iter().filter_map(|v| {
        let name = v.get("Name")?.as_str()?.to_string();
        let path = v.get("Path")?.as_str()?.to_string();
        let state = v.get("State")?.as_str().unwrap_or("Unknown").to_string();
        let action = v.get("Action")?.as_str().unwrap_or("").to_string();
        Some(StartupEntry {
            id: format!("task:{}{}", path, name),
            origin: StartupOrigin::ScheduledTask { task_path: format!("{}{}", path, name) },
            display_name: name.clone(),
            command: action,
            exe_path: None,
            icon_path: None,
            publisher: None,
            signature_valid: None,
            impact: StartupImpact::Unknown,
            last_modified: None,
            enabled: state == "Ready",
            category: StartupCategory::Unknown,
        })
    }).collect()
}
```

### 4. Services Automatic

```rust
pub fn list_automatic_services() -> Vec<StartupEntry> {
    let services = crate::platform::services::list_all_services()
        .unwrap_or_default();
    services.into_iter()
        .filter(|s| s.start_type == "Automatic")
        .map(|s| StartupEntry {
            id: format!("service:{}", s.name),
            origin: StartupOrigin::Service { service_name: s.name.clone() },
            display_name: s.display_name.clone(),
            command: s.name.clone(),
            exe_path: None,
            icon_path: None,
            publisher: None,
            signature_valid: None,
            impact: StartupImpact::Unknown,
            last_modified: None,
            enabled: true,
            category: StartupCategory::System,
        })
        .collect()
}
```

### 5. UWP startupTasks

Más complejo. Usar `Get-StartApps` o leer `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder\*` para detectar disabled. Para v1.0, **omitir** y dejar TODO. Es nicho.

### 6. Agregador

```rust
// src-tauri/src/domain/startup.rs

use crate::core::AppResult;
use crate::models::startup::StartupEntry;
use crate::platform::startup as platform;

pub fn list_all() -> AppResult<Vec<StartupEntry>> {
    let mut out = Vec::new();
    out.extend(platform::list_registry_run_entries());
    out.extend(platform::list_startup_folder_entries());
    out.extend(platform::list_logon_scheduled_tasks());
    out.extend(platform::list_automatic_services());
    // out.extend(platform::list_uwp_autostart()); // TODO v1.1
    Ok(out)
}
```

## Criterio de done

- [ ] `list_all()` devuelve ≥ 20 entries en una máquina típica.
- [ ] Cada entry tiene `id` único.
- [ ] `enabled` correcto para cada origen.
- [ ] Performance: <2s para listado completo.
