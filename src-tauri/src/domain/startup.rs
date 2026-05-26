// domain/startup.rs — agregador de auto-arranque + disable/enable reversible.

use crate::core::{AppError, AppResult};
use crate::domain::audit;
use crate::models::restore::ReverseRecipe;
use crate::models::startup::{
    StartupCategory, StartupEntry, StartupImpact, StartupOrigin,
};
use crate::platform::startup as platform;

pub fn list_all() -> AppResult<Vec<StartupEntry>> {
    let mut entries = Vec::new();
    entries.extend(platform::list_registry_run_entries());
    entries.extend(platform::list_startup_folder_entries());
    entries.extend(platform::list_logon_scheduled_tasks());
    entries.extend(platform::list_automatic_services());

    for e in &mut entries {
        if matches!(e.category, StartupCategory::Unknown) {
            e.category = classify_by_name(&e.display_name, &e.command);
        }
        if matches!(e.impact, StartupImpact::Unknown) {
            e.impact = estimate_impact(e);
        }
    }

    Ok(entries)
}

pub fn disable_startup(id: &str) -> AppResult<()> {
    let entries = list_all()?;
    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::Validation(format!("startup entry no encontrada: {}", id)))?;

    match &entry.origin {
        StartupOrigin::Registry { hive, key, name } => {
            disable_registry_entry(hive, key, name)?;
        }
        StartupOrigin::StartupFolder { lnk_path } => {
            disable_lnk_entry(std::path::Path::new(lnk_path))?;
        }
        StartupOrigin::ScheduledTask { task_path } => {
            disable_scheduled_task(task_path)?;
        }
        StartupOrigin::Service { service_name } => {
            crate::platform::services::set_start_type(service_name, "Disabled")?;
        }
        StartupOrigin::UwpAutoStart { .. } => {
            return Err(AppError::Validation(
                "UWP autostart disable aún no soportado".into(),
            ));
        }
    }

    let audit_entry = audit::make_entry(
        "startup",
        "disable",
        false,
        None,
        vec![id.to_string()],
        ReverseRecipe::Noop {
            reason: "Reactivar manualmente desde Arranque".into(),
        },
        "success",
        None,
    );
    let _ = audit::write_entry(&audit_entry);

    Ok(())
}

pub fn enable_startup(id: &str) -> AppResult<()> {
    let entries = list_all()?;
    let _entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::Validation(format!("startup entry no encontrada: {}", id)))?;

    match &entries.iter().find(|e| e.id == id).unwrap().origin {
        StartupOrigin::Registry { hive, key, name } => {
            enable_registry_entry(hive, key, name)?;
        }
        StartupOrigin::StartupFolder { lnk_path } => {
            enable_lnk_entry(std::path::Path::new(lnk_path))?;
        }
        StartupOrigin::ScheduledTask { task_path } => {
            enable_scheduled_task(task_path)?;
        }
        StartupOrigin::Service { service_name } => {
            crate::platform::services::set_start_type(service_name, "Automatic")?;
        }
        StartupOrigin::UwpAutoStart { .. } => {
            return Err(AppError::Validation(
                "UWP autostart enable aún no soportado".into(),
            ));
        }
    }

    let audit_entry = audit::make_entry(
        "startup",
        "enable",
        false,
        None,
        vec![id.to_string()],
        ReverseRecipe::Noop {
            reason: "Deshabilitar manualmente desde Arranque".into(),
        },
        "success",
        None,
    );
    let _ = audit::write_entry(&audit_entry);

    Ok(())
}

#[cfg(windows)]
fn disable_registry_entry(hive: &str, _key: &str, name: &str) -> AppResult<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let root = match hive {
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return Err(AppError::Validation(format!("hive inválido: {}", hive))),
    };

    let approved_key_path = match hive {
        "HKCU" => r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
        "HKLM" => r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
        _ => unreachable!(),
    };

    let (approved_key, _) = root.create_subkey(approved_key_path)
        .map_err(|e| AppError::Registry(format!("create StartupApproved: {}", e)))?;

    let disabled_bytes: Vec<u8> = vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    approved_key.set_raw_value(name, &winreg::RegValue {
        bytes: disabled_bytes,
        vtype: REG_BINARY,
    }).map_err(|e| AppError::Registry(format!("set disabled flag: {}", e)))?;

    Ok(())
}

