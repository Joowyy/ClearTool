// ipc/network.rs — comandos Tauri para Network Utilities.
//
// Cada operación requiere admin y crea un restore point antes.
// Audit log obligatorio.

use crate::core::{AppError, AppResult};
use crate::domain::audit::write_entry;
use crate::domain::restore;
use crate::models::restore::{AuditEntry, ReverseRecipe};
use crate::platform::powershell;
use std::path::PathBuf;

fn run_network_op(script: &str, action: &str, items_affected: Vec<String>, dry_run: bool) -> AppResult<()> {
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
        items_affected,
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
    run_network_op("ipconfig /flushdns", "flush_dns", vec!["DNS resolver cache".into()], dry_run)
}

#[tauri::command]
pub async fn renew_ip(dry_run: bool) -> AppResult<()> {
    run_network_op("ipconfig /release; ipconfig /renew", "renew_ip", vec!["IP configuration".into()], dry_run)
}

#[tauri::command]
pub async fn reset_winsock(dry_run: bool) -> AppResult<()> {
    run_network_op("netsh winsock reset", "reset_winsock", vec!["Winsock catalog".into()], dry_run)
}

#[tauri::command]
pub async fn reset_tcpip(dry_run: bool) -> AppResult<()> {
    run_network_op("netsh int ip reset", "reset_tcpip", vec!["TCP/IP stack".into()], dry_run)
}

#[tauri::command]
pub async fn reset_proxy(dry_run: bool) -> AppResult<()> {
    run_network_op("netsh winhttp reset proxy", "reset_proxy", vec!["WinHTTP proxy settings".into()], dry_run)
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
        $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $backupDir = [System.IO.Path]::GetTempPath()
        $backupPath = Join-Path $backupDir "cleartool-hosts-backup-$timestamp.txt"
        Copy-Item -Path $path -Destination $backupPath -Force
        Write-Output "BACKUP:$backupPath"

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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let backup_path = stdout
        .lines()
        .find(|l| l.starts_with("BACKUP:"))
        .map(|l| l.trim_start_matches("BACKUP:"))
        .unwrap_or("unknown");

    let backup_path_buf = PathBuf::from(backup_path);
    let hosts_path_buf = PathBuf::from(hosts_path);

    let entry = AuditEntry {
        run_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        module: "network".to_string(),
        operation: "restore_hosts_file".to_string(),
        dry_run: false,
        restore_point_seq: None,
        items_affected: vec![
            hosts_path.to_string(),
            format!("Backup: {}", backup_path),
        ],
        reverse_recipe: ReverseRecipe::Noop {
            reason: format!("Restored {} to defaults — backup at {}", hosts_path, backup_path),
        },
        status: "success".to_string(),
        error: None,
    };
    let _ = write_entry(&entry);

    let _ = backup_path_buf;
    let _ = hosts_path_buf;

    Ok(())
}
