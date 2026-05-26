# Paso 01 — Modelo `StartupEntry` + 5 orígenes

**Área**: 05-startup-manager
**Tiempo estimado**: 1.5 horas
**Dependencias**: ninguna

## Qué hacemos

Definir los DTOs Rust + TS que representan una entry de auto-arranque, con discriminator por tipo de origen.

## Archivos

- `src-tauri/src/models/startup.rs` (nuevo)
- `src-tauri/src/models/mod.rs` (export)
- `src/api/types.ts` (tipos)

## Cómo

```rust
// src-tauri/src/models/startup.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    pub id: String,                   // unique: kind + path/name
    pub origin: StartupOrigin,
    pub display_name: String,
    pub command: String,
    pub exe_path: Option<String>,
    pub icon_path: Option<String>,
    pub publisher: Option<String>,
    pub signature_valid: Option<bool>,
    pub impact: StartupImpact,
    pub last_modified: Option<String>,
    pub enabled: bool,
    pub category: StartupCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StartupOrigin {
    Registry { hive: String, key: String, name: String },
    StartupFolder { lnk_path: String },
    ScheduledTask { task_path: String },
    Service { service_name: String },
    UwpAutoStart { package_family_name: String, task_id: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StartupImpact { Unknown, Low, Medium, High }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StartupCategory {
    Updater, Launcher, Widget, CloudSync, Communication,
    Media, Security, Driver, UserApp, System, Unknown,
}
```

## Criterio de done

- [ ] Structs Rust definidos.
- [ ] Discriminated union `StartupOrigin` con tag `kind`.
- [ ] TS types correspondientes.
- [ ] `cargo check` + `tsc` pasan.
