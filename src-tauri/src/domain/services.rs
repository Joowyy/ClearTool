// domain/services.rs — orquestación de servicios Windows.

use crate::core::{AppError, AppResult};
use crate::domain::{audit, catalog};
use crate::models::restore::ReverseRecipe;
use crate::models::service::{
    ApplyPresetReport, ApplyServicePresetInput, PerServiceResult, Service, ServiceEntry,
    SetServiceStateInput,
};
use crate::platform;
use std::collections::HashMap;
use uuid::Uuid;

pub fn list() -> AppResult<Vec<Service>> {
    let runtime = platform::services::list_all()?;
    let catalog_entries = catalog::load_services_catalog().unwrap_or_default();
    let cat_map: HashMap<String, ServiceEntry> = catalog_entries
        .into_iter()
        .map(|e| (e.service_name.clone(), e))
        .collect();

    Ok(runtime
        .into_iter()
        .map(|r| {
            let in_catalog = cat_map.contains_key(&r.name);
            let catalog = cat_map.get(&r.name).cloned();
            Service {
                name: r.name,
                display_name: r.display_name,
                state: r.state,
                start_type: r.start_type,
                description: r.description,
                in_catalog,
                catalog,
            }
        })
        .collect())
}

pub fn set_state(input: &SetServiceStateInput) -> AppResult<()> {
    if !catalog::is_service_allowed(&input.name) {
        return Err(AppError::Permission(format!(
            "servicio no en catálogo: {}",
            input.name
        )));
    }

    audit::with_audit("services", "setState", input.dry_run, |restore_seq| {
        let all = platform::services::list_all()?;
        let prev = all
            .iter()
            .find(|s| s.name == input.name)
            .ok_or_else(|| {
                AppError::Services(format!("servicio no encontrado: {}", input.name))
            })?;
        let previous_start = prev.start_type.clone();
        let previous_state = prev.state.clone();

        if !input.dry_run {
            platform::services::set_start_type(&input.name, &input.start_type)?;
            if input.stop_now && previous_state == "Running" {
                let _ = platform::services::stop(&input.name);
            }
        }

        Ok(audit::make_entry(
            "services",
            "setState",
            input.dry_run,
            restore_seq,
            vec![input.name.clone()],
            ReverseRecipe::Service {
                service_name: input.name.clone(),
                previous_start_type: previous_start,
                previous_state,
            },
            if input.dry_run { "dry-run" } else { "success" },
            None,
        ))
    })
    .map(|_| ())
}

pub fn apply_preset<F>(
    input: &ApplyServicePresetInput,
    mut emit_progress: F,
) -> AppResult<ApplyPresetReport>
where
    F: FnMut(u32, u32, &str),
{
    let entries = catalog::load_services_catalog()?;
    let candidates: Vec<&ServiceEntry> = entries
        .iter()
        .filter(|e| e.presets.iter().any(|p| p == &input.preset))
        .collect();
    let total = candidates.len() as u32;

    let restore_seq = if input.create_restore_point && !input.dry_run {
        platform::restore_point::create(
            &format!("ClearTool — preset servicios '{}'", input.preset),
            12,
            true,
        )
        .ok()
    } else {
        None
    };

    let mut applied = 0u32;
    let mut failed = 0u32;
    let mut protected = 0u32;
    let mut per: Vec<PerServiceResult> = Vec::with_capacity(candidates.len());

    let all = platform::services::list_all()?;
    let runtime_map: std::collections::HashMap<String, _> =
        all.into_iter().map(|r| (r.name.clone(), r)).collect();

    for (i, entry) in candidates.iter().enumerate() {
        emit_progress((i + 1) as u32, total, &entry.service_name);
        let recommended = entry
            .recommended_start_type
            .clone()
            .unwrap_or_else(|| "Manual".into());
        let runtime = runtime_map.get(&entry.service_name);
        let prev_start = runtime
            .map(|r| r.start_type.clone())
            .unwrap_or_else(|| "Unknown".into());
        let prev_state = runtime
            .map(|r| r.state.clone())
            .unwrap_or_else(|| "Unknown".into());

        let result = set_state(&SetServiceStateInput {
            name: entry.service_name.clone(),
            start_type: recommended.clone(),
            stop_now: input.stop_now,
            dry_run: input.dry_run,
        });

        match result {
            Ok(_) => {
                applied += 1;
                per.push(PerServiceResult {
                    name: entry.service_name.clone(),
                    status: if input.dry_run {
                        "dry-run".into()
                    } else {
                        "applied".into()
                    },
                    previous_start_type: prev_start,
                    previous_state: prev_state,
                    error: None,
                });
            }
            Err(AppError::Permission(msg)) if msg.contains("protected") => {
                protected += 1;
                per.push(PerServiceResult {
                    name: entry.service_name.clone(),
                    status: "protected".into(),
                    previous_start_type: prev_start,
                    previous_state: prev_state,
                    error: Some(msg),
                });
            }
            Err(e) => {
                failed += 1;
                per.push(PerServiceResult {
                    name: entry.service_name.clone(),
                    status: "failed".into(),
                    previous_start_type: prev_start,
                    previous_state: prev_state,
                    error: Some(format!("{}", e)),
                });
            }
        }
    }

    Ok(ApplyPresetReport {
        run_id: Uuid::new_v4().to_string(),
        preset: input.preset.clone(),
        total,
        applied,
        failed,
        protected,
        skipped: 0,
        restore_point_seq: restore_seq,
        per_service: per,
    })
}
