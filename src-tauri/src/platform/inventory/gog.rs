// platform/inventory/gog.rs — Listado de juegos GOG desde el registro.
//
// HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\*

use std::path::PathBuf;
use winreg::{RegKey, enums::*};

pub struct GogGame {
    pub game_id: u64,
    pub game_name: String,
    pub install_path: Option<PathBuf>,
    pub version: Option<String>,
    pub uninstall_exe: Option<PathBuf>,
}

pub fn list_gog_games() -> Vec<GogGame> {
    let mut games = Vec::new();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let paths = [
        "SOFTWARE\\WOW6432Node\\GOG.com\\Games",
        "SOFTWARE\\GOG.com\\Games",
    ];

    for path in paths {
        let Ok(base) = hklm.open_subkey(path) else {
            continue;
        };
        for sub_name in base.enum_keys().flatten() {
            let Ok(sub) = base.open_subkey(&sub_name) else {
                continue;
            };
            let game_id: u64 = sub_name.parse().unwrap_or(0);
            if game_id == 0 {
                continue;
            }

            let game_name: String = sub.get_value("gameName").or_else(|_| sub.get_value("GAMENAME")).unwrap_or_else(|_| format!("GOG Game {}", game_id));
            let install_path: Option<PathBuf> = sub
                .get_value::<String, _>("path")
                .or_else(|_| sub.get_value::<String, _>("PATH"))
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from);
            let version: Option<String> = sub.get_value("ver").or_else(|_| sub.get_value("VER")).ok();
            let uninstall_exe: Option<PathBuf> = sub
                .get_value::<String, _>("uninstallCommand")
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from);

            games.push(GogGame {
                game_id,
                game_name,
                install_path,
                version,
                uninstall_exe,
            });
        }
    }

    games
}
