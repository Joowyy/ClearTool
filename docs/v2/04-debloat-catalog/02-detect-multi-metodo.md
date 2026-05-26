# Paso 02 — Detect multi-método

**Área**: 04-debloat-catalog
**Tiempo estimado**: 4-5 horas
**Dependencias**: Paso 01

## Qué hacemos

Extender `detect_installed_bloatware` para que verifique presencia por 5 vías:
1. Appx user
2. Appx provisioned
3. Uninstaller string en registry
4. Service presente
5. Scheduled task presente

Hoy solo cubre Appx. OEM bloatware (HP Wolf, McAfee trial) son uninstaller-string y se reportan como "no detectado" incorrectamente.

## Archivos que tocamos

- `src-tauri/src/domain/debloat.rs` (extender `detect_installed`)
- `src-tauri/src/platform/uninstaller.rs` (añadir `list_all_uninstallers`)
- `src-tauri/src/platform/services.rs` (añadir `service_exists`)
- `src-tauri/src/platform/scheduled_tasks.rs` (nuevo módulo)

## Cómo

### 1. Listar todos los uninstallers del registry

```rust
// src-tauri/src/platform/uninstaller.rs (añadir)

#[derive(Debug, Clone)]
pub struct UninstallerEntry {
    pub display_name: String,
    pub publisher: Option<String>,
    pub uninstall_string: Option<String>,
    pub registry_key_path: String,
    pub hive: &'static str,
}

pub fn list_all_uninstallers() -> AppResult<Vec<UninstallerEntry>> {
    let mut out = Vec::new();
    out.extend(read_uninstall_hive("HKLM", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall")?);
    out.extend(read_uninstall_hive("HKLM", r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall")?);
    out.extend(read_uninstall_hive("HKCU", r"Software\Microsoft\Windows\CurrentVersion\Uninstall")?);
    Ok(out)
}

fn read_uninstall_hive(hive: &'static str, path: &str) -> AppResult<Vec<UninstallerEntry>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let root = match hive {
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return Err(AppError::Validation(format!("hive inválido: {}", hive))),
    };

    let key = match root.open_subkey(path) {
        Ok(k) => k,
        Err(_) => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    for sub_name_r in key.enum_keys() {
        let sub_name = match sub_name_r { Ok(n) => n, Err(_) => continue };
        let sub = match key.open_subkey(&sub_name) { Ok(s) => s, Err(_) => continue };
        let display_name: Option<String> = sub.get_value("DisplayName").ok();
        let display_name = match display_name { Some(d) if !d.is_empty() => d, _ => continue };
        out.push(UninstallerEntry {
            display_name,
            publisher: sub.get_value("Publisher").ok(),
            uninstall_string: sub.get_value("UninstallString").ok(),
            registry_key_path: format!("{}\\{}", path, sub_name),
            hive,
        });
    }
    Ok(out)
}

pub fn match_uninstaller<'a>(
    list: &'a [UninstallerEntry],
    entry: &crate::models::debloat::BloatwareEntry,
) -> Vec<&'a UninstallerEntry> {
    let pattern_opt = entry.display_name_pattern.as_deref()
        .unwrap_or(&entry.display_name);

    // Wildcards * → regex
    let regex = match pattern_to_regex(pattern_opt) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    list.iter()
        .filter(|u| {
            regex.is_match(&u.display_name)
                && entry.publisher.as_deref().map_or(true, |p|
                    u.publisher.as_deref().map_or(false, |up| up.contains(p)))
        })
        .collect()
}

fn pattern_to_regex(pattern: &str) -> Result<regex::Regex, regex::Error> {
    // Soporta *, ?, y literales escapados
    let mut re = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => re.push_str(".*"),
            '?' => re.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '^' | '$' | '|' => {
                re.push('\\');
                re.push(c);
            }
            _ => re.push(c),
        }
    }
    re.push('$');
    regex::RegexBuilder::new(&re).case_insensitive(true).build()
}
```

Necesitas crate `regex`:
```toml
regex = "1"
```

### 2. Listar servicios (ya existe; verificar `service_exists`)

```rust
// src-tauri/src/platform/services.rs (añadir o asegurar que existe)

pub fn service_exists(name: &str) -> AppResult<bool> {
    let services = list_all_services()?;
    Ok(services.iter().any(|s| s.name.eq_ignore_ascii_case(name)))
}
```

### 3. Listar scheduled tasks

```rust
// src-tauri/src/platform/scheduled_tasks.rs (nuevo)

use crate::core::{AppError, AppResult};
use crate::platform::powershell;

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub task_path: String,
    pub state: String,
}

const LIST_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Get-ScheduledTask | Select-Object TaskName, TaskPath, State | ConvertTo-Json -Compress -Depth 2
"#;

pub fn list_all_scheduled_tasks() -> AppResult<Vec<ScheduledTask>> {
    let out = powershell::run_script(LIST_SCRIPT)?;
    if !out.status.success() {
        return Err(AppError::Powershell(String::from_utf8_lossy(&out.stderr).into()));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("scheduled tasks: {}", e)))?;
    Ok(parsed.into_iter().filter_map(|v| {
        Some(ScheduledTask {
            name: v.get("TaskName")?.as_str()?.to_string(),
            task_path: v.get("TaskPath")?.as_str()?.to_string(),
            state: v.get("State")?.as_str().unwrap_or("Unknown").to_string(),
        })
    }).collect())
}

pub fn task_exists(name_or_path: &str) -> AppResult<bool> {
    let tasks = list_all_scheduled_tasks()?;
    Ok(tasks.iter().any(|t|
        t.name == name_or_path || format!("{}{}", t.task_path, t.name) == name_or_path
    ))
}
```

