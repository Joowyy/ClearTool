// platform/inventory/steam.rs — Parser de librerías Steam y manifiestos ACF.
//
// VDF v2 format: "key" "value" pairs, or "key" { ... } blocks.
// Busca: %PROGRAMFILES(X86)%\Steam\steamapps\libraryfolders.vdf

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SteamGame {
    pub app_id: u64,
    pub name: String,
    pub install_dir: Option<PathBuf>,
    pub size_bytes: Option<u64>,
    pub last_updated: Option<u64>,
    pub library_path: PathBuf,
}

pub fn list_steam_games() -> Vec<SteamGame> {
    let libraries = discover_steam_libraries();
    let mut games = Vec::new();
    for lib in libraries {
        let steamapps = lib.join("steamapps");
        if !steamapps.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("appmanifest_") && name_str.ends_with(".acf") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(game) = parse_acf(&content, &lib) {
                        games.push(game);
                    }
                }
            }
        }
    }
    games
}

fn discover_steam_libraries() -> Vec<PathBuf> {
    let mut libs: Vec<PathBuf> = Vec::new();

    // Default Steam install paths
    let candidates: Vec<PathBuf> = {
        let mut c = Vec::new();
        if let Ok(pf) = std::env::var("ProgramFiles(x86)") {
            c.push(PathBuf::from(pf).join("Steam"));
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            c.push(PathBuf::from(pf).join("Steam"));
        }
        c.push(PathBuf::from("C:\\Program Files (x86)\\Steam"));
        c.push(PathBuf::from("C:\\Program Files\\Steam"));
        c
    };

    for steam_dir in candidates {
        let lf = steam_dir.join("steamapps").join("libraryfolders.vdf");
        if lf.exists() {
            libs.push(steam_dir.clone());
            if let Ok(content) = std::fs::read_to_string(&lf) {
                let parsed = parse_vdf_flat(&content);
                // libraryfolders.vdf has numbered keys "0", "1", ... mapping to library paths
                // or in newer format, nested blocks with "path" key
                for (k, v) in &parsed {
                    if k == "path" || k.parse::<u32>().is_ok() {
                        let p = PathBuf::from(v);
                        if p.is_dir() && !libs.contains(&p) {
                            libs.push(p);
                        }
                    }
                }
            }
            break;
        }
    }

    libs
}

fn parse_acf(content: &str, library_path: &Path) -> Option<SteamGame> {
    let kvs = parse_vdf_flat(content);
    let app_id: u64 = kvs.get("appid")?.parse().ok()?;
    let name = kvs.get("name").cloned().unwrap_or_else(|| format!("AppID {}", app_id));
    let install_dir = kvs.get("installdir").and_then(|d| {
        let p = library_path.join("steamapps").join("common").join(d);
        if p.is_dir() { Some(p) } else { None }
    });
    let size_bytes: Option<u64> = kvs.get("SizeOnDisk").or_else(|| kvs.get("sizeondisk"))
        .and_then(|s| s.parse().ok());
    let last_updated: Option<u64> = kvs.get("LastUpdated").or_else(|| kvs.get("lastupdated"))
        .and_then(|s| s.parse().ok());

    Some(SteamGame {
        app_id,
        name,
        install_dir,
        size_bytes,
        last_updated,
        library_path: library_path.to_path_buf(),
    })
}

/// Parsea un VDF plano (solo pares "key" "value" a un nivel de profundidad).
/// Ignora bloques anidados para simplificar.
fn parse_vdf_flat(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut chars = content.chars().peekable();
    let mut tokens: Vec<String> = Vec::new();

    while chars.peek().is_some() {
        skip_whitespace_and_comments(&mut chars);
        match chars.peek() {
            Some('"') => {
                tokens.push(read_quoted(&mut chars));
            }
            Some('{') | Some('}') => {
                chars.next();
            }
            None => break,
            _ => {
                chars.next();
            }
        }
    }

    // Pares consecutivos de tokens = key/value
    let mut i = 0;
    while i + 1 < tokens.len() {
        map.insert(tokens[i].clone(), tokens[i + 1].clone());
        i += 2;
    }

    map
}

fn skip_whitespace_and_comments(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '/' {
            // consume rest of line (VDF comment)
            while let Some(&nc) = chars.peek() {
                chars.next();
                if nc == '\n' {
                    break;
                }
            }
        } else {
            break;
        }
    }
}

fn read_quoted(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    chars.next(); // consume opening "
    let mut s = String::new();
    let mut escaped = false;
    for c in chars.by_ref() {
        if escaped {
            s.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            break;
        } else {
            s.push(c);
        }
    }
    s
}
