// domain/inventory.rs — Orquestador del inventario universal de apps.

use crate::core::{AppError, AppResult};
use crate::domain::catalog;
use crate::models::inventory::*;
use crate::platform::inventory as pi;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ── Heurísticas de criticidad ─────────────────────────────────────────

const CRITICAL_PUBLISHERS: &[&str] = &["Microsoft Corporation", "Microsoft Windows"];

const CRITICAL_NAME_PREFIXES: &[&str] = &[
    "Microsoft Visual C++",
    "Windows SDK",
    ".NET",
    "Microsoft Edge WebView2",
    "DirectX",
    "Windows Installer",
    "Microsoft ASP.NET",
    "Microsoft Build Tools",
];

fn is_system_critical(display_name: &str, publisher: Option<&str>, system_component: bool) -> bool {
    if system_component {
        return true;
    }
    let is_ms_publisher = publisher
        .map(|p| CRITICAL_PUBLISHERS.iter().any(|cp| p.eq_ignore_ascii_case(cp)))
        .unwrap_or(false);
    if !is_ms_publisher {
        return false;
    }
    CRITICAL_NAME_PREFIXES
        .iter()
        .any(|pfx| display_name.starts_with(pfx))
}

// ── AppData / registry denylist para residuales ──────────────────────

const APPDATA_DENYLIST: &[&str] = &[
    "microsoft",
    "windows",
    "packages",
    "roaming",
    "local",
    "temp",
    "mozilla",
];

fn is_denied_appdata(path: &Path) -> bool {
    let Some(last) = path.file_name() else {
        return true;
    };
    let lower = last.to_string_lossy().to_lowercase();
    APPDATA_DENYLIST.iter().any(|d| lower == *d)
}

// ── Cálculo de residuales (heurístico) ──────────────────────────────

pub fn compute_residual_hints(app: &InstalledApp) -> ResidualHints {
    let name_candidates = name_candidates(app);
    let mut hints = ResidualHints::default();

    // AppData roaming
    if let Ok(roaming) = std::env::var("APPDATA") {
        hints.appdata_roaming = find_candidate_dirs(Path::new(&roaming), &name_candidates);
    }

    // AppData local
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        hints.appdata_local = find_candidate_dirs(Path::new(&local), &name_candidates);
    }

    // ProgramData
    if let Ok(pd) = std::env::var("ProgramData") {
        hints.programdata = find_candidate_dirs(Path::new(&pd), &name_candidates);
    }

    // Registry keys
    hints.registry_keys = find_registry_keys(&name_candidates, app.publisher.as_deref());

    // Start menu shortcuts
    if let Ok(appdata) = std::env::var("APPDATA") {
        let sm = Path::new(&appdata)
            .join("Microsoft\\Windows\\Start Menu\\Programs");
        hints.start_menu_shortcuts.extend(find_shortcuts(&sm, &name_candidates));
    }
    if let Ok(pd) = std::env::var("ProgramData") {
        let sm = Path::new(&pd)
            .join("Microsoft\\Windows\\Start Menu\\Programs");
        hints.start_menu_shortcuts.extend(find_shortcuts(&sm, &name_candidates));
    }

    // Desktop shortcuts
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let desktop = Path::new(&userprofile).join("Desktop");
        hints.desktop_shortcuts = find_shortcuts(&desktop, &name_candidates);
    }

    hints
}

fn name_candidates(app: &InstalledApp) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    v.push(app.display_name.clone());
    if let Some(pub_name) = &app.publisher {
        v.push(pub_name.clone());
        // "Publisher Name Inc" → ["Publisher Name Inc", "Publisher Name", "Publisher"]
        if pub_name.contains(' ') {
            let first_word = pub_name.split_whitespace().next().unwrap_or("");
            if first_word.len() > 3 {
                v.push(first_word.to_string());
            }
        }
    }
    v
}

fn find_candidate_dirs(base: &Path, candidates: &[String]) -> Vec<PathBuf> {
    if !base.is_dir() {
        return Vec::new();
    }
    let mut found = Vec::new();
    for candidate in candidates {
        let p = base.join(candidate);
        if p.is_dir() && !is_denied_appdata(&p) {
            found.push(p);
        }
    }
    found
}

