// domain/privacy.rs — Privacy Hardening: presets que combinan registry tweaks + servicios.
//
// Los IDs de tweaks hacen referencia a registry-tweaks.json y los servicios a
// services-catalog.json. Toda operación delega en las funciones de dominio existentes.

use crate::core::AppResult;
use crate::domain::audit;
use crate::models::privacy::{
    PrivacyApplyReport, PrivacyLevel, PrivacyPresetPreview, ServiceChange,
};
use crate::models::registry::ApplyTweakInput;
use crate::models::restore::{RegistryRevertOp, ReverseRecipe};
use uuid::Uuid;

// ── Definición de presets ──────────────────────────────────────────────

struct PresetDef {
    registry_tweak_ids: &'static [&'static str],
    service_changes: &'static [(&'static str, &'static str)],
}

const BALANCED: PresetDef = PresetDef {
    registry_tweak_ids: &[
        "search.disable-bing",
        "cortana.disable",
        "telemetry.allow-zero",
        "widgets.disable",
        "start.no-suggestions",
    ],
    service_changes: &[],
};

const STRICT: PresetDef = PresetDef {
    registry_tweak_ids: &[
        "search.disable-bing",
        "cortana.disable",
        "telemetry.allow-zero",
        "widgets.disable",
        "start.no-suggestions",
        "lockscreen.no-spotlight",
    ],
    service_changes: &[
        ("DiagTrack", "Disabled"),
        ("dmwappushservice", "Disabled"),
        ("WerSvc", "Manual"),
        ("RetailDemo", "Disabled"),
    ],
};

const PARANOID: PresetDef = PresetDef {
    registry_tweak_ids: &[
        "search.disable-bing",
        "cortana.disable",
        "telemetry.allow-zero",
        "widgets.disable",
        "start.no-suggestions",
        "lockscreen.no-spotlight",
        "edge.no-startup-boost",
    ],
    service_changes: &[
        ("DiagTrack", "Disabled"),
        ("dmwappushservice", "Disabled"),
        ("WerSvc", "Disabled"),
        ("RetailDemo", "Disabled"),
        ("diagnosticshub.standardcollector.service", "Disabled"),
        ("MapsBroker", "Disabled"),
    ],
};

fn preset_for(level: PrivacyLevel) -> &'static PresetDef {
    match level {
        PrivacyLevel::Balanced => &BALANCED,
        PrivacyLevel::Strict => &STRICT,
        PrivacyLevel::Paranoid => &PARANOID,
    }
}

// ── Preview ────────────────────────────────────────────────────────────

pub fn preview(level: PrivacyLevel) -> AppResult<PrivacyPresetPreview> {
    let def = preset_for(level);
    let service_changes = def
        .service_changes
        .iter()
        .map(|(name, target)| ServiceChange {
            service_name: name.to_string(),
            target_start_type: target.to_string(),
        })
        .collect::<Vec<_>>();
    let estimated = def.registry_tweak_ids.len() as u32 + service_changes.len() as u32;
    Ok(PrivacyPresetPreview {
        level,
        registry_tweak_ids: def.registry_tweak_ids.iter().map(|s| s.to_string()).collect(),
        service_changes,
        estimated_changes: estimated,
    })
}

// ── Apply ──────────────────────────────────────────────────────────────

