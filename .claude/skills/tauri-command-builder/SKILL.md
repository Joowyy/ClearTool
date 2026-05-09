---
name: tauri-command-builder
description: Plantilla para crear nuevos comandos Tauri en ClearTool con validación de inputs, manejo tipado de errores, soporte de dry-run, emisión de eventos de progreso y generación automática de bindings TS via ts-rs. Usar SIEMPRE este skill al añadir un comando, en lugar de improvisar.
---

# Skill: tauri-command-builder

Plantilla canónica para nuevos comandos Tauri.

## Reglas

1. **Cada comando vive en su archivo dentro de `src-tauri/src/commands/`.**
2. **Cada comando tiene su DTO de input y output en `src-tauri/src/models/`** con `ts_rs::TS` derive y `#[ts(export)]`.
3. **Cada comando devuelve `Result<TOut, AppError>`.**
4. **Si la operación es destructiva, recibe `dry_run: bool` como input.**
5. **Si la operación dura > 1s, emite eventos de progreso.**
6. **Cada comando declara su capability en `src-tauri/capabilities/`.**

## Plantilla

```rust
// src-tauri/src/commands/example.rs

use crate::error::AppError;
use crate::models::example::{ExampleInput, ExampleReport};
use crate::services::example_service;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[tauri::command]
pub async fn run_example(
    input: ExampleInput,
    dry_run: bool,
    app: tauri::AppHandle,
) -> Result<ExampleReport, AppError> {
    // 1. Validar inputs (regex, paths canónicos, whitelist).
    input.validate()?;

    // 2. Si dry-run: ejecutar simulación y devolver.
    if dry_run {
        return example_service::simulate(&input).await;
    }

    // 3. Pre-flight: restore point si destructivo.
    let restore_id = if input.is_destructive() {
        Some(crate::services::restore_point::create("ClearTool: example").await?)
    } else { None };

    // 4. Ejecutar con emisión de progreso.
    let report = example_service::execute(&input, &app).await?;

    // 5. Loguear evento al log JSONL.
    crate::services::audit_log::record(&input, &report, restore_id)?;

    Ok(report)
}
```

## Modelo input/output

```rust
// src-tauri/src/models/example.rs

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ExampleInput {
    pub target: String,
    #[serde(default)]
    pub force: bool,
}

impl ExampleInput {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        static RE: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[A-Za-z0-9._-]{1,128}$").unwrap());
        if !RE.is_match(&self.target) {
            return Err(crate::error::AppError::Permission(format!("invalid target: {}", self.target)));
        }
        Ok(())
    }

    pub fn is_destructive(&self) -> bool { true }
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ExampleReport {
    pub status: ExampleStatus,
    pub items_affected: u32,
    pub bytes_freed: u64,
    pub log_entry_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
#[serde(rename_all = "kebab-case")]
pub enum ExampleStatus {
    Success,
    PartialSuccess,
    AlreadyDone,
    Cancelled,
    Failed,
}
```

## Registrar en `lib.rs`

```rust
// src-tauri/src/lib.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::example::run_example,
            // ... otros
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecuta la aplicación tauri");
}
```

## Capability mínima

```jsonc
// src-tauri/capabilities/elevated.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "elevated",
  "description": "Operaciones que requieren admin",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-execute",
    {
      "identifier": "shell:allow-spawn",
      "allow": [{ "name": "powershell", "cmd": "powershell.exe", "args": true, "sidecar": false }]
    }
  ]
}
```

## Frontend wrapper

Una vez generados los bindings TS:

```ts
// src/lib/tauri.ts
import { invoke } from '@tauri-apps/api/core';
import type { ExampleInput, ExampleReport } from '@/bindings';

export async function runExample(input: ExampleInput, dryRun = false): Promise<ExampleReport> {
  return invoke<ExampleReport>('run_example', { input, dryRun });
}
```

## Eventos

```rust
app.emit("example:progress", ProgressUpdate { total, done, current })?;
app.emit("example:complete", report.clone())?;
```

```ts
import { listen } from '@tauri-apps/api/event';
listen<ProgressUpdate>('example:progress', (e) => setProgress(e.payload));
```

## Test obligatorio

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validates_input_regex() {
        let bad = ExampleInput { target: "../etc/passwd".into(), force: false };
        assert!(bad.validate().is_err());
    }

    #[tokio::test]
    async fn dry_run_does_not_create_restore_point() {
        // mock del restore service y verificar que no se llamó
    }
}
```
