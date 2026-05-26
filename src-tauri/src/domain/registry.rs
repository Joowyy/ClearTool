// domain/registry.rs — tweaks de registro orquestados.

use crate::core::{AppError, AppResult};
use crate::domain::{audit, catalog};
use crate::models::registry::{
    ApplyTweakBatchReport, ApplyTweakInput, PerTweakResult, RegistryTweak,
    TweakState,
};
use crate::models::restore::{RegistryRevertOp, ReverseRecipe};
use crate::platform::registry as preg;
use crate::platform;
use serde_json::Value;
use uuid::Uuid;

pub fn list_tweaks() -> AppResult<Vec<RegistryTweak>> {
    catalog::load_registry_tweaks()
}

pub fn read_state(id: &str) -> AppResult<TweakState> {
    let tweaks = catalog::load_registry_tweaks()?;
    let tweak = tweaks.iter().find(|t| t.id == id).ok_or_else(|| {
        AppError::Permission(format!("tweak no en allowlist: {}", id))
    })?;

    let mut current = Vec::with_capacity(tweak.operations.len());
    let mut matches_enabled = 0;

    for op in &tweak.operations {
        let v = preg::read_value(&op.hive, &op.key, &op.name)?;
        if v
            .as_ref()
            .map(|x| x == &op.enabled_value)
            .unwrap_or(false)
        {
            matches_enabled += 1;
        }
        current.push(v.unwrap_or(Value::Null));
    }

    let total = tweak.operations.len();
    Ok(TweakState {
        id: id.to_string(),
        enabled: matches_enabled == total,
        partial: matches_enabled > 0 && matches_enabled < total,
        current_values: current,
    })
}

pub fn apply(input: &ApplyTweakInput) -> AppResult<()> {
    audit::with_audit("registry", "applyTweak", input.dry_run, |restore_seq| {
        let tweak = lookup_tweak(&input.id)?;
        let mut previous = Vec::with_capacity(tweak.operations.len());

        for op in &tweak.operations {
            let prev = preg::read_value(&op.hive, &op.key, &op.name)?;
            previous.push(RegistryRevertOp {
                hive: op.hive.clone(),
                key: op.key.clone(),
                name: op.name.clone(),
                kind: op.kind.clone(),
                previous_value: prev,
            });
        }

        if !input.dry_run {
            for op in &tweak.operations {
                if input.enable {
                    preg::write_value(&op.hive, &op.key, &op.name, &op.kind, &op.enabled_value, op.create_if_missing)?;
                } else if op.disabled_value.is_null() {
                    preg::delete_value(&op.hive, &op.key, &op.name)?;
                } else {
                    preg::write_value(&op.hive, &op.key, &op.name, &op.kind, &op.disabled_value, op.create_if_missing)?;
                }
            }
        }

        Ok(audit::make_entry(
            "registry",
            "applyTweak",
            input.dry_run,
            restore_seq,
            vec![input.id.clone()],
            ReverseRecipe::Registry {
                operations: previous,
            },
            if input.dry_run { "dry-run" } else { "success" },
            None,
        ))
    })
    .map(|_| ())
}

pub fn apply_batch<F>(
    inputs: &[ApplyTweakInput],
    create_restore_point: bool,
    mut emit_progress: F,
) -> AppResult<ApplyTweakBatchReport>
where
    F: FnMut(u32, u32, &str),
{
    let run_id = Uuid::new_v4().to_string();
    let total = inputs.len() as u32;

    let restore_seq = if create_restore_point && inputs.iter().any(|i| !i.dry_run) {
        platform::restore_point::create(
            &format!("ClearTool — batch registry ({})", inputs.len()),
            12,
            true,
        )
        .ok()
    } else {
        None
    };

    let mut per: Vec<PerTweakResult> = Vec::with_capacity(inputs.len());
    let mut applied = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (i, input) in inputs.iter().enumerate() {
        emit_progress((i + 1) as u32, total, &input.id);

        match apply(input) {
            Ok(_) => {
                applied += 1;
                per.push(PerTweakResult {
                    id: input.id.clone(),
                    status: if input.dry_run {
                        "dry-run".into()
                    } else {
                        "applied".into()
                    },
                    error: None,
                    previous_values: Vec::new(),
                });
            }
            Err(e) => {
                failed += 1;
                per.push(PerTweakResult {
                    id: input.id.clone(),
                    status: "failed".into(),
                    error: Some(format!("{}", e)),
                    previous_values: Vec::new(),
                });
            }
        }
    }

    Ok(ApplyTweakBatchReport {
        run_id,
        total,
        applied,
        failed,
        skipped,
        restore_point_seq: restore_seq,
        per_tweak: per,
    })
}

pub fn revert(id: &str) -> AppResult<()> {
    apply(&ApplyTweakInput {
        id: id.to_string(),
        enable: false,
        dry_run: false,
    })
}

fn lookup_tweak(id: &str) -> AppResult<RegistryTweak> {
    catalog::load_registry_tweaks()?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| AppError::Permission(format!("tweak no en allowlist: {}", id)))
}
