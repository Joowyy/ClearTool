// platform/inventory/uninstall_registry.rs — Lectura del árbol Uninstall del registro.
//
// Hives escaneados:
//   HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*
//   HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*
//   HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\*

use std::path::PathBuf;
use winreg::{RegKey, enums::*};

pub struct Win32App {
    pub registry_key: String,
    pub hive: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_location: Option<PathBuf>,
    pub install_date: Option<String>,
    pub size_bytes: Option<u64>,
    pub uninstall_string: Option<String>,
    pub quiet_uninstall_string: Option<String>,
    pub is_msi: bool,
    pub product_code: Option<String>,
    pub is_system_component: bool,
    pub requires_admin: bool,
}

const UNINSTALL_PATHS: &[(&str, isize, &str)] = &[
    (
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_LOCAL_MACHINE,
        "HKLM",
    ),
    (
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_LOCAL_MACHINE,
        "HKLM (WOW64)",
    ),
    (
        "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_CURRENT_USER,
        "HKCU",
    ),
];

pub fn list_win32_apps() -> Vec<Win32App> {
    let mut result = Vec::new();

    for (path, hive_raw, hive_label) in UNINSTALL_PATHS {
        let root = if *hive_raw == HKEY_LOCAL_MACHINE {
            RegKey::predef(HKEY_LOCAL_MACHINE)
        } else if *hive_raw == HKEY_CURRENT_USER {
            RegKey::predef(HKEY_CURRENT_USER)
        } else {
            continue
        };
        let Ok(base) = root.open_subkey(path) else {
            continue;
        };
        for sub_name in base.enum_keys().flatten() {
            let Ok(sub) = base.open_subkey(&sub_name) else {
                continue;
            };

            // Descartar componentes del sistema
            let system_component: u32 = sub.get_value("SystemComponent").unwrap_or(0);
            if system_component == 1 {
                continue;
            }

            let display_name: String = match sub.get_value("DisplayName") {
                Ok(v) => v,
                Err(_) => continue, // Sin nombre → descartado
            };
            if display_name.trim().is_empty() {
                continue;
            }

            let publisher: Option<String> = sub.get_value("Publisher").ok();
            let version: Option<String> = sub.get_value("DisplayVersion").ok();
            let install_location: Option<PathBuf> = sub
                .get_value::<String, _>("InstallLocation")
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from);
            let install_date: Option<String> = sub.get_value("InstallDate").ok();
            let size_kb: Option<u32> = sub.get_value("EstimatedSize").ok();
            let size_bytes = size_kb.map(|kb| kb as u64 * 1024);

            let uninstall_string: Option<String> = sub.get_value("UninstallString").ok();
            let quiet_uninstall_string: Option<String> =
                sub.get_value("QuietUninstallString").ok();

            // Detectar MSI: WindowsInstaller == 1
            let windows_installer: u32 = sub.get_value("WindowsInstaller").unwrap_or(0);
            let is_msi = windows_installer == 1;

            // Extraer GUID del producto del UninstallString si es MSI
            let product_code = if is_msi {
                uninstall_string.as_ref().and_then(|s| extract_msi_guid(s))
            } else {
                None
            };

            // Heurística: ¿requiere admin? Si es HKLM o MSI, sí.
            let requires_admin = (*hive_raw == HKEY_LOCAL_MACHINE) || is_msi;

            let registry_key = format!("{}\\{}", path, sub_name);

            result.push(Win32App {
                registry_key,
                hive: hive_label.to_string(),
                display_name,
                publisher,
                version,
                install_location,
                install_date,
                size_bytes,
                uninstall_string,
                quiet_uninstall_string,
                is_msi,
                product_code,
                is_system_component: false,
                requires_admin,
            });
        }
    }

    // Deduplicar por display_name + publisher (WOW64 suele duplicar entradas)
    dedup_by_name(result)
}

fn extract_msi_guid(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"\{[A-Fa-f0-9\-]{36}\}").unwrap();
    re.find(s).map(|m| m.as_str().to_string())
}

fn dedup_by_name(apps: Vec<Win32App>) -> Vec<Win32App> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(apps.len());
    for app in apps {
        let key = format!("{}|{}", app.display_name.to_lowercase(), app.publisher.as_deref().unwrap_or(""));
        if seen.insert(key) {
            out.push(app);
        }
    }
    out
}
