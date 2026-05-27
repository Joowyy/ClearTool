// platform/inventory/epic.rs — Lectura de manifiestos de Epic Games Launcher.
//
// Cada .item en %PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests\ es JSON.

use std::path::PathBuf;

pub struct EpicGame {
    pub catalog_item_id: String,
    pub display_name: String,
    pub install_location: Option<PathBuf>,
    pub version: Option<String>,
    pub app_name: Option<String>,
    pub manifest_path: PathBuf,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EpicManifest {
    #[serde(rename = "CatalogItemId")]
    catalog_item_id: Option<String>,
    display_name: Option<String>,
    install_location: Option<String>,
    app_version_string: Option<String>,
    app_name: Option<String>,
}

pub fn list_epic_games() -> Vec<EpicGame> {
    let manifests_dir = get_manifests_dir();
    let Some(dir) = manifests_dir else {
        return Vec::new();
    };
    if !dir.is_dir() {
        return Vec::new();
    }

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut games = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<EpicManifest>(&content) else {
            continue;
        };
        let Some(catalog_item_id) = manifest.catalog_item_id else {
            continue;
        };
        let display_name = manifest.display_name.unwrap_or_else(|| catalog_item_id.clone());
        games.push(EpicGame {
            catalog_item_id,
            display_name,
            install_location: manifest.install_location.filter(|s| !s.is_empty()).map(PathBuf::from),
            version: manifest.app_version_string,
            app_name: manifest.app_name,
            manifest_path: path,
        });
    }

    games
}

fn get_manifests_dir() -> Option<PathBuf> {
    let programdata = std::env::var("ProgramData").ok()?;
    Some(PathBuf::from(programdata)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests"))
}