pub fn apply(level: PrivacyLevel, dry_run: bool) -> AppResult<PrivacyApplyReport> {
    let def = preset_for(level);
    let run_id = Uuid::new_v4().to_string();
    let mut errors: Vec<String> = Vec::new();
    let mut reg_applied = 0u32;
    let mut reg_failed = 0u32;
    let mut svc_applied = 0u32;
    let mut svc_failed = 0u32;
    let mut sub_recipes: Vec<ReverseRecipe> = Vec::new();

    // ── Restore point (solo si no dry-run) ────────────────────────────
    let restore_seq = if dry_run {
        None
    } else {
        match crate::platform::restore_point::create(
            &format!("ClearTool — Privacy Preset {:?}", level),
            0,   // SOFTWARE_INSTALL type
            true, // bypass throttle
        ) {
            Ok(seq) => Some(seq),
            Err(e) => {
                log::warn!("no se pudo crear restore point: {}", e);
                None
            }
        }
    };

    // ── Registry tweaks ────────────────────────────────────────────────
    for id in def.registry_tweak_ids {
        let input = ApplyTweakInput {
            id: id.to_string(),
            enable: true,
            dry_run,
        };

        if dry_run {
            reg_applied += 1;
            continue;
        }

        // Leer estado previo para el reverse recipe antes de aplicar.
        let tweak = match crate::domain::catalog::load_registry_tweaks() {
            Ok(tweaks) => tweaks.into_iter().find(|t| t.id == *id),
            Err(_) => None,
        };

        let prev_ops: Vec<RegistryRevertOp> = if let Some(ref t) = tweak {
            t.operations
                .iter()
                .map(|op| {
                    let prev_val = crate::platform::registry::read_value(
                        &op.hive, &op.key, &op.name,
                    ).unwrap_or(None);
                    RegistryRevertOp {
                        hive: op.hive.clone(),
                        key: op.key.clone(),
                        name: op.name.clone(),
                        kind: op.kind.clone(),
                        previous_value: prev_val,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        match crate::domain::registry::apply(&input) {
            Ok(()) => {
                reg_applied += 1;
                if !prev_ops.is_empty() {
                    sub_recipes.push(ReverseRecipe::Registry { operations: prev_ops });
                }
            }
            Err(e) => {
                reg_failed += 1;
                errors.push(format!("registry tweak {}: {}", id, e));
                log::warn!("privacy apply_tweak {} error: {}", id, e);
            }
        }
    }

    // ── Servicios ──────────────────────────────────────────────────────
    for (service_name, target_type) in def.service_changes {
        if dry_run {
            svc_applied += 1;
            continue;
        }

        // Leer estado previo.
        let prev_start = crate::platform::services::get_start_type_str(service_name)
            .unwrap_or_else(|| "Manual".to_string());
        let prev_state = crate::platform::services::get_state_str(service_name)
            .unwrap_or_else(|| "Stopped".to_string());

        match crate::platform::services::set_start_type(service_name, target_type) {
            Ok(()) => {
                if *target_type == "Disabled" {
                    let _ = crate::platform::services::stop(service_name);
                }
                svc_applied += 1;
                sub_recipes.push(ReverseRecipe::Service {
                    service_name: service_name.to_string(),
                    previous_start_type: prev_start,
                    previous_state: prev_state,
                });
            }
            Err(e) => {
                svc_failed += 1;
                errors.push(format!("servicio {}: {}", service_name, e));
                log::warn!("privacy set_service {} error: {}", service_name, e);
            }
        }
    }

    // ── Audit entry ────────────────────────────────────────────────────
    if !dry_run {
        let items: Vec<String> = def
            .registry_tweak_ids
            .iter()
            .map(|s| s.to_string())
            .chain(def.service_changes.iter().map(|(n, _)| n.to_string()))
            .collect();

        let status = if errors.is_empty() { "success" } else { "partial" };
        let audit_entry = audit::make_entry(
            "privacy",
            "apply_preset",
            false,
            restore_seq,
            items,
            ReverseRecipe::Composite { recipes: sub_recipes },
            status,
            if errors.is_empty() { None } else { Some(errors.join("; ")) },
        );
        let _ = audit::write_entry(&audit_entry);
    }

    Ok(PrivacyApplyReport {
        level,
        dry_run,
        registry_applied: reg_applied,
        registry_failed: reg_failed,
        services_applied: svc_applied,
        services_failed: svc_failed,
        restore_point_seq: restore_seq,
        audit_run_id: run_id,
        errors,
    })
}
