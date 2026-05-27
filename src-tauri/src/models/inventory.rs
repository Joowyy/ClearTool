// models/inventory.rs — DTOs del inventario universal de aplicaciones.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub id: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub source: AppSource,
    pub install_location: Option<PathBuf>,
    pub install_date: Option<String>,
    pub size_bytes: Option<u64>,
    pub uninstall_method: UninstallMethod,
    pub is_system_critical: bool,
    pub catalog_match: Option<CatalogMatch>,
    pub residual_hints: ResidualHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppSource {
    AppxPackage {
        full_name: String,
        family_name: String,
        appx_kind: AppxKind,
    },
    AppxProvisioned {
        full_name: String,
    },
    Win32Uninstaller {
        registry_key: String,
        hive: String,
    },
    Steam {
        app_id: u64,
        library_path: PathBuf,
    },
    EpicGames {
        catalog_item_id: String,
        manifest_path: PathBuf,
    },
    Gog {
        game_id: u64,
    },
    Xbox {
        package_family_name: String,
        msstore_id: Option<String>,
    },
    Winget {
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AppxKind {
    User,
    Provisioned,
    Framework,
    Bundle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UninstallMethod {
    AppxRemove,
    AppxProvisionedRemove,
    UninstallString {
        exe: PathBuf,
        args: Vec<String>,
        requires_admin: bool,
    },
    QuietUninstallString {
        exe: PathBuf,
        args: Vec<String>,
        requires_admin: bool,
    },
    MsiUninstall {
        product_code: String,
    },
    SteamUninstall {
        app_id: u64,
    },
    EpicUninstall {
        catalog_item_id: String,
    },
    GogUninstall {
        exe: PathBuf,
    },
    NoUninstaller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogMatch {
    pub catalog_id: String,
    pub requires_disclaimer: bool,
    pub risk: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResidualHints {
    pub appdata_roaming: Vec<PathBuf>,
    pub appdata_local: Vec<PathBuf>,
    pub programdata: Vec<PathBuf>,
    pub registry_keys: Vec<(String, String)>,
    pub start_menu_shortcuts: Vec<PathBuf>,
    pub desktop_shortcuts: Vec<PathBuf>,
    pub scheduled_tasks: Vec<String>,
    pub services: Vec<String>,
    pub firewall_rules: Vec<String>,
}

// ── Reports ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallReport {
    pub app_id: String,
    pub display_name: String,
    pub dry_run: bool,
    pub success: bool,
    pub method_used: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResidualsReport {
    pub app_id: String,
    pub dry_run: bool,
    pub paths_deleted: Vec<PathBuf>,
    pub registry_keys_deleted: Vec<(String, String)>,
    pub shortcuts_deleted: Vec<PathBuf>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallCompleteReport {
    pub app_id: String,
    pub display_name: String,
    pub dry_run: bool,
    pub restore_point_seq: Option<u32>,
    pub uninstall: UninstallReport,
    pub residuals: CleanResidualsReport,
    pub audit_run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedResiduals {
    pub appdata_roaming: Vec<PathBuf>,
    pub appdata_local: Vec<PathBuf>,
    pub programdata: Vec<PathBuf>,
    pub registry_keys: Vec<(String, String)>,
    pub shortcuts: Vec<PathBuf>,
}