#[cfg(windows)]
fn enable_registry_entry(hive: &str, _key: &str, name: &str) -> AppResult<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let root = match hive {
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return Err(AppError::Validation(format!("hive inválido: {}", hive))),
    };

    let approved_key_path = match hive {
        "HKCU" => r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
        "HKLM" => r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
        _ => unreachable!(),
    };

    let approved_key = root.open_subkey_with_flags(approved_key_path, KEY_ALL_ACCESS)
        .map_err(|e| AppError::Registry(format!("open StartupApproved: {}", e)))?;

    let enabled_bytes: Vec<u8> = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    approved_key.set_raw_value(name, &winreg::RegValue {
        bytes: enabled_bytes,
        vtype: REG_BINARY,
    }).map_err(|e| AppError::Registry(format!("set enabled flag: {}", e)))?;

    Ok(())
}

#[cfg(not(windows))]
fn disable_registry_entry(_hive: &str, _key: &str, _name: &str) -> AppResult<()> {
    Err(AppError::NotImplemented("Windows only".into()))
}

#[cfg(not(windows))]
fn enable_registry_entry(_hive: &str, _key: &str, _name: &str) -> AppResult<()> {
    Err(AppError::NotImplemented("Windows only".into()))
}

fn disable_lnk_entry(path: &std::path::Path) -> AppResult<()> {
    let new_path = path.with_extension("lnk.disabled");
    std::fs::rename(path, &new_path)
        .map_err(|e| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("rename {}: {}", path.display(), e),
        )))
}

fn enable_lnk_entry(disabled_path: &std::path::Path) -> AppResult<()> {
    let original = disabled_path.with_extension("lnk");
    std::fs::rename(disabled_path, &original)
        .map_err(|e| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("rename {}: {}", disabled_path.display(), e),
        )))
}

fn disable_scheduled_task(task_path_name: &str) -> AppResult<()> {
    let escaped = task_path_name.replace('"', "");
    let path_part = std::path::Path::new(&escaped)
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let name_part = std::path::Path::new(&escaped)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let script = format!(
        r#"Disable-ScheduledTask -TaskPath '{}' -TaskName '{}'"#,
        path_part, name_part
    );
    let out = crate::platform::powershell::run_script_owned(&script)?;
    if !out.status.success() {
        return Err(AppError::Powershell(
            String::from_utf8_lossy(&out.stderr).into(),
        ));
    }
    Ok(())
}

fn enable_scheduled_task(task_path_name: &str) -> AppResult<()> {
    let escaped = task_path_name.replace('"', "");
    let path_part = std::path::Path::new(&escaped)
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let name_part = std::path::Path::new(&escaped)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let script = format!(
        r#"Enable-ScheduledTask -TaskPath '{}' -TaskName '{}'"#,
        path_part, name_part
    );
    let out = crate::platform::powershell::run_script_owned(&script)?;
    if !out.status.success() {
        return Err(AppError::Powershell(
            String::from_utf8_lossy(&out.stderr).into(),
        ));
    }
    Ok(())
}

fn classify_by_name(name: &str, cmd: &str) -> StartupCategory {
    let n = name.to_lowercase();
    let c = cmd.to_lowercase();
    let combined = format!("{} {}", n, c);

    if combined.contains("update") {
        return StartupCategory::Updater;
    }
    if combined.contains("onedrive")
        || combined.contains("dropbox")
        || combined.contains("googledrive")
        || combined.contains("icloud")
    {
        return StartupCategory::CloudSync;
    }
    if combined.contains("discord")
        || combined.contains("teams")
        || combined.contains("slack")
        || combined.contains("zoom")
        || combined.contains("skype")
        || combined.contains("whatsapp")
    {
        return StartupCategory::Communication;
    }
    if combined.contains("spotify")
        || combined.contains("itunes")
        || combined.contains("music")
        || combined.contains("steam")
        || combined.contains("epic")
    {
        return StartupCategory::Media;
    }
    if combined.contains("nvidia")
        || combined.contains("amd")
        || combined.contains("realtek")
        || combined.contains("intel")
        || combined.contains("driver")
    {
        return StartupCategory::Driver;
    }
    if combined.contains("defender")
        || combined.contains("antivirus")
        || combined.contains("firewall")
        || combined.contains("security")
    {
        return StartupCategory::Security;
    }
    if combined.contains("widget") || combined.contains("gadget") {
        return StartupCategory::Widget;
    }
    if combined.contains("launcher") || combined.contains("hub") {
        return StartupCategory::Launcher;
    }

    StartupCategory::UserApp
}

fn estimate_impact(entry: &StartupEntry) -> StartupImpact {
    match entry.category {
        StartupCategory::Updater => StartupImpact::Low,
        StartupCategory::Launcher | StartupCategory::Widget | StartupCategory::CloudSync => {
            StartupImpact::Medium
        }
        StartupCategory::Communication
        | StartupCategory::Media
        | StartupCategory::Driver
        | StartupCategory::Security => StartupImpact::High,
        _ => StartupImpact::Unknown,
    }
}