fn find_shortcuts(dir: &Path, candidates: &[String]) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_lowercase();
        for c in candidates {
            if fname.contains(&c.to_lowercase()) {
                found.push(path.clone());
                break;
            }
        }
    }
    found
}

fn find_registry_keys(candidates: &[String], publisher: Option<&str>) -> Vec<(String, String)> {
    use winreg::{RegKey, enums::*};
    let mut keys = Vec::new();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let reg_roots: &[(&str, &RegKey, &str)] = &[
        ("HKCU", &hkcu, "Software"),
        ("HKLM", &hklm, "SOFTWARE"),
        ("HKLM", &hklm, "SOFTWARE\\WOW6432Node"),
    ];

    for (hive_name, root, base) in reg_roots {
        let Ok(base_key) = root.open_subkey(base) else {
            continue;
        };
        // Check Publisher\Name and Name directly
        for candidate in candidates {
            let path = candidate.as_str();
            if base_key.open_subkey(path).is_ok() {
                keys.push((hive_name.to_string(), format!("{}\\{}", base, path)));
            }
            if let Some(pub_name) = publisher {
                let nested = format!("{}\\{}", pub_name, candidate);
                if base_key.open_subkey(&nested).is_ok() {
                    keys.push((hive_name.to_string(), format!("{}\\{}", base, nested)));
                }
            }
        }
    }

    // Dedup
    keys.sort();
    keys.dedup();
    keys
}

// ── Construcción del inventario ──────────────────────────────────────

