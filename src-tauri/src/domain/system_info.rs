// domain/system_info.rs — info general del sistema y elevación.
//
// Concentra lo que antes vivía mezclado entre `commands/system_info.rs`
// y `elevation.rs`. Aquí se decide qué exponer; el FFI (registro, RAM,
// drives) vive en `platform`.

use crate::core::AppResult;
use crate::models::system::{DriveInfo, SystemSummary};
use crate::platform::elevation;
use std::env;

pub fn is_elevated() -> bool {
    elevation::is_elevated()
}

pub fn summary() -> AppResult<SystemSummary> {
    Ok(SystemSummary {
        os_name: os_name(),
        os_version: os_version(),
        build_number: build_number(),
        username: username(),
        is_elevated: is_elevated(),
        total_ram_bytes: total_ram_bytes(),
        drives: drives_info()?,
    })
}

#[cfg(target_os = "windows")]
fn os_name() -> String {
    "Windows".to_string()
}

#[cfg(not(target_os = "windows"))]
fn os_name() -> String {
    std::env::consts::OS.to_string()
}

#[cfg(target_os = "windows")]
fn os_version() -> String {
    use std::process::Command;
    Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
fn os_version() -> String {
    "Unknown".to_string()
}

#[cfg(target_os = "windows")]
fn build_number() -> String {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        Ok(key) => key
            .get_value::<String, _>("CurrentBuildNumber")
            .unwrap_or_else(|_| "Unknown".to_string()),
        Err(_) => "Unknown".to_string(),
    }
}

#[cfg(not(target_os = "windows"))]
fn build_number() -> String {
    "Unknown".to_string()
}

fn username() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

#[cfg(target_os = "windows")]
fn total_ram_bytes() -> u64 {
    use windows::Win32::System::SystemInformation::GetPhysicallyInstalledSystemMemory;
    let mut ram_kb: u64 = 0;
    unsafe {
        if GetPhysicallyInstalledSystemMemory(&mut ram_kb).is_ok() {
            ram_kb * 1024
        } else {
            0
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn total_ram_bytes() -> u64 {
    0
}

#[cfg(target_os = "windows")]
fn drives_info() -> AppResult<Vec<DriveInfo>> {
    use std::path::Path;
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive_letter = (letter as char).to_string();
        let path_str = format!("{}:\\", drive_letter);
        if Path::new(&path_str).exists() {
            let total = drive_total_bytes(&path_str).unwrap_or(0);
            let free = drive_free_bytes(&path_str).unwrap_or(0);
            if total > 0 || free > 0 {
                drives.push(DriveInfo {
                    letter: drive_letter,
                    total_bytes: total,
                    free_bytes: free,
                    label: String::new(),
                });
            }
        }
    }
    Ok(drives)
}

#[cfg(not(target_os = "windows"))]
fn drives_info() -> AppResult<Vec<DriveInfo>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
fn drive_total_bytes(drive: &str) -> Option<u64> {
    use windows::core::PCSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExA;
    let drive_bytes = drive.as_bytes();
    let mut total: u64 = 0;
    unsafe {
        if GetDiskFreeSpaceExA(PCSTR(drive_bytes.as_ptr()), None, Some(&mut total), None).is_ok() {
            Some(total)
        } else {
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn drive_free_bytes(drive: &str) -> Option<u64> {
    use windows::core::PCSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExA;
    let drive_bytes = drive.as_bytes();
    let mut free: u64 = 0;
    unsafe {
        if GetDiskFreeSpaceExA(PCSTR(drive_bytes.as_ptr()), Some(&mut free), None, None).is_ok() {
            Some(free)
        } else {
            None
        }
    }
}
