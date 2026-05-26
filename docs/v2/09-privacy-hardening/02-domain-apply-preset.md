# Paso 02 — Domain: `apply_privacy_preset`

**Área**: 09-privacy-hardening
**Tiempo estimado**: 3-4 horas
**Dependencias**: Paso 01

## Qué hacemos

Implementar la función que carga el preset, calcula los cambios, crea restore point, y aplica todo en orden con audit log.

## Archivos

- `src-tauri/src/domain/privacy.rs` (nuevo)
- `src-tauri/src/models/privacy.rs` (nuevo)

## Cómo

### Modelos

```rust
// src-tauri/src/models/privacy.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPresetsFile {
    pub schema_version: u32,
    pub presets: std::collections::HashMap<String, PrivacyPreset>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPreset {
    pub display_name: String,
    pub description: String,
    pub disclaimer: String,
    pub estimated_changes: u32,
    pub registry_tweaks: Vec<String>,
    pub services: Vec<ServiceConfig>,
    pub scheduled_tasks: Vec<String>,
    pub debloat_entries: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    pub name: String,
    pub start_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyApplyReport {
    pub run_id: String,
    pub level: String,
    pub dry_run: bool,
    pub registry_changed: u32,
    pub services_changed: u32,
    pub tasks_disabled: u32,
    pub appx_removed: u32,
    pub failed: u32,
    pub errors: Vec<String>,
    pub restore_point_seq: Option<u32>,
}
```

### Domain

```rust
// src-tauri/src/domain/privacy.rs

use crate::core::{AppError, AppResult};
use crate::models::privacy::*;

const PRESETS_JSON: &str = include_str!("../../../.claude/skills/windows-registry-ops/RESOURCES/privacy-presets.json");

pub fn load_presets() -> AppResult<PrivacyPresetsFile> {
    serde_json::from_str(PRESETS_JSON)
        .map_err(|e| AppError::Catalog(format!("privacy presets: {}", e)))
}

pub fn get_preset(level: &str) -> AppResult<PrivacyPreset> {
    let presets = load_presets()?;
    presets.presets.get(level)
        .cloned()
        .ok_or_else(|| AppError::Validation(format!("preset desconocido: {}", level)))
}

pub fn apply_preset(level: &str, dry_run: bool) -> AppResult<PrivacyApplyReport> {
    let preset = get_preset(level)?;
    let run_id = uuid::Uuid::new_v4().to_string();

    let restore_point_seq = if !dry_run {
        super::restore::ensure_or_create(
            &format!("ClearTool — privacy {}", level)
        ).ok().flatten()
    } else { None };

    let mut report = PrivacyApplyReport {
        run_id: run_id.clone(),
        level: level.to_string(),
        dry_run,
        registry_changed: 0,
        services_changed: 0,
        tasks_disabled: 0,
        appx_removed: 0,
        failed: 0,
        errors: Vec::new(),
        restore_point_seq,
    };

    // 1) Registry tweaks
    for tweak_id in &preset.registry_tweaks {
        if dry_run {
            report.registry_changed += 1;
            continue;
        }
        match super::registry::apply_tweak(tweak_id, true, false) {
            Ok(_) => report.registry_changed += 1,
            Err(e) => {
                report.failed += 1;
                report.errors.push(format!("registry/{}: {}", tweak_id, e));
            }
        }
    }

    // 2) Services
    for svc in &preset.services {
        if dry_run {
            report.services_changed += 1;
            continue;
        }
        match crate::platform::services::set_start_type(&svc.name, &svc.start_type) {
            Ok(_) => report.services_changed += 1,
            Err(e) => {
                report.failed += 1;
                report.errors.push(format!("service/{}: {}", svc.name, e));
            }
        }
    }

    // 3) Scheduled tasks (disable)
    for task in &preset.scheduled_tasks {
        if dry_run {
            report.tasks_disabled += 1;
            continue;
        }
        let script = format!(r#"Disable-ScheduledTask -TaskName '{}'"#, task.replace('"', ""));
        match crate::platform::powershell::run_script_owned(&script) {
            Ok(out) if out.status.success() => report.tasks_disabled += 1,
            Ok(out) => {
                report.failed += 1;
                report.errors.push(format!("task/{}: {}", task, String::from_utf8_lossy(&out.stderr)));
            }
            Err(e) => {
                report.failed += 1;
                report.errors.push(format!("task/{}: {}", task, e));
            }
        }
    }

    // 4) Debloat entries
    if !preset.debloat_entries.is_empty() {
        if dry_run {
            report.appx_removed = preset.debloat_entries.len() as u32;
        } else {
            let input = crate::models::debloat::RemoveBloatwareInput {
                entry_ids: preset.debloat_entries.clone(),
                dry_run: false,
                create_restore_point: false,  // ya lo creamos arriba
                apply_policies: true,
                disable_services: false,
            };
            let dr = super::debloat::remove(&input, |_, _, _, _| {})?;
            report.appx_removed = dr.removed;
            report.failed += dr.failed;
        }
    }

    // Audit
    let _ = super::audit::write_entry(&super::audit::make_entry(
        "privacy",
        &format!("apply-{}", level),
        dry_run,
        restore_point_seq,
        vec![level.to_string()],
        crate::models::restore::ReverseRecipe::Noop {
            reason: format!("Restaurar desde Restore Point #{:?} para revertir.", restore_point_seq),
        },
        if report.failed == 0 { "success" } else { "partial" },
        if !report.errors.is_empty() { Some(report.errors.join("; ")) } else { None },
    ));

    Ok(report)
}
```

## Criterio de done

- [ ] `load_presets()` valida y devuelve los 3 presets.
- [ ] `apply_preset("balanced", true)` (dry-run) cuenta cambios sin tocar nada.
- [ ] `apply_preset("balanced", false)` aplica registry + services + tasks + debloat en orden.
- [ ] Restore point creado antes (verificable en audit log).
- [ ] Audit entry con `operation: "apply-balanced"` etc.
- [ ] Errors agregados en report, no detiene la ejecución.
