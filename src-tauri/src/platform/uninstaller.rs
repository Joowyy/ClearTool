// platform/uninstaller.rs — búsqueda y ejecución de uninstall strings.

use crate::core::{AppError, AppResult};
use std::process::Command;

const UNINSTALL_HIVES: &[(&str, &str)] = &[
    ("HKLM", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    (
        "HKLM",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ),
    ("HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
];

pub fn find_uninstall_string(display_name_pattern: &str) -> AppResult<Option<String>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let pattern = display_name_pattern.to_lowercase();

    for (hive, key) in UNINSTALL_HIVES {
        let root = match *hive {
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            _ => continue,
        };
        let subkey = match root.open_subkey(key) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub_name in subkey.enum_keys().filter_map(Result::ok) {
            if let Ok(item) = subkey.open_subkey(&sub_name) {
                let name: Result<String, _> = item.get_value("DisplayName");
                let unstr: Result<String, _> = item.get_value("UninstallString");
                if let (Ok(n), Ok(u)) = (name, unstr) {
                    if n.to_lowercase().contains(&pattern) {
                        return Ok(Some(u));
                    }
                }
            }
        }
    }
    Ok(None)
}

pub fn run_uninstaller(uninstall_string: &str) -> AppResult<()> {
    let lower = uninstall_string.to_lowercase();
    let (program, args) = if lower.contains("msiexec.exe") {
        let s = uninstall_string
            .replace("/I", "/X")
            .replace("/i", "/X");
        (
            "cmd".to_string(),
            vec!["/C".to_string(), format!("{} /quiet /norestart", s)],
        )
    } else {
        (
            "cmd".to_string(),
            vec![
                "/C".to_string(),
                format!("{} /S /silent /quiet", uninstall_string),
            ],
        )
    };

    let status = Command::new(&program)
        .args(&args)
        .status()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    if !status.success() {
        return Err(AppError::Services(format!(
            "uninstaller exit: {:?}",
            status.code()
        )));
    }
    Ok(())
}