pub fn build_inventory() -> AppResult<Vec<InstalledApp>> {
    let catalog = catalog::load_bloatware_catalog().unwrap_or_default();
    let mut apps: Vec<InstalledApp> = Vec::new();

    // 1. Win32 Uninstall registry
    let win32 = pi::list_win32_apps();
    for app in win32 {
        let critical = is_system_critical(&app.display_name, app.publisher.as_deref(), app.is_system_component);
        let catalog_match = find_catalog_match_win32(&app.display_name, app.publisher.as_deref(), &catalog);
        let uninstall_method = build_win32_uninstall(&app);
        let id = format!("win32:{}:{}", app.hive.replace(' ', "-"), sanitize_id(&app.registry_key));
        apps.push(InstalledApp {
            id,
            display_name: app.display_name,
            publisher: app.publisher,
            version: app.version,
            source: AppSource::Win32Uninstaller {
                registry_key: app.registry_key,
                hive: app.hive,
            },
            install_location: app.install_location,
            install_date: app.install_date,
            size_bytes: app.size_bytes,
            uninstall_method,
            is_system_critical: critical,
            catalog_match,
            residual_hints: ResidualHints::default(),
        });
    }

    // 2. Appx packages
    if let Ok(packages) = pi::list_appx_packages() {
        for pkg in packages {
            let catalog_match = find_catalog_match_appx(&pkg.family_name, &catalog);
            let is_framework = pkg.appx_kind == AppxKind::Framework;
            let uninstall_method = if is_framework {
                UninstallMethod::NoUninstaller
            } else {
                UninstallMethod::AppxRemove
            };
            let id = format!("appx:{}", sanitize_id(&pkg.full_name));
            apps.push(InstalledApp {
                id,
                display_name: pkg.display_name,
                publisher: pkg.publisher,
                version: pkg.version,
                source: AppSource::AppxPackage {
                    full_name: pkg.full_name,
                    family_name: pkg.family_name,
                    appx_kind: pkg.appx_kind.clone(),
                },
                install_location: pkg.install_location,
                install_date: None,
                size_bytes: None,
                uninstall_method,
                is_system_critical: is_framework,
                catalog_match,
                residual_hints: ResidualHints::default(),
            });
        }
    }

    // 3. Appx provisioned
    if let Ok(provs) = pi::list_appx_provisioned() {
        for prov in provs {
            // Skip if already listed as user package
            let already_listed = apps.iter().any(|a| {
                if let AppSource::AppxPackage { full_name, .. } = &a.source {
                    full_name.contains(&prov.full_name)
                } else {
                    false
                }
            });
            if already_listed {
                continue;
            }
            let catalog_match = catalog.iter().find(|e| {
                e.appx_provisioned_name.as_deref() == Some(&prov.full_name)
            }).map(|e| CatalogMatch {
                catalog_id: e.id.clone(),
                requires_disclaimer: e.consequences.is_empty().not(),
                risk: e.risk.clone(),
                category: e.category.clone(),
            });
            let id = format!("appx-prov:{}", sanitize_id(&prov.full_name));
            apps.push(InstalledApp {
                id,
                display_name: prov.display_name,
                publisher: None,
                version: None,
                source: AppSource::AppxProvisioned { full_name: prov.full_name },
                install_location: None,
                install_date: None,
                size_bytes: None,
                uninstall_method: UninstallMethod::AppxProvisionedRemove,
                is_system_critical: false,
                catalog_match,
                residual_hints: ResidualHints::default(),
            });
        }
    }

    // 4. Steam games
    let steam_games = pi::list_steam_games();
    for game in steam_games {
        let id = format!("steam:{}", game.app_id);
        apps.push(InstalledApp {
            id,
            display_name: game.name,
            publisher: Some("Steam".to_string()),
            version: None,
            source: AppSource::Steam {
                app_id: game.app_id,
                library_path: game.library_path.clone(),
            },
            install_location: game.install_dir,
            install_date: None,
            size_bytes: game.size_bytes,
            uninstall_method: UninstallMethod::SteamUninstall { app_id: game.app_id },
            is_system_critical: false,
            catalog_match: None,
            residual_hints: ResidualHints::default(),
        });
    }

    // 5. Epic Games
    let epic_games = pi::list_epic_games();
    for game in epic_games {
        let id = format!("epic:{}", sanitize_id(&game.catalog_item_id));
        let catalog_item_id = game.catalog_item_id.clone();
        apps.push(InstalledApp {
            id,
            display_name: game.display_name,
            publisher: Some("Epic Games".to_string()),
            version: game.version,
            source: AppSource::EpicGames {
                catalog_item_id: catalog_item_id.clone(),
                manifest_path: game.manifest_path,
            },
            install_location: game.install_location,
            install_date: None,
            size_bytes: None,
            uninstall_method: UninstallMethod::EpicUninstall { catalog_item_id },
            is_system_critical: false,
            catalog_match: None,
            residual_hints: ResidualHints::default(),
        });
    }

    // 6. GOG games
    let gog_games = pi::list_gog_games();
    for game in gog_games {
        let id = format!("gog:{}", game.game_id);
        let uninstall_method = match game.uninstall_exe.clone() {
            Some(exe) => UninstallMethod::GogUninstall { exe },
            None => UninstallMethod::NoUninstaller,
        };
        apps.push(InstalledApp {
            id,
            display_name: game.game_name,
            publisher: Some("GOG.com".to_string()),
            version: game.version,
            source: AppSource::Gog { game_id: game.game_id },
            install_location: game.install_path,
            install_date: None,
            size_bytes: None,
            uninstall_method,
            is_system_critical: false,
            catalog_match: None,
            residual_hints: ResidualHints::default(),
        });
    }

    Ok(apps)
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}

fn build_win32_uninstall(app: &pi::uninstall_registry::Win32App) -> UninstallMethod {
    if app.is_msi {
        if let Some(ref guid) = app.product_code {
            return UninstallMethod::MsiUninstall { product_code: guid.clone() };
        }
    }
    if let Some(ref quiet) = app.quiet_uninstall_string {
        if let Some((exe, args)) = parse_command_string(quiet) {
            return UninstallMethod::QuietUninstallString {
                exe,
                args,
                requires_admin: app.requires_admin,
            };
        }
    }
    if let Some(ref uninstall) = app.uninstall_string {
        if let Some((exe, args)) = parse_command_string(uninstall) {
            return UninstallMethod::UninstallString {
                exe,
                args,
                requires_admin: app.requires_admin,
            };
        }
    }
    UninstallMethod::NoUninstaller
}

