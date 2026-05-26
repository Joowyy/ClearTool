// domain/debloat.rs — orquestación de eliminación de bloatware.

use crate::core::{AppError, AppResult};
use crate::domain::{audit, catalog};
use crate::models::debloat::{
    BloatwareEntry, DetectedPackage, PerEntryResult, RemoveBloatwareInput, RemoveReport,
};
use crate::models::restore::ReverseRecipe;
use crate::platform;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn list_catalog() -> AppResult<Vec<BloatwareEntry>> {
    catalog::load_bloatware_catalog()
}

pub fn detect_installed() -> AppResult<Vec<DetectedPackage>> {
    let cat = catalog::load_bloatware_catalog()?;
    let (users, provs) = platform::debloat::list_appx()?;

    let users_set: HashSet<String> = users
        .iter()
        .map(|u| u.package_family_name.clone())
        .collect();
    let users_loc: HashMap<String, Option<String>> = users
        .iter()
        .map(|u| (u.package_family_name.clone(), u.install_location.clone()))
        .collect();
    let provs_set: HashSet<String> = provs
        .iter()
        .map(|p| p.display_name.clone())
        .collect();

    let mut out = Vec::new();
    for entry in cat {
        let pfn = entry.appx_package_family_name.clone().unwrap_or_default();
        let prov = entry.appx_provisioned_name.clone().unwrap_or_default();
        let in_user = !pfn.is_empty() && users_set.contains(&pfn);
        let in_prov = !prov.is_empty() && provs_set.contains(&prov);
        if !in_user
            && !in_prov
            && entry.removal_method != "uninstaller-string"
            && entry.removal_method != "service-and-files"
        {
            continue;
        }
        out.push(DetectedPackage {
            id: entry.id.clone(),
            display_name: entry.display_name.clone(),
            installed_for_user: in_user,
            installed_provisioned: in_prov,
            size_estimate_mb: None,
            install_location: users_loc.get(&pfn).cloned().flatten(),
        });
    }
    Ok(out)
}

pub fn remove<F>(input: &RemoveBloatwareInput, mut emit_progress: F) -> AppResult<RemoveReport>
where
    F: FnMut(u32, u32, &str, &str),
{
    let cat = catalog::load_bloatware_catalog()?;
    let cat_by_id: HashMap<String, BloatwareEntry> =
        cat.into_iter().map(|e| (e.id.clone(), e)).collect();

    let total = input.entry_ids.len() as u32;

    let restore_seq = if input.create_restore_point && !input.dry_run {
        platform::restore_point::create(
            &format!("ClearTool — debloat {} apps", input.entry_ids.len()),
            0,
            true,
        )
        .ok()
    } else {
        None
    };

    let mut removed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut per: Vec<PerEntryResult> = Vec::with_capacity(input.entry_ids.len());

    for (i, id) in input.entry_ids.iter().enumerate() {
        let entry = match cat_by_id.get(id) {
            Some(e) => e,
            None => {
                skipped += 1;
                per.push(PerEntryResult {
                    id: id.clone(),
                    status: "skipped".into(),
                    method_used: None,
                    error: Some("not-in-catalog".into()),
                });
                continue;
            }
        };
        emit_progress(
            (i + 1) as u32,
            total,
            id,
            &entry.display_name,
        );

        if input.dry_run {
            per.push(PerEntryResult {
                id: id.clone(),
                status: "dry-run".into(),
                method_used: Some(entry.removal_method.clone()),
                error: None,
            });
            removed += 1;
            continue;
        }

        let result = remove_single(entry);

        // Audit individual
        let recipe = make_reverse_recipe(entry);
        let audit_entry = audit::make_entry(
            "debloat",
            "remove",
            false,
            restore_seq,
            vec![id.clone()],
            recipe,
            if result.is_ok() { "success" } else { "failed" },
            result.as_ref().err().map(|e| format!("{}", e)),
        );
        let _ = audit::write_entry(&audit_entry);

        match result {
            Ok(method) => {
                removed += 1;
                per.push(PerEntryResult {
                    id: id.clone(),
                    status: "removed".into(),
                    method_used: Some(method),
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                per.push(PerEntryResult {
                    id: id.clone(),
                    status: "failed".into(),
                    method_used: Some(entry.removal_method.clone()),
                    error: Some(format!("{}", e)),
                });
            }
        }
    }

    Ok(RemoveReport {
        run_id: Uuid::new_v4().to_string(),
        total,
        removed,
        failed,
        skipped,
        restore_point_seq: restore_seq,
        per_entry: per,
    })
}

fn remove_single(entry: &BloatwareEntry) -> AppResult<String> {
    match entry.removal_method.as_str() {
        "appx-user" => {
            let pfn = entry.appx_package_family_name.clone().ok_or_else(|| {
                AppError::Validation(format!("entry {} sin pfn", entry.id))
            })?;
            let st = platform::debloat::remove_appx_user(&pfn)?;
            Ok(format!("appx-user:{}", st))
        }
        "appx-provisioned" => {
            let name = entry.appx_provisioned_name.clone().ok_or_else(|| {
                AppError::Validation(format!("entry {} sin prov name", entry.id))
            })?;
            let st = platform::debloat::remove_appx_provisioned(&name)?;
            Ok(format!("appx-prov:{}", st))
        }
        "uninstaller-string" => {
            let pat = entry.display_name.clone();
            let s = platform::uninstaller::find_uninstall_string(&pat)?
                .ok_or_else(|| {
                    AppError::Services(format!(
                        "no uninstall string para: {}",
                        pat
                    ))
                })?;
            platform::uninstaller::run_uninstaller(&s)?;
            Ok("uninstaller".into())
        }
        "service-and-files" => handle_edge_special(entry),
        other => Err(AppError::Validation(format!(
            "removal_method desconocido: {}",
            other
        ))),
    }
}

fn handle_edge_special(_entry: &BloatwareEntry) -> AppResult<String> {
    platform::registry::write_value(
        "HKLM",
        r"SOFTWARE\Policies\Microsoft\EdgeUpdate",
        "DisableEdgeDesktopShortcutCreation",
        "dword",
        &serde_json::json!(1),
        true,
    )?;
    platform::registry::write_value(
        "HKCU",
        r"Software\Policies\Microsoft\Edge",
        "HubsSidebarEnabled",
        "dword",
        &serde_json::json!(0),
        true,
    )?;

    let candidates = [
        format!(
            r"{}\Desktop\Microsoft Edge.lnk",
            std::env::var("PUBLIC").unwrap_or_default()
        ),
        format!(
            r"{}\Desktop\Microsoft Edge.lnk",
            std::env::var("USERPROFILE").unwrap_or_default()
        ),
    ];
    for path in candidates {
        let _ = std::fs::remove_file(&path);
    }
    Ok("policies-only".into())
}

fn make_reverse_recipe(entry: &BloatwareEntry) -> ReverseRecipe {
    match (
        &entry.removal_method[..],
        &entry.appx_package_family_name,
        &entry.reverse_recipe,
    ) {
        ("appx-user", Some(pfn), Some(r)) => ReverseRecipe::AppxReinstall {
            package_family_name: pfn.clone(),
            store_url: r.store_url.clone(),
        },
        ("appx-provisioned", Some(pfn), Some(r)) => ReverseRecipe::AppxReinstall {
            package_family_name: pfn.clone(),
            store_url: r.store_url.clone(),
        },
        _ => ReverseRecipe::Noop {
            reason: format!(
                "método {} no reversible automáticamente",
                entry.removal_method
            ),
        },
    }
}
