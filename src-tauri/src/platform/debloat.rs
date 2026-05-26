// platform/debloat.rs — wrapper Rust para scripts PowerShell de Appx.

use crate::core::{AppError, AppResult};
use crate::platform::powershell;
use std::process::Command;

const SCRIPT_LIST: &str = include_str!("ps-scripts/list-appx.ps1");
const SCRIPT_REMOVE_USER: &str = include_str!("ps-scripts/remove-appx-user.ps1");
const SCRIPT_REMOVE_PROV: &str = include_str!("ps-scripts/remove-appx-provisioned.ps1");

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxUserRow {
    name: String,
    #[serde(rename = "PackageFamilyName")]
    package_family_name: String,
    install_location: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxProvRow {
    display_name: String,
    package_name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AppxListOutput {
    user: Vec<AppxUserRow>,
    provisional: Vec<AppxProvRow>,
}

pub struct AppxUser {
    pub name: String,
    pub package_family_name: String,
    pub install_location: Option<String>,
    pub version: Option<String>,
}

pub struct AppxProv {
    pub display_name: String,
    pub package_name: String,
}

pub fn list_appx() -> AppResult<(Vec<AppxUser>, Vec<AppxProv>)> {
    let out = powershell::run_script(SCRIPT_LIST)?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: AppxListOutput = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("list-appx: {}", e)))?;
    let users = parsed
        .user
        .into_iter()
        .map(|r| AppxUser {
            name: r.name,
            package_family_name: r.package_family_name,
            install_location: r.install_location,
            version: r.version,
        })
        .collect();
    let provs = parsed
        .provisional
        .into_iter()
        .map(|r| AppxProv {
            display_name: r.display_name,
            package_name: r.package_name,
        })
        .collect();
    Ok((users, provs))
}

pub fn remove_appx_user(package_family_name: &str) -> AppResult<String> {
    validate_pkg_family_name(package_family_name)?;
    let out = run_with_env(SCRIPT_REMOVE_USER, &[("CT_APPX_PFN", package_family_name)])?;
    extract_status(&out)
}

pub fn remove_appx_provisioned(provisioned_name: &str) -> AppResult<String> {
    validate_prov_name(provisioned_name)?;
    let out = run_with_env(SCRIPT_REMOVE_PROV, &[("CT_APPX_PROV_NAME", provisioned_name)])?;
    extract_status(&out)
}

fn validate_pkg_family_name(s: &str) -> AppResult<()> {
    let re = regex::Regex::new(r"^[A-Za-z0-9_.-]+_[A-Za-z0-9]{13}$").unwrap();
    if !re.is_match(s) {
        return Err(AppError::Validation(format!(
            "PackageFamilyName inválido: {}",
            s
        )));
    }
    Ok(())
}

fn validate_prov_name(s: &str) -> AppResult<()> {
    let re = regex::Regex::new(r"^[A-Za-z0-9_.]+$").unwrap();
    if !re.is_match(s) {
        return Err(AppError::Validation(format!(
            "ProvisionedName inválido: {}",
            s
        )));
    }
    Ok(())
}

fn run_with_env(script: &'static str, vars: &[(&str, &str)]) -> AppResult<std::process::Output> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    for (k, v) in vars {
        cmd.env(k, v);
    }
    cmd.output()
        .map_err(|e| AppError::Powershell(e.to_string()))
}

#[derive(serde::Deserialize)]
struct PsResult {
    ok: bool,
    status: Option<String>,
    error: Option<String>,
}

fn extract_status(out: &std::process::Output) -> AppResult<String> {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: PsResult = serde_json::from_str(stdout.trim()).map_err(|e| {
        AppError::Parse(format!("ps result: {}: {}", e, stdout.trim()))
    })?;
    if !parsed.ok {
        return Err(AppError::Powershell(
            parsed.error.unwrap_or_else(|| "unknown".into()),
        ));
    }
    Ok(parsed.status.unwrap_or_else(|| "ok".into()))
}
