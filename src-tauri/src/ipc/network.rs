// ipc/network.rs — comandos Tauri para Network Utilities.
//
// Cada operación requiere admin y crea un restore point antes.
// Audit log obligatorio.

use crate::core::{AppError, AppResult};
use crate::domain::audit::write_entry;
use crate::domain::restore;
use crate::models::restore::{AuditEntry, ReverseRecipe};
use crate::platform::powershell;

fn run_network_op(script: &str, action: &str, dry_run: bool) -> AppResult<()> {
    if dry_run {
        return Ok(());
    }

    let desc = format!("Before: {}", action);
    restore::create_restore_point(&desc)?;

    let output = powershell::run_script_owned(script)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Powershell(format!(
            "{} failed: {}",
            action, stderr
        )));
    }

    let entry = AuditEntry {
        run_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        module: "network".to_string(),
        operation: action.to_string(),
        dry_run: false,
        restore_point_seq: None,
        items_affected: vec![script.to_string()],
        reverse_recipe: ReverseRecipe::Noop {
            reason: format!("Network operation {} — manual revert may be needed", action),
        },
        status: "success".to_string(),
        error: None,
    };
    let _ = write_entry(&entry);

    Ok(())
}

#[tauri::command]
pub async fn flush_dns(dry_run: bool) -> AppResult<()> {
    run_network_op("ipconfig /flushdns", "flush_dns", dry_run)
}

#[tauri::command]
pub async fn renew_ip(dry_run: bool) -> AppResult<()> {
    run_network_op("ipconfig /release; ipconfig /renew", "renew_ip", dry_run)
}

#[tauri::command]
pub async fn reset_winsock(dry_run: bool) -> AppResult<()> {
    run_network_op("netsh winsock reset", "reset_winsock", dry_run)
}

#[tauri::command]
pub async fn reset_tcpip(dry_run: bool) -> AppResult<()> {
    run_network_op("netsh int ip reset", "reset_tcpip", dry_run)
}

#[tauri::command]
pub async fn reset_proxy(dry_run: bool) -> AppResult<()> {
    run_network_op("netsh winhttp reset proxy", "reset_proxy", dry_run)
}

#[tauri::command]
pub async fn restore_hosts_file(dry_run: bool) -> AppResult<()> {
    if dry_run {
        return Ok(());
    }

    restore::create_restore_point("Before: Restore hosts file")?;

    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    let script = format!(
        r#"
        $path = "{}"
        $defaultContent = @"
# Copyright (c) 1993-2009 Microsoft Corp.
#
# This is a sample HOSTS file used by Microsoft TCP/IP for Windows.
#
# localhost name resolution is handled within DNS itself.
#	127.0.0.1       localhost
#	::1             localhost
"@
        Set-Content -Path $path -Value $defaultContent -Encoding UTF8 -Force
        "#,
        hosts_path
    );

    let output = powershell::run_script_owned(&script)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Powershell(format!(
            "restore_hosts_file failed: {}",
            stderr
        )));
    }

    let entry = AuditEntry {
        run_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        module: "network".to_string(),
        operation: "restore_hosts_file".to_string(),
        dry_run: false,
        restore_point_seq: None,
        items_affected: vec![hosts_path.to_string()],
        reverse_recipe: ReverseRecipe::Noop {
            reason: format!("Restored {} to defaults — manual edit if needed", hosts_path),
        },
        status: "success".to_string(),
        error: None,
    };
    let _ = write_entry(&entry);

    Ok(())
}
