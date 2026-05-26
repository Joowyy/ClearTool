// platform/restore_point.rs — adaptador a SRSetRestorePointW + WMI.
//
// En Windows: create/list/restore vía SRSetRestorePointW y PowerShell+WMI.
// En Linux: stub que devuelve error "no disponible".

use crate::core::{AppError, AppResult};

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use crate::platform::powershell;
    use windows::Win32::System::Restore::{
        BEGIN_NESTED_SYSTEM_CHANGE, END_NESTED_SYSTEM_CHANGE, RESTOREPOINTINFOW,
        SRSetRestorePointW, STATEMGRSTATUS,
    };
    use winreg::enums::*;
    use winreg::RegKey;

    const RESTORE_FREQ_KEY: &str =
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
    const RESTORE_FREQ_VAL: &str = "SystemRestorePointCreationFrequency";

    const LIST_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-ComputerRestorePoint | Select-Object SequenceNumber, Description, CreationTime, RestorePointType, EventType | ConvertTo-Json -Compress -Depth 2
"#;

    const CHECK_ENABLED_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$d = Get-WmiObject -Namespace 'root\default' -Class SystemRestoreConfig -ErrorAction SilentlyContinue
if ($null -eq $d) {
    @{ enabled = $false; reason = 'wmi-class-missing' } | ConvertTo-Json -Compress
    return
}
@{ enabled = ($d.RPSessionInterval -gt 0); reason = 'ok' } | ConvertTo-Json -Compress
"#;

    const ENABLE_SCRIPT: &str = "Enable-ComputerRestore -Drive 'C:\\'";

    #[derive(serde::Deserialize)]
    struct PsRestorePoint {
        #[serde(rename = "SequenceNumber")]
        sequence_number: u32,
        #[serde(rename = "Description")]
        description: Option<String>,
        #[serde(rename = "CreationTime")]
        creation_time: String,
        #[serde(rename = "RestorePointType")]
        restore_point_type: u32,
        #[serde(rename = "EventType")]
        event_type: u32,
    }

    #[derive(serde::Deserialize)]
    struct EnabledCheck {
        enabled: bool,
        #[allow(dead_code)]
        reason: String,
    }

    pub fn create(description: &str, restore_type: u32, bypass_throttle: bool) -> AppResult<u32> {
        let original_freq = if bypass_throttle {
            set_throttle_bypass()?
        } else {
            None
        };

        let result = create_inner(description, restore_type);

        if bypass_throttle {
            restore_throttle(original_freq);
        }

        result
    }

    fn create_inner(description: &str, restore_type: u32) -> AppResult<u32> {
        use windows::Win32::System::Restore::{
            RESTOREPOINTINFO_EVENT_TYPE, RESTOREPOINTINFO_TYPE,
        };

        let desc_wide: Vec<u16> = description.encode_utf16().chain([0]).collect();
        let copy_len = desc_wide.len().min(256);

        let mut info: RESTOREPOINTINFOW = unsafe { std::mem::zeroed() };
        info.dwEventType = RESTOREPOINTINFO_EVENT_TYPE(BEGIN_NESTED_SYSTEM_CHANGE.0);
        info.dwRestorePtType = RESTOREPOINTINFO_TYPE(restore_type);
        info.llSequenceNumber = 0;

        unsafe {
            let desc_ptr = std::ptr::addr_of_mut!(info.szDescription) as *mut u16;
            std::ptr::copy_nonoverlapping(desc_wide.as_ptr(), desc_ptr, copy_len);
        }

        let mut status = STATEMGRSTATUS::default();

        let ok = unsafe { SRSetRestorePointW(&info, &mut status) };
        if !ok.as_bool() {
            let n_status = status.nStatus.0;
            return Err(AppError::RestorePoint(format!(
                "SRSetRestorePointW failed: nStatus={}",
                n_status
            )));
        }

        let mut close_info: RESTOREPOINTINFOW = unsafe { std::mem::zeroed() };
        close_info.dwEventType = RESTOREPOINTINFO_EVENT_TYPE(END_NESTED_SYSTEM_CHANGE.0);
        close_info.dwRestorePtType = RESTOREPOINTINFO_TYPE(restore_type);
        close_info.llSequenceNumber = status.llSequenceNumber;

        let mut _close_status = STATEMGRSTATUS::default();
        let _ = unsafe { SRSetRestorePointW(&close_info, &mut _close_status) };

        Ok(status.llSequenceNumber as u32)
    }

    fn set_throttle_bypass() -> AppResult<Option<u32>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm
            .create_subkey(RESTORE_FREQ_KEY)
            .map_err(|e| AppError::Registry(format!("open {}: {}", RESTORE_FREQ_KEY, e)))?;
        let previous: Option<u32> = key.get_value(RESTORE_FREQ_VAL).ok();
        key.set_value(RESTORE_FREQ_VAL, &0u32)
            .map_err(|e| AppError::Registry(format!("set {}: {}", RESTORE_FREQ_VAL, e)))?;
        Ok(previous)
    }

    fn restore_throttle(previous: Option<u32>) {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = match hklm.open_subkey_with_flags(RESTORE_FREQ_KEY, KEY_SET_VALUE) {
            Ok(k) => k,
            Err(_) => return,
        };
        match previous {
            Some(v) => {
                let _ = key.set_value(RESTORE_FREQ_VAL, &v);
            }
            None => {
                let _ = key.delete_value(RESTORE_FREQ_VAL);
            }
        }
    }

    pub fn list() -> AppResult<Vec<crate::models::restore::RestorePoint>> {
        let out = powershell::run_script(LIST_SCRIPT)?;
        if !out.status.success() {
            return Err(AppError::Powershell(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let parsed: serde_json::Value = serde_json::from_str(trimmed)
            .map_err(|e| AppError::Parse(format!("list restore: {}", e)))?;
        let items: Vec<PsRestorePoint> = match parsed {
            serde_json::Value::Array(_) => serde_json::from_value(parsed)
                .map_err(|e| AppError::Parse(format!("array: {}", e)))?,
            serde_json::Value::Object(_) => vec![serde_json::from_value(parsed)
                .map_err(|e| AppError::Parse(format!("single: {}", e)))?],
            _ => Vec::new(),
        };

        Ok(items
            .into_iter()
            .map(|p| crate::models::restore::RestorePoint {
                sequence_number: p.sequence_number,
                description: p.description.unwrap_or_default(),
                creation_time: parse_wmi_datetime(&p.creation_time),
                restore_point_type: p.restore_point_type,
                event_type: p.event_type,
            })
            .collect())
    }

    pub fn restore_to(sequence_number: u32) -> AppResult<()> {
        let script =
            format!("Restore-Computer -RestorePoint {} -Confirm:$false", sequence_number);
        let out = powershell::run_script_owned(&script)?;
        if !out.status.success() {
            return Err(AppError::Powershell(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(())
    }

    pub fn is_enabled() -> AppResult<bool> {
        let out = powershell::run_script(CHECK_ENABLED_SCRIPT)?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let chk: EnabledCheck = serde_json::from_str(stdout.trim())
            .map_err(|e| AppError::Parse(format!("is_enabled: {}", e)))?;
        Ok(chk.enabled)
    }

    pub fn enable_for_system_drive() -> AppResult<()> {
        let out = powershell::run_script(ENABLE_SCRIPT)?;
        if !out.status.success() {
            return Err(AppError::Powershell(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(())
    }

    fn parse_wmi_datetime(s: &str) -> String {
        if s.len() < 14 {
            return s.to_string();
        }
        let year = &s[0..4];
        let month = &s[4..6];
        let day = &s[6..8];
        let h = &s[8..10];
        let m = &s[10..12];
        let sec = &s[12..14];
        format!("{}-{}-{}T{}:{}:{}Z", year, month, day, h, m, sec)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_wmi_datetime_basico() {
            let s = "20260521143200.000000-300";
            assert_eq!(parse_wmi_datetime(s), "2026-05-21T14:32:00Z");
        }
    }
}

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
pub fn create(_description: &str, _restore_type: u32, _bypass_throttle: bool) -> AppResult<u32> {
    Err(AppError::RestoreUnavailable(
        "System Restore es una función exclusiva de Windows".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn list() -> AppResult<Vec<crate::models::restore::RestorePoint>> {
    Err(AppError::RestoreUnavailable(
        "System Restore es una función exclusiva de Windows".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn restore_to(_sequence_number: u32) -> AppResult<()> {
    Err(AppError::RestoreUnavailable(
        "System Restore es una función exclusiva de Windows".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn is_enabled() -> AppResult<bool> {
    Ok(false)
}

#[cfg(not(windows))]
pub fn enable_for_system_drive() -> AppResult<()> {
    Err(AppError::RestoreUnavailable(
        "System Restore es una función exclusiva de Windows".to_string(),
    ))
}
