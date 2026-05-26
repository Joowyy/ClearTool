// platform/startup.rs — recolección de entries de auto-arranque desde 5 orígenes.

use crate::models::startup::{StartupCategory, StartupEntry, StartupImpact, StartupOrigin};

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use winreg::enums::*;
    use winreg::RegKey;

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
            let key = match root.open_subkey(path) {
                Ok(k) => k,
                Err(_) => continue,
            };
            for value_r in key.enum_values() {
                if let Ok((name, value)) = value_r {
                    let cmd: String = match value.to_string() {
                        s if !s.is_empty() => s,
                        _ => continue,
                    };
                    let exe_path = extract_exe_path(&cmd);
                    let display = exe_path
                        .as_deref()
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

    pub fn list_startup_folder_entries() -> Vec<StartupEntry> {
        let mut out = Vec::new();
        let dirs = [
            std::env::var("APPDATA")
                .map(|a| format!(r"{}\Microsoft\Windows\Start Menu\Programs\Startup", a))
                .ok(),
            std::env::var("PROGRAMDATA")
                .map(|p| format!(r"{}\Microsoft\Windows\Start Menu\Programs\Startup", p))
                .ok(),
        ];
        for dir_opt in dirs.iter().flatten() {
            let dir = std::path::Path::new(dir_opt);
            if !dir.exists() {
                continue;
            }
            let entries = match std::fs::read_dir(dir) {
                Ok(it) => it,
                Err(_) => continue,
            };
            for entry in entries.filter_map(|r| r.ok()) {
                let p = entry.path();
                let ext = p.extension().and_then(|e| e.to_str());
                let is_lnk = ext == Some("lnk");
                let is_disabled = ext == Some("disabled")
                    || p.to_string_lossy().ends_with(".lnk.disabled");
                if !is_lnk && !is_disabled {
                    continue;
                }
                let display = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("?")
                    .to_string();
                out.push(StartupEntry {
                    id: format!("folder:{}", p.display()),
                    origin: StartupOrigin::StartupFolder {
                        lnk_path: p.display().to_string(),
                    },
                    display_name: display.clone(),
                    command: p.display().to_string(),
                    exe_path: None,
                    icon_path: None,
                    publisher: None,
                    signature_valid: None,
                    impact: StartupImpact::Unknown,
                    last_modified: None,
                    enabled: is_lnk,
                    category: StartupCategory::Unknown,
                });
            }
        }
        out
    }

    const TASKS_AT_LOGON_SCRIPT: &str = r#"
$global:ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = 'SilentlyContinue'
Get-ScheduledTask | Where-Object {
  $_.Triggers | Where-Object { $_.CimClass.CimClassName -eq 'MSFT_TaskLogonTrigger' }
} | ForEach-Object {
  $action = $_.Actions[0]
  $cmd = ''
  if ($action.Execute) {
    $cmd = $action.Execute
    if ($action.Arguments) { $cmd = "$cmd $($action.Arguments)" }
  }
  [PSCustomObject]@{
    Name = $_.TaskName
    Path = $_.TaskPath
    State = $_.State
    Action = $cmd
  }
} | ConvertTo-Json -Compress -Depth 3
"#;

    pub fn list_logon_scheduled_tasks() -> Vec<StartupEntry> {
        let out = match crate::platform::powershell::run_script(TASKS_AT_LOGON_SCRIPT) {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };
        if !out.status.success() {
            return Vec::new();
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or(serde_json::Value::Null);

        let items = match parsed {
            serde_json::Value::Array(arr) => arr,
            serde_json::Value::Object(_) => vec![parsed],
            _ => return Vec::new(),
        };

        items
            .into_iter()
            .filter_map(|v| {
                let name = v.get("Name")?.as_str()?.to_string();
                let path = v.get("Path")?.as_str()?.to_string();
                let state = v.get("State")?.as_str().unwrap_or("Unknown").to_string();
                let action = v.get("Action")?.as_str().unwrap_or("").to_string();
                let full_path = format!("{}{}", path, name);
                Some(StartupEntry {
                    id: format!("task:{}", full_path),
                    origin: StartupOrigin::ScheduledTask {
                        task_path: full_path.clone(),
                    },
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
            })
            .collect()
    }

    pub fn list_automatic_services() -> Vec<StartupEntry> {
        let services = match crate::platform::services::list_all() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        services
            .into_iter()
            .filter(|s| s.start_type == "Automatic" || s.start_type == "AutomaticDelayed")
            .map(|s| StartupEntry {
                id: format!("service:{}", s.name),
                origin: StartupOrigin::Service {
                    service_name: s.name.clone(),
                },
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
}

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
pub fn list_registry_run_entries() -> Vec<StartupEntry> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn list_startup_folder_entries() -> Vec<StartupEntry> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn list_logon_scheduled_tasks() -> Vec<StartupEntry> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn list_automatic_services() -> Vec<StartupEntry> {
    Vec::new()
}