### 4. `detect_installed_v2`

```rust
// src-tauri/src/domain/debloat.rs (reemplazar `detect_installed`)

pub fn detect_installed() -> AppResult<Vec<DetectedPackage>> {
    let catalog = catalog::load_bloatware_catalog()?;
    let (appx_users, appx_provs) = platform::debloat::list_appx().unwrap_or_default();
    let uninstallers = platform::uninstaller::list_all_uninstallers().unwrap_or_default();
    let services_cache = platform::services::list_all_services().unwrap_or_default();
    let tasks_cache = platform::scheduled_tasks::list_all_scheduled_tasks().unwrap_or_default();

    let users_pfn: HashSet<String> = appx_users.iter().map(|u| u.package_family_name.clone()).collect();
    let provs_name: HashSet<String> = appx_provs.iter().map(|p| p.display_name.clone()).collect();

    let mut detected = Vec::new();

    for entry in catalog {
        let mut hits = false;
        let mut install_location: Option<String> = None;

        match entry.removal_method.as_str() {
            "appx-user" | "appx-user-and-provisioned" => {
                if let Some(pfn) = &entry.appx_package_family_name {
                    if users_pfn.contains(pfn) {
                        hits = true;
                        install_location = appx_users.iter()
                            .find(|u| &u.package_family_name == pfn)
                            .and_then(|u| u.install_location.clone());
                    }
                }
            }
            "appx-provisioned" => {
                if let Some(name) = &entry.appx_provisioned_name {
                    if provs_name.contains(name) { hits = true; }
                }
            }
            "uninstaller-string" => {
                if !platform::uninstaller::match_uninstaller(&uninstallers, &entry).is_empty() {
                    hits = true;
                }
            }
            "service-and-files" | "service-stop-only" => {
                // Detecta presencia del servicio referenciado en compound_steps o por convención
                if let Some(svc_name) = guess_service_name(&entry) {
                    if services_cache.iter().any(|s| s.name.eq_ignore_ascii_case(&svc_name)) {
                        hits = true;
                    }
                }
            }
            "scheduled-task-disable" => {
                if let Some(task) = guess_task_name(&entry) {
                    if tasks_cache.iter().any(|t| t.name.eq_ignore_ascii_case(&task)) {
                        hits = true;
                    }
                }
            }
            "compound" => {
                hits = entry.compound_steps.iter().any(|step| {
                    match step.method.as_str() {
                        "appx-user" => step.appx_package_family_name.as_ref()
                            .map_or(false, |pfn| users_pfn.contains(pfn)),
                        "appx-provisioned" => step.appx_package_family_name.as_ref()
                            .map_or(false, |pfn| provs_name.contains(pfn)),
                        _ => false,
                    }
                });
            }
            _ => {}
        }

        if hits {
            detected.push(DetectedPackage {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                installed_for_user: entry.appx_package_family_name.as_ref()
                    .map(|p| users_pfn.contains(p)).unwrap_or(false),
                installed_provisioned: entry.appx_provisioned_name.as_ref()
                    .map(|p| provs_name.contains(p)).unwrap_or(false),
                size_estimate_mb: None,
                install_location,
            });
        }
    }
    Ok(detected)
}

fn guess_service_name(entry: &BloatwareEntry) -> Option<String> {
    // Heurística simple. Para uso real, añadir campo `serviceName` al schema.
    entry.compound_steps.iter().find_map(|s| s.service_name.clone())
}

fn guess_task_name(entry: &BloatwareEntry) -> Option<String> {
    entry.compound_steps.iter().find_map(|s| s.task_name.clone())
}
```

### 5. Caveats

- `list_all_scheduled_tasks` vía PowerShell tarda 1-3 segundos (es lento). Cachear el resultado durante 30s.
- `list_all_uninstallers` lee ~500 keys del registry — rápido (<200ms en SSD).
- Hacer el detect en una sola pasada (load todos los caches una vez, luego match cada entry).

## Criterio de done

- [ ] `list_all_uninstallers` devuelve ≥ 100 entradas en una máquina típica.
- [ ] `match_uninstaller` con `displayNamePattern: "HP Wolf Security*"` matchea las versiones reales.
- [ ] `detect_installed` cubre los 5 métodos.
- [ ] `cargo test` pasa.
- [ ] En la VM OEM Lenovo, detect devuelve ≥ 20 (sin contar Appx que ya funcionaban).
- [ ] Tiempo total de `detect_installed` < 5 segundos en máquina con 300 procesos.
