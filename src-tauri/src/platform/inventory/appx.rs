// platform/inventory/appx.rs — Listado de paquetes Appx (usuario + provisionados).

use crate::core::{AppError, AppResult};
use crate::models::inventory::AppxKind;
use crate::platform::powershell;
use std::path::PathBuf;

const SCRIPT_LIST_ALL: &str = r#"
$ErrorActionPreference = 'Stop'
try {
    $user = Get-AppxPackage -AllUsers 2>$null | Select-Object Name, PackageFullName, PackageFamilyName, InstallLocation, Version, PublisherId, SignatureKind |
        ForEach-Object {
            $kind = "user"
            if ($_.SignatureKind -eq "None" -or $_.Name -match "Framework") { $kind = "framework" }
            [PSCustomObject]@{
                Name               = $_.Name
                PackageFullName    = $_.PackageFullName
                PackageFamilyName  = $_.PackageFamilyName
                InstallLocation    = $_.InstallLocation
                Version            = $_.Version
                SignatureKind      = $_.SignatureKind
                Kind               = $kind
            }
        }
    $prov = Get-AppxProvisionedPackage -Online 2>$null | Select-Object DisplayName, PackageName |
        ForEach-Object { [PSCustomObject]@{ DisplayName = $_.DisplayName; PackageName = $_.PackageName } }
    @{ user = @($user); provisioned = @($prov) } | ConvertTo-Json -Depth 5
} catch {
    @{ error = $_.Exception.Message } | ConvertTo-Json
}
"#;

#[derive(serde::Deserialize)]
struct AppxListOut {
    user: Option<Vec<AppxUserRow>>,
    provisioned: Option<Vec<AppxProvRow>>,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxUserRow {
    name: String,
    package_full_name: String,
    package_family_name: String,
    install_location: Option<String>,
    version: Option<String>,
    signature_kind: Option<String>,
    kind: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxProvRow {
    display_name: String,
    package_name: String,
}

pub struct AppxPackage {
    pub full_name: String,
    pub family_name: String,
    pub display_name: String,
    pub version: Option<String>,
    pub install_location: Option<PathBuf>,
    pub appx_kind: AppxKind,
    pub publisher: Option<String>,
}

pub struct AppxProvisionedPackage {
    pub display_name: String,
    pub full_name: String,
}

pub fn list_appx_packages() -> AppResult<Vec<AppxPackage>> {
    let out = powershell::run_script(SCRIPT_LIST_ALL)?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: AppxListOut = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("appx list: {e}")))?;
    if let Some(err) = parsed.error {
        return Err(AppError::Powershell(err));
    }
    let rows = parsed.user.unwrap_or_default();
    let packages = rows
        .into_iter()
        .map(|r| {
            let appx_kind = classify_kind(&r.name, r.kind.as_deref(), r.signature_kind.as_deref());
            let publisher = extract_publisher_from_pfn(&r.package_family_name);
            AppxPackage {
                full_name: r.package_full_name,
                family_name: r.package_family_name,
                display_name: r.name.clone(),
                version: r.version,
                install_location: r.install_location.filter(|s| !s.is_empty()).map(PathBuf::from),
                appx_kind,
                publisher,
            }
        })
        .collect();
    Ok(packages)
}

pub fn list_appx_provisioned() -> AppResult<Vec<AppxProvisionedPackage>> {
    let out = powershell::run_script(SCRIPT_LIST_ALL)?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: AppxListOut = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("appx list: {e}")))?;
    let rows = parsed.provisioned.unwrap_or_default();
    Ok(rows
        .into_iter()
        .map(|r| AppxProvisionedPackage {
            display_name: r.display_name,
            full_name: r.package_name,
        })
        .collect())
}

fn classify_kind(name: &str, kind_hint: Option<&str>, sig: Option<&str>) -> AppxKind {
    let name_lower = name.to_lowercase();
    if name_lower.contains("framework") || name_lower.contains("vclibs") || name_lower.contains("runtime") {
        return AppxKind::Framework;
    }
    if sig == Some("None") {
        return AppxKind::User;
    }
    match kind_hint {
        Some("framework") => AppxKind::Framework,
        Some("bundle") => AppxKind::Bundle,
        _ => AppxKind::User,
    }
}

fn extract_publisher_from_pfn(pfn: &str) -> Option<String> {
    // PackageFamilyName = "Microsoft.WindowsCalculator_8wekyb3d8bbwe"
    // El prefijo es el nombre del paquete, que incluye el publisher en el nombre
    pfn.split('_').next().and_then(|n| {
        let parts: Vec<&str> = n.split('.').collect();
        if parts.len() >= 2 {
            Some(parts[0].to_string())
        } else {
            None
        }
    })
}