fn parse_command_string(s: &str) -> Option<(PathBuf, Vec<String>)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try quoted path first: "C:\path\app.exe" args...
    if s.starts_with('"') {
        if let Some(end) = s[1..].find('"') {
            let exe = PathBuf::from(&s[1..=end]);
            let rest = s[end + 2..].trim();
            let args = if rest.is_empty() {
                Vec::new()
            } else {
                rest.split_whitespace().map(str::to_string).collect()
            };
            return Some((exe, args));
        }
    }

    // Unquoted: first token is exe
    let mut parts = s.splitn(2, ' ');
    let exe = PathBuf::from(parts.next()?);
    let args: Vec<String> = parts
        .next()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    Some((exe, args))
}

fn find_catalog_match_win32(
    display_name: &str,
    _publisher: Option<&str>,
    catalog: &[crate::models::debloat::BloatwareEntry],
) -> Option<CatalogMatch> {
    let name_lower = display_name.to_lowercase();
    catalog.iter().find(|e| {
        e.display_name.to_lowercase() == name_lower
            || e.winget_id.as_deref().map_or(false, |id| {
                display_name.to_lowercase().contains(&id.to_lowercase())
            })
    }).map(|e| CatalogMatch {
        catalog_id: e.id.clone(),
        requires_disclaimer: !e.consequences.is_empty(),
        risk: e.risk.clone(),
        category: e.category.clone(),
    })
}

fn find_catalog_match_appx(
    family_name: &str,
    catalog: &[crate::models::debloat::BloatwareEntry],
) -> Option<CatalogMatch> {
    catalog.iter().find(|e| {
        e.appx_package_family_name.as_deref() == Some(family_name)
    }).map(|e| CatalogMatch {
        catalog_id: e.id.clone(),
        requires_disclaimer: !e.consequences.is_empty(),
        risk: e.risk.clone(),
        category: e.category.clone(),
    })
}

// ── Uninstall ────────────────────────────────────────────────────────

pub fn uninstall_app(app: &InstalledApp, dry_run: bool) -> AppResult<UninstallReport> {
    let start = Instant::now();
    let method_name = method_label(&app.uninstall_method);

    if dry_run {
        return Ok(UninstallReport {
            app_id: app.id.clone(),
            display_name: app.display_name.clone(),
            dry_run: true,
            success: true,
            method_used: method_name,
            error: None,
            duration_ms: 0,
        });
    }

    let result = execute_uninstall(app);
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(()) => Ok(UninstallReport {
            app_id: app.id.clone(),
            display_name: app.display_name.clone(),
            dry_run: false,
            success: true,
            method_used: method_name,
            error: None,
            duration_ms,
        }),
        Err(e) => Ok(UninstallReport {
            app_id: app.id.clone(),
            display_name: app.display_name.clone(),
            dry_run: false,
            success: false,
            method_used: method_name,
            error: Some(e.to_string()),
            duration_ms,
        }),
    }
}

fn execute_uninstall(app: &InstalledApp) -> AppResult<()> {
    match &app.uninstall_method {
        UninstallMethod::AppxRemove => {
            if let AppSource::AppxPackage { family_name, .. } = &app.source {
                crate::platform::debloat::remove_appx_user(family_name)?;
            }
        }
        UninstallMethod::AppxProvisionedRemove => {
            if let AppSource::AppxProvisioned { full_name } = &app.source {
                crate::platform::debloat::remove_appx_provisioned(full_name)?;
            }
        }
        UninstallMethod::MsiUninstall { product_code } => {
            let status = std::process::Command::new("msiexec")
                .args(["/x", product_code, "/qn", "/norestart"])
                .status()
                .map_err(|e| AppError::External(e.to_string()))?;
            if !status.success() {
                return Err(AppError::External(format!("msiexec exited with {:?}", status.code())));
            }
        }
        UninstallMethod::UninstallString { exe, args, .. }
        | UninstallMethod::QuietUninstallString { exe, args, .. } => {
            let status = std::process::Command::new(exe)
                .args(args)
                .status()
                .map_err(|e| AppError::External(e.to_string()))?;
            if !status.success() {
                return Err(AppError::External(format!("uninstaller exited with {:?}", status.code())));
            }
        }
        UninstallMethod::SteamUninstall { app_id } => {
            open_url(&format!("steam://uninstall/{}", app_id))?;
        }
        UninstallMethod::EpicUninstall { catalog_item_id } => {
            open_url(&format!(
                "com.epicgames.launcher://apps/{}?action=uninstall",
                catalog_item_id
            ))?;
        }
        UninstallMethod::GogUninstall { exe } => {
            let status = std::process::Command::new(exe)
                .arg("/SILENT")
                .status()
                .map_err(|e| AppError::External(e.to_string()))?;
            if !status.success() {
                return Err(AppError::External(format!("GOG uninstaller exited with {:?}", status.code())));
            }
        }
        UninstallMethod::NoUninstaller => {
            return Err(AppError::External(
                "No uninstaller available for this app".to_string(),
            ));
        }
    }
    Ok(())
}

