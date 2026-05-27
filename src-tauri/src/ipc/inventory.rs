// ipc/inventory.rs — Comandos Tauri para el inventario universal de apps.

use crate::core::AppResult;
use crate::domain::inventory;
use crate::models::inventory::*;
use crate::models::restore::CreateRestorePointInput;

#[tauri::command]
pub async fn list_installed_apps() -> AppResult<Vec<InstalledApp>> {
    tokio::task::spawn_blocking(inventory::build_inventory)
        .await
        .map_err(|e| crate::core::AppError::External(e.to_string()))?
}

#[tauri::command]
pub async fn compute_residual_hints(_app_id: String, app: InstalledApp) -> AppResult<ResidualHints> {
    tokio::task::spawn_blocking(move || inventory::compute_residual_hints(&app))
        .await
        .map_err(|e| crate::core::AppError::External(e.to_string()))
}

#[tauri::command]
pub async fn uninstall_app(app: InstalledApp, dry_run: bool) -> AppResult<UninstallReport> {
    tokio::task::spawn_blocking(move || inventory::uninstall_app(&app, dry_run))
        .await
        .map_err(|e| crate::core::AppError::External(e.to_string()))?
}

#[tauri::command]
pub async fn clean_residuals(
    app_id: String,
    selected: SelectedResiduals,
    dry_run: bool,
) -> AppResult<CleanResidualsReport> {
    tokio::task::spawn_blocking(move || inventory::clean_residuals(&app_id, &selected, dry_run))
        .await
        .map_err(|e| crate::core::AppError::External(e.to_string()))?
}

#[tauri::command]
pub async fn uninstall_app_complete(
    app: InstalledApp,
    dry_run: bool,
) -> AppResult<UninstallCompleteReport> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || run_complete_uninstall(app_clone, dry_run))
        .await
        .map_err(|e| crate::core::AppError::External(e.to_string()))?
}

fn run_complete_uninstall(app: InstalledApp, dry_run: bool) -> AppResult<UninstallCompleteReport> {
    use uuid::Uuid;

    // 1. Restore point
    let restore_point_seq = if !dry_run {
        let rp_input = CreateRestorePointInput {
            description: format!("ClearTool — uninstall {}", app.display_name),
            restore_type: Some(12),
            bypass_throttle: true,
        };
        crate::domain::restore::create(&rp_input)
            .ok()
            .map(|r| r.sequence_number)
    } else {
        None
    };

    // 2. Uninstall
    let uninstall_report = inventory::uninstall_app(&app, dry_run)?;

    // 3. Compute residuals (post-uninstall, some installers clean up)
    let hints = inventory::compute_residual_hints(&app);

    // 4. Clean all hints
    let all_selected = SelectedResiduals {
        appdata_roaming: hints.appdata_roaming.clone(),
        appdata_local: hints.appdata_local.clone(),
        programdata: hints.programdata.clone(),
        registry_keys: hints.registry_keys.clone(),
        shortcuts: hints
            .start_menu_shortcuts
            .iter()
            .chain(&hints.desktop_shortcuts)
            .cloned()
            .collect(),
    };
    let residuals_report = inventory::clean_residuals(&app.id, &all_selected, dry_run)?;

    // 5. Audit entry
    let audit_run_id = Uuid::new_v4().to_string();
    let reverse_recipe = match &app.source {
        AppSource::AppxPackage { family_name, .. } => {
            crate::models::restore::ReverseRecipe::AppxReinstall {
                package_family_name: family_name.clone(),
                store_url: None,
            }
        }
        _ => crate::models::restore::ReverseRecipe::Noop {
            reason: format!(
                "Win32/other uninstall — reinstall manual from publisher: {}",
                app.publisher.as_deref().unwrap_or("unknown")
            ),
        },
    };

    if !dry_run {
        let entry = crate::models::restore::AuditEntry {
            run_id: audit_run_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            module: "inventory".to_string(),
            operation: format!("uninstall_complete:{}", app.id),
            dry_run: false,
            restore_point_seq,
            items_affected: vec![app.id.clone()],
            reverse_recipe,
            status: if uninstall_report.success { "ok".to_string() } else { "partial".to_string() },
            error: uninstall_report.error.clone(),
        };
        let _ = crate::domain::audit::write_entry(&entry);
    }

    Ok(UninstallCompleteReport {
        app_id: app.id.clone(),
        display_name: app.display_name.clone(),
        dry_run,
        restore_point_seq,
        uninstall: uninstall_report,
        residuals: residuals_report,
        audit_run_id,
    })
}
