# 02 — Backend Rust (Tauri)

## Estructura `src-tauri/`

```
src-tauri/
├── Cargo.toml
├── build.rs
├── tauri.conf.json
├── icons/
├── capabilities/
│   ├── default.json
│   └── elevated.json
└── src/
    ├── main.rs
    ├── lib.rs
    ├── error.rs
    ├── elevation.rs
    ├── commands/
    │   ├── mod.rs
    │   ├── system_info.rs
    │   ├── explorer.rs
    │   ├── cache.rs
    │   ├── debloat.rs
    │   ├── services.rs
    │   ├── registry.rs
    │   ├── restore.rs
    │   └── audit.rs
    ├── services/
    │   ├── mod.rs
    │   ├── filesystem.rs
    │   ├── powershell.rs
    │   ├── registry.rs
    │   ├── service_manager.rs
    │   ├── restore_point.rs
    │   ├── audit_log.rs
    │   └── catalog.rs            # carga bloatware-catalog.json + cache-locations.json
    └── models/
        ├── mod.rs
        ├── system.rs
        ├── tree.rs
        ├── cache.rs
        ├── debloat.rs
        ├── service.rs
        ├── registry.rs
        └── restore.rs
```

## Cargo.toml — base

```toml
[package]
name = "cleartool"
version = "0.1.0"
edition = "2021"
rust-version = "1.78"

[lib]
name = "cleartool_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-shell = "2"
tauri-plugin-log = "2"
tauri-plugin-os = "2"
tauri-plugin-process = "2"

serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
once_cell = "1"
regex = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
walkdir = "2"
dunce = "1"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_Storage_FileSystem",
  "Win32_System_Registry",
  "Win32_System_Services",
  "Win32_System_Wmi",
  "Win32_System_Com",
  "Win32_System_Threading",
  "Win32_Security",
] }
winreg = "0.52"
windows-service = "0.7"

[dependencies.ts-rs]
version = "9"
features = ["serde-compat", "chrono-impl", "uuid-impl"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

## Tauri.conf.json — extracto

```jsonc
{
  "productName": "ClearTool",
  "version": "0.1.0",
  "identifier": "dev.jowy.cleartool",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "ClearTool",
        "width": 1280,
        "height": 800,
        "minWidth": 1024,
        "minHeight": 700,
        "resizable": true,
        "decorations": true,
        "transparent": false,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: asset:; style-src 'self' 'unsafe-inline'; script-src 'self'"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"],
    "windows": {
      "wix": {},
      "nsis": {
        "installerIcon": "icons/icon.ico",
        "installMode": "currentUser"
      }
    }
  }
}
```

> El `requireAdministrator` se activa con un manifest XML embebido vía `build.rs` (`embed-resource` crate) o configuración `bundle.windows.allowDowngrades` + manifest custom.

## error.rs

```rust
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("registry: {0}")]
    Registry(String),
    #[error("powershell: {0}")]
    Powershell(String),
    #[error("permission: {0}")]
    Permission(String),
    #[error("not elevated")]
    NotElevated,
    #[error("cancelled")]
    Cancelled,
    #[error("restore unavailable: {0}")]
    RestoreUnavailable(String),
    #[error("external: {0}")]
    External(String),
    #[error("not implemented")]
    NotImplemented,
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = serde_json::Map::new();
        let kind = match self {
            AppError::Io(_) => "io",
            AppError::Registry(_) => "registry",
            AppError::Powershell(_) => "powershell",
            AppError::Permission(_) => "permission",
            AppError::NotElevated => "not-elevated",
            AppError::Cancelled => "cancelled",
            AppError::RestoreUnavailable(_) => "restore-unavailable",
            AppError::External(_) => "external",
            AppError::NotImplemented => "not-implemented",
        };
        map.insert("kind".into(), kind.into());
        map.insert("message".into(), self.to_string().into());
        serde_json::Value::Object(map).serialize(s)
    }
}
```

## elevation.rs

```rust
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::*;
use windows::Win32::System::Threading::GetCurrentProcess;

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        ).is_ok();
        let _ = windows::Win32::Foundation::CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}
```

## Comandos exportados (lista cerrada inicial)

```rust
// lib.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::system_info::is_elevated,
            commands::system_info::system_summary,

            commands::explorer::scan_tree,
            commands::explorer::cancel_scan,
            commands::explorer::compute_directory_size,

            commands::cache::list_cache_locations,
            commands::cache::scan_cache_locations,
            commands::cache::clean_cache_locations,

            commands::debloat::list_bloatware_catalog,
            commands::debloat::detect_installed_bloatware,
            commands::debloat::remove_bloatware,

            commands::services::list_services,
            commands::services::set_service_state,
            commands::services::apply_service_preset,

            commands::registry::list_registry_tweaks,
            commands::registry::read_registry_tweak_state,
            commands::registry::apply_registry_tweak,
            commands::registry::apply_registry_tweak_batch,
            commands::registry::revert_registry_tweak,

            commands::restore::ensure_restore_enabled,
            commands::restore::create_restore_point,
            commands::restore::list_restore_points,
            commands::restore::restore_to_point,

            commands::audit::list_audit_log,
            commands::audit::revert_audit_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecuta la aplicación");
}
```

## Convenciones

- Cada `command` -> función `pub async fn` en su archivo.
- Cada operación destructiva acepta `dry_run: bool` como parámetro nominal.
- Eventos: namespace `<modulo>:<evento>` (`cache:progress`, `debloat:complete`).
- Logging: `tracing` redirigido por el plugin `tauri-plugin-log`.

## Build scripts

```toml
# tauri.conf.json -> hooks
"beforeBuildCommand": "npm run build && cargo run --bin generate_bindings"
```

`generate_bindings.rs`: itera structs con `#[ts(export)]` y emite `src/bindings/index.ts` agregando todos los DTOs.