fn open_url(url: &str) -> AppResult<()> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map_err(|e| AppError::External(e.to_string()))?;
    Ok(())
}

fn method_label(m: &UninstallMethod) -> String {
    match m {
        UninstallMethod::AppxRemove => "AppxRemove".to_string(),
        UninstallMethod::AppxProvisionedRemove => "AppxProvisionedRemove".to_string(),
        UninstallMethod::MsiUninstall { .. } => "MSI".to_string(),
        UninstallMethod::UninstallString { .. } => "UninstallString".to_string(),
        UninstallMethod::QuietUninstallString { .. } => "QuietUninstall".to_string(),
        UninstallMethod::SteamUninstall { .. } => "Steam".to_string(),
        UninstallMethod::EpicUninstall { .. } => "EpicGames".to_string(),
        UninstallMethod::GogUninstall { .. } => "GOG".to_string(),
        UninstallMethod::NoUninstaller => "NoUninstaller".to_string(),
    }
}

// ── Clean residuals ──────────────────────────────────────────────────

pub fn clean_residuals(
    app_id: &str,
    selected: &SelectedResiduals,
    dry_run: bool,
) -> AppResult<CleanResidualsReport> {
    let mut paths_deleted: Vec<PathBuf> = Vec::new();
    let mut registry_keys_deleted: Vec<(String, String)> = Vec::new();
    let mut shortcuts_deleted: Vec<PathBuf> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    let all_paths: Vec<&PathBuf> = selected
        .appdata_roaming
        .iter()
        .chain(&selected.appdata_local)
        .chain(&selected.programdata)
        .collect();

    for path in all_paths {
        if !path.exists() {
            continue;
        }
        if dry_run {
            paths_deleted.push(path.clone());
            continue;
        }
        match std::fs::remove_dir_all(path) {
            Ok(()) => paths_deleted.push(path.clone()),
            Err(e) => errors.push(format!("delete {}: {}", path.display(), e)),
        }
    }

    if !dry_run {
        for (hive, key_path) in &selected.registry_keys {
            match delete_registry_key(hive, key_path) {
                Ok(()) => registry_keys_deleted.push((hive.clone(), key_path.clone())),
                Err(e) => errors.push(format!("registry {}\\{}: {}", hive, key_path, e)),
            }
        }
    } else {
        registry_keys_deleted.extend(selected.registry_keys.iter().cloned());
    }

    for shortcut in &selected.shortcuts {
        if !shortcut.exists() {
            continue;
        }
        if dry_run {
            shortcuts_deleted.push(shortcut.clone());
            continue;
        }
        match std::fs::remove_file(shortcut) {
            Ok(()) => shortcuts_deleted.push(shortcut.clone()),
            Err(e) => errors.push(format!("shortcut {}: {}", shortcut.display(), e)),
        }
    }

    Ok(CleanResidualsReport {
        app_id: app_id.to_string(),
        dry_run,
        paths_deleted,
        registry_keys_deleted,
        shortcuts_deleted,
        errors,
    })
}

fn delete_registry_key(hive: &str, key_path: &str) -> AppResult<()> {
    use winreg::{RegKey, enums::*};
    let root = match hive {
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return Err(AppError::Registry(format!("unknown hive: {}", hive))),
    };
    root.delete_subkey_all(key_path)
        .map_err(|e| AppError::Registry(e.to_string()))
}

// Workaround: BloatwareEntry.consequences puede estar vacío → no requires_disclaimer
trait Not {
    fn not(self) -> bool;
}
impl Not for bool {
    fn not(self) -> bool {
        !self
    }
}
