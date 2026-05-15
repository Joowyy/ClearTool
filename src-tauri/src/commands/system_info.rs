// commands/system_info.rs — información general del sistema y elevación.

use crate::elevation;
use crate::error::{AppError, AppResult};
use crate::models::system::{DriveInfo, SystemSummary};
use std::env;

#[tauri::command]
pub async fn is_elevated() -> bool {
    elevation::is_elevated()
}

#[tauri::command]
pub async fn system_summary() -> AppResult<SystemSummary> {
    let os_name = get_os_name();
    let os_version = get_os_version();
    let build_number = get_build_number();
    let username = get_username();
    let is_elevated = elevation::is_elevated();
    let total_ram_bytes = get_total_ram();
    let drives = get_drives_info()?;

    Ok(SystemSummary {
        os_name,
        os_version,
        build_number,
        username,
        is_elevated,
        total_ram_bytes,
        drives,
    })
}

#[cfg(target_os = "windows")]
fn get_os_name() -> String {
    "Windows".to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_os_name() -> String {
    std::env::consts::OS.to_string()
}

#[cfg(target_os = "windows")]
fn get_os_version() -> String {
    use std::process::Command;
    
    Command::new("cmd")
        .args(&["/C", "ver"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_os_version() -> String {
    "Unknown".to_string()
}

#[cfg(target_os = "windows")]
fn get_build_number() -> String {
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
fn get_build_number() -> String {
    "Unknown".to_string()
}

fn get_username() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

#[cfg(target_os = "windows")]
fn get_total_ram() -> u64 {
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
fn get_total_ram() -> u64 {
    use std::process::Command;
    
    Command::new("grep")
        .args(&["MemTotal", "/proc/meminfo"])
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .ok()
                .and_then(|s| {
                    s.split_whitespace()
                        .nth(1)
                        .and_then(|num| num.parse::<u64>().ok())
                        .map(|kb| kb * 1024)
                })
        })
        .unwrap_or(0)
}

#[cfg(target_os = "windows")]
fn get_drives_info() -> AppResult<Vec<DriveInfo>> {
    use std::fs;
    use std::path::Path;
    
    let mut drives = Vec::new();
    
    for letter in b'A'..=b'Z' {
        let drive_letter = (letter as char).to_string();
        let path_str = format!("{}:\\", drive_letter);
        let path = Path::new(&path_str);
        
        if path.exists() {
            if let Ok(metadata) = fs::metadata(path) {
                let total = get_drive_total_bytes(&path_str).unwrap_or(0);
                let free = get_drive_free_bytes(&path_str).unwrap_or(0);
                
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
fn get_drives_info() -> AppResult<Vec<DriveInfo>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
fn get_drive_total_bytes(drive: &str) -> Option<u64> {
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExA;
    use std::ffi::CString;
    
    let drive_cstr = CString::new(drive).ok()?;
    let mut total_bytes: u64 = 0;
    
    unsafe {
        if GetDiskFreeSpaceExA(
            drive_cstr.as_ptr() as *const _,
            None,
            Some(&mut total_bytes),
            None,
        )
        .into()
        {
            Some(total_bytes)
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_drive_total_bytes(_drive: &str) -> Option<u64> {
    None
}

#[cfg(target_os = "windows")]
fn get_drive_free_bytes(drive: &str) -> Option<u64> {
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExA;
    use std::ffi::CString;
    
    let drive_cstr = CString::new(drive).ok()?;
    let mut free_bytes: u64 = 0;
    
    unsafe {
        if GetDiskFreeSpaceExA(
            drive_cstr.as_ptr() as *const _,
            Some(&mut free_bytes),
            None,
            None,
        )
        .into()
        {
            Some(free_bytes)
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_drive_free_bytes(_drive: &str) -> Option<u64> {
    None
}
