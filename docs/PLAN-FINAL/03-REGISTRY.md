# 03 — Tweaks de Registro

> **Posición:** 3/14. Primer módulo destructivo real.
> **Dependencias:** [00-CATALOGOS](00-CATALOGOS.md), [01-RESTORE-POINTS](01-RESTORE-POINTS.md), [02-AUDIT-LOG](02-AUDIT-LOG.md).
> **Output:** `domain::registry` y `platform::registry` completos. UI con diff viewer + dry-run + apply batch.

---

## 1. Resumen ejecutivo

Estado actual:

| Componente | Estado |
|---|---|
| `domain::registry::*` | ❌ Todos `NotImplemented` |
| `platform::registry::*` | ❌ Stub |
| Catálogo `registry-tweaks.json` | ✅ Curado en [00-CATALOGOS](00-CATALOGOS.md) |
| UI `registry-page.tsx` | 🟡 Esqueleto contra stub |
| Backup `.reg` previo | ❌ No existe |
| Diff viewer | ❌ No existe |

Cierre del archivo:
- 5 funciones del domain implementadas.
- `platform::registry` con `winreg` típico + write seguro.
- UI con: lista categorizada, dry-run preview, apply batch con barra de progreso, diff viewer, integración con audit log.

---

## 2. Diagnóstico

### 2.1 Por qué `winreg` y no `windows-rs`

`winreg` ya está en `Cargo.toml`. Es la crate de facto para registro Windows. `windows-rs` puede hacerlo pero la API es más verbosa.

### 2.2 Backup `.reg`

Antes de cualquier write, generar un `.reg` con los valores actuales de las keys afectadas. Esto es **redundante con el audit log + restore point**, pero da una tercera capa: si el audit JSONL se corrompe, el usuario puede aún ejecutar el `.reg` manualmente desde `regedit`.

**Decisión:** mantener el `.reg` como nice-to-have, no bloquea v1.0. Si hay tiempo, implementar; si no, audit log + restore point son suficientes.

### 2.3 Allowlist de hives

Solo `HKLM`, `HKCU`, `HKU`, `HKCR` permitidos. Cualquier otro hive → abort.

```rust
const ALLOWED_HIVES: &[&str] = &["HKLM", "HKCU", "HKU", "HKCR"];
```

### 2.4 Path traversal

Las `key` del catálogo son strings literales del JSON, no input de usuario. Pero el `key` también se acepta en `ipc::registry::apply_registry_tweak` (vía `input.id`). Validación: `input.id` debe coincidir con un `id` del catálogo. **Nunca** se acepta `(hive, key, name)` directos del frontend.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| `winreg` para lectura/escritura | Existente, simple |
| Lectura de "estado actual" via `previous_value: Option<Value>` | None = la entrada no existe; Some = está |
| Apply usa `enabledValue` o `disabledValue` del catálogo | No improvisa valores |
| Dry-run reporta diff sin tocar | Confianza del usuario |
| Batch: ejecuta ops en orden, aborta al primer fallo (no rollback transaccional) | Las operaciones son independientes; el audit log permite revert |
| Cada batch emite `registry:progress` cada 5 operaciones | UI con barra de progreso |
| Validación hive contra `ALLOWED_HIVES` en cada op | Defense in depth |

---

## 4. Modelo de datos

Ya definido en [00-CATALOGOS §8.3.2](00-CATALOGOS.md#paso-832--modelsregistryrs-ampliar). Recap:

```rust
pub struct RegistryTweak { id, display_name, category, risk, requires_admin, consequences, operations: Vec<RegistryOp>, presets, ... }
pub struct RegistryOp { hive, key, name, kind, enabled_value, disabled_value, create_if_missing }
pub struct TweakState { id, enabled, partial, current_values }
pub struct ApplyTweakInput { id, enable, dry_run }
```

Tipos nuevos a añadir:

```rust
// models/registry.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakBatchInput {
    pub inputs: Vec<ApplyTweakInput>,
    pub create_restore_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakBatchReport {
    pub run_id: String,
    pub total: u32,
    pub applied: u32,
    pub failed: u32,
    pub skipped: u32,
    pub restore_point_seq: Option<u32>,
    pub per_tweak: Vec<PerTweakResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerTweakResult {
    pub id: String,
    pub status: String,    // "applied" | "skipped" | "failed" | "dry-run"
    pub error: Option<String>,
    pub previous_values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryProgressEvent {
    pub run_id: String,
    pub processed: u32,
    pub total: u32,
    pub current_id: String,
}
```

---

## 5. Plan UX

### 5.1 Pantalla principal

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Tweaks de Registro                          [Preview]  [Aplicar (12)]   │
│ 45 tweaks · 12 seleccionados · ⚠ requiere admin                          │
├─────────────────────────────────────────────────────────────────────────┤
│ Filtros: [Categoría ▾] [Riesgo ▾] [Preset ▾]                            │
│                                                                          │
│  Telemetría                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ ☑ Reducir telemetría a 'Required Only'        low   ◐ aplicado   │   │
│  │   Establece AllowTelemetry=1 (mínimo legal en Pro/Enterprise).   │   │
│  │   1 operación · HKLM                            [Ver detalles]   │   │
│  ├──────────────────────────────────────────────────────────────────┤   │
│  │ ☑ Desactivar CEIP                              low   ○ no apl    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  Ads                                                                     │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ ☑ Quitar sugerencias en Start                  low   ○ no apl    │   │
│  │ ☐ Quitar Spotlight del lockscreen              low   ◐ aplicado  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│  ...                                                                     │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Modal "Ver detalles" (diff viewer)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Reducir telemetría a 'Required Only'                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  Categoría:    Telemetry                                                 │
│  Riesgo:       Bajo                                                      │
│  Requiere:     Admin (HKLM)                                              │
│                                                                          │
│  Consecuencias:                                                          │
│   • Sigue habiendo datos básicos enviados a Microsoft (compatibilidad   │
│     de drivers, etc.).                                                   │
│                                                                          │
│  Operaciones (1):                                                        │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection           │  │
│  │ AllowTelemetry (DWORD)                                            │  │
│  │                                                                    │  │
│  │ Estado actual:   3   (Full)                                       │  │
│  │ Aplicar:         1   (Basic only) ✓                                │  │
│  │ Revertir:        3   (Full) ✓                                      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│                                       [Cerrar]   [Aplicar este tweak]   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Modal de preview pre-batch

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Aplicar 12 tweaks                                                       │
├─────────────────────────────────────────────────────────────────────────┤
│  Vas a aplicar 12 tweaks afectando 23 valores de registro:               │
│                                                                          │
│  HKLM (requiere admin): 4 valores                                        │
│  HKCU                  : 19 valores                                      │
│                                                                          │
│  ☑ Crear punto de restauración antes (recomendado)                       │
│  ☐ Dry-run (solo simular, no aplicar)                                    │
│                                                                          │
│  Estimación: ~5 segundos.                                                │
│                                                                          │
│                                       [Cancelar]   [Aplicar →]           │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.4 Estado durante el batch

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Aplicando tweaks...                                              [×]    │
├─────────────────────────────────────────────────────────────────────────┤
│  ████████████████░░░░░░  8 / 12                                          │
│                                                                          │
│  ✓ ads-disable-tips                                                      │
│  ✓ taskbar-hide-widgets                                                  │
│  ✓ taskbar-hide-chat                                                     │
│  ✓ telemetry-allowtelemetry                                              │
│  ⏳ explorer-show-extensions                                              │
│  ...                                                                     │
│                                                                          │
│  Punto de restauración Seq #145 creado.                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Plan de implementación por fases

### Fase 1 — `platform::registry`

#### Paso 1.1 — Helpers básicos

**Archivo:** `src-tauri/src/platform/registry.rs`

```rust
use crate::core::{AppError, AppResult};
use winreg::enums::*;
use winreg::RegKey;
use winreg::types::FromRegValue;
use serde_json::Value;

const ALLOWED_HIVES: &[&str] = &["HKLM", "HKCU", "HKU", "HKCR"];

fn hkey_for(hive: &str) -> AppResult<RegKey> {
    if !ALLOWED_HIVES.contains(&hive) {
        return Err(AppError::Permission(format!("hive no permitido: {}", hive)));
    }
    Ok(match hive {
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        "HKU"  => RegKey::predef(HKEY_USERS),
        "HKCR" => RegKey::predef(HKEY_CLASSES_ROOT),
        _ => unreachable!(),
    })
}

/// Lee un valor. Devuelve None si la key o el value name no existen.
pub fn read_value(hive: &str, key: &str, name: &str) -> AppResult<Option<Value>> {
    let root = hkey_for(hive)?;
    let subkey = match root.open_subkey(key) {
        Ok(k) => k,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::Registry(format!("open {}\\{}: {}", hive, key, e))),
    };
    let v = match subkey.get_raw_value(name) {
        Ok(rv) => rv,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::Registry(format!("get {}: {}", name, e))),
    };
    Ok(Some(reg_value_to_json(&v)))
}

fn reg_value_to_json(v: &winreg::RegValue) -> Value {
    match v.vtype {
        REG_DWORD => {
            let n = u32::from_le_bytes(v.bytes[..4].try_into().unwrap_or([0;4]));
            Value::from(n)
        }
        REG_QWORD => {
            let n = u64::from_le_bytes(v.bytes[..8].try_into().unwrap_or([0;8]));
            Value::from(n)
        }
        REG_SZ | REG_EXPAND_SZ => {
            let s = String::from_reg_value(v).unwrap_or_default();
            Value::from(s)
        }
        REG_MULTI_SZ => {
            let s: Vec<String> = winreg::types::FromRegValue::from_reg_value(v).unwrap_or_default();
            Value::from(s)
        }
        REG_BINARY => Value::from(v.bytes.clone()),
        _ => Value::Null,
    }
}

/// Escribe un valor. Crea la key si no existe (si create_if_missing).
pub fn write_value(
    hive: &str, key: &str, name: &str, kind: &str, value: &Value, create_if_missing: bool,
) -> AppResult<()> {
    let root = hkey_for(hive)?;
    let subkey = if create_if_missing {
        root.create_subkey(key)
            .map_err(|e| AppError::Registry(format!("create {}\\{}: {}", hive, key, e)))?.0
    } else {
        root.open_subkey_with_flags(key, KEY_SET_VALUE)
            .map_err(|e| AppError::Registry(format!("open {}\\{}: {}", hive, key, e)))?
    };

    match kind {
        "dword" => {
            let n = value.as_u64().unwrap_or(0) as u32;
            subkey.set_value(name, &n).map_err(|e| AppError::Registry(format!("set dword: {}", e)))
        }
        "qword" => {
            let n = value.as_u64().unwrap_or(0);
            subkey.set_value(name, &n).map_err(|e| AppError::Registry(format!("set qword: {}", e)))
        }
        "string" => {
            let s = value.as_str().unwrap_or("").to_string();
            subkey.set_value(name, &s).map_err(|e| AppError::Registry(format!("set string: {}", e)))
        }
        "expand-string" => {
            let s = value.as_str().unwrap_or("").to_string();
            let rv = winreg::RegValue { bytes: encode_wide_string(&s), vtype: REG_EXPAND_SZ };
            subkey.set_raw_value(name, &rv).map_err(|e| AppError::Registry(format!("set expand: {}", e)))
        }
        "multi-string" => {
            let arr = value.as_array().cloned().unwrap_or_default();
            let v: Vec<String> = arr.into_iter().filter_map(|x| x.as_str().map(String::from)).collect();
            subkey.set_value(name, &v).map_err(|e| AppError::Registry(format!("set multi: {}", e)))
        }
        _ => Err(AppError::Registry(format!("kind no soportado: {}", kind))),
    }
}

fn encode_wide_string(s: &str) -> Vec<u8> {
    let mut buf: Vec<u16> = s.encode_utf16().collect();
    buf.push(0);
    buf.into_iter().flat_map(|c| c.to_le_bytes()).collect()
}

/// Borra un valor. Si la key no existe, no-op.
pub fn delete_value(hive: &str, key: &str, name: &str) -> AppResult<()> {
    let root = hkey_for(hive)?;
    let subkey = match root.open_subkey_with_flags(key, KEY_SET_VALUE) {
        Ok(k) => k,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(AppError::Registry(format!("open for delete: {}", e))),
    };
    match subkey.delete_value(name) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Registry(format!("delete: {}", e))),
    }
}

/// Helper para el audit revert. Si previous_value es None, borra. Si es Some, escribe.
pub fn write_value_or_delete(
    hive: &str, key: &str, name: &str, previous_value: Option<&Value>,
) -> AppResult<()> {
    match previous_value {
        Some(v) => {
            let kind = infer_kind(v);
            write_value(hive, key, name, kind, v, true)
        }
        None => delete_value(hive, key, name),
    }
}

fn infer_kind(v: &Value) -> &'static str {
    match v {
        Value::Number(n) if n.is_u64() && n.as_u64().unwrap() <= u32::MAX as u64 => "dword",
        Value::Number(_) => "qword",
        Value::String(_) => "string",
        Value::Array(_) => "multi-string",
        _ => "string",
    }
}
```

#### Paso 1.2 — Test unitario de helpers

**Archivo:** `src-tauri/tests/registry.rs`

```rust
//! Tests del platform layer. Tocan registro real bajo HKCU\Software\ClearTool-Test.

use cleartool::platform::registry as preg;
use serde_json::json;

const TEST_KEY: &str = r"Software\ClearTool-Test";

fn cleanup() {
    let _ = preg::delete_value("HKCU", TEST_KEY, "TestDword");
    let _ = preg::delete_value("HKCU", TEST_KEY, "TestStr");
}

#[test]
fn hive_no_permitido_falla() {
    let r = preg::read_value("HKPD", "", "x");
    assert!(r.is_err());
}

#[test]
fn write_read_dword() {
    cleanup();
    preg::write_value("HKCU", TEST_KEY, "TestDword", "dword", &json!(42), true).unwrap();
    let v = preg::read_value("HKCU", TEST_KEY, "TestDword").unwrap();
    assert_eq!(v, Some(json!(42)));
    cleanup();
}

#[test]
fn write_read_string() {
    cleanup();
    preg::write_value("HKCU", TEST_KEY, "TestStr", "string", &json!("hello"), true).unwrap();
    let v = preg::read_value("HKCU", TEST_KEY, "TestStr").unwrap();
    assert_eq!(v, Some(json!("hello")));
    cleanup();
}

#[test]
fn read_value_inexistente_devuelve_none() {
    cleanup();
    let v = preg::read_value("HKCU", TEST_KEY, "NoExisto").unwrap();
    assert!(v.is_none());
}

#[test]
fn write_or_delete_borra_si_previous_none() {
    cleanup();
    preg::write_value("HKCU", TEST_KEY, "TestDword", "dword", &json!(1), true).unwrap();
    preg::write_value_or_delete("HKCU", TEST_KEY, "TestDword", None).unwrap();
    let v = preg::read_value("HKCU", TEST_KEY, "TestDword").unwrap();
    assert!(v.is_none());
}
```

### Fase 2 — `domain::registry`

#### Paso 2.1 — `list_tweaks` y `read_state`

**Archivo:** `src-tauri/src/domain/registry.rs`

```rust
use crate::core::{AppError, AppResult};
use crate::domain::{catalog, audit};
use crate::models::registry::{
    ApplyTweakInput, ApplyTweakBatchInput, ApplyTweakBatchReport, PerTweakResult,
    RegistryTweak, RegistryOp, TweakState,
};
use crate::models::restore::{ReverseRecipe, RegistryRevertOp};
use crate::platform::registry as preg;
use crate::platform;
use uuid::Uuid;
use serde_json::Value;

pub fn list_tweaks() -> AppResult<Vec<RegistryTweak>> {
    catalog::load_registry_tweaks()
}

pub fn read_state(id: &str) -> AppResult<TweakState> {
    let tweaks = catalog::load_registry_tweaks()?;
    let tweak = tweaks.iter().find(|t| t.id == id)
        .ok_or_else(|| AppError::Permission(format!("tweak no en allowlist: {}", id)))?;

    let mut current = Vec::with_capacity(tweak.operations.len());
    let mut matches_enabled = 0;

    for op in &tweak.operations {
        let v = preg::read_value(&op.hive, &op.key, &op.name)?;
        if v.as_ref().map(|x| x == &op.enabled_value).unwrap_or(false) {
            matches_enabled += 1;
        }
        current.push(v.unwrap_or(Value::Null));
    }

    let total = tweak.operations.len();
    Ok(TweakState {
        id: id.to_string(),
        enabled: matches_enabled == total,
        partial: matches_enabled > 0 && matches_enabled < total,
        current_values: current,
    })
}
```

#### Paso 2.2 — `apply` con audit

```rust
pub fn apply(input: &ApplyTweakInput) -> AppResult<()> {
    audit::with_audit("registry", "applyTweak", input.dry_run, |restore_seq| {
        let tweak = lookup_tweak(&input.id)?;
        let mut previous = Vec::with_capacity(tweak.operations.len());

        // 1. Capturar previos
        for op in &tweak.operations {
            let prev = preg::read_value(&op.hive, &op.key, &op.name)?;
            previous.push(RegistryRevertOp {
                hive: op.hive.clone(),
                key: op.key.clone(),
                name: op.name.clone(),
                kind: op.kind.clone(),
                previous_value: prev,
            });
        }

        // 2. Aplicar (a menos que dry-run)
        if !input.dry_run {
            for op in &tweak.operations {
                let value = if input.enable { &op.enabled_value } else { &op.disabled_value };
                preg::write_value(&op.hive, &op.key, &op.name, &op.kind, value, op.create_if_missing)?;
            }
        }

        Ok(audit::make_entry(
            "registry",
            "applyTweak",
            input.dry_run,
            restore_seq,
            vec![input.id.clone()],
            ReverseRecipe::Registry { operations: previous },
            if input.dry_run { "dry-run" } else { "success" },
            None,
        ))
    }).map(|_| ())
}

fn lookup_tweak(id: &str) -> AppResult<RegistryTweak> {
    catalog::load_registry_tweaks()?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| AppError::Permission(format!("tweak no en allowlist: {}", id)))
}
```

#### Paso 2.3 — `apply_batch` con emit de progreso

```rust
use tauri::{AppHandle, Emitter};

pub fn apply_batch<F>(
    inputs: &[ApplyTweakInput],
    create_restore_point: bool,
    mut emit_progress: F,
) -> AppResult<ApplyTweakBatchReport>
where
    F: FnMut(u32, u32, &str),
{
    let run_id = Uuid::new_v4().to_string();
    let total = inputs.len() as u32;

    let restore_seq = if create_restore_point && inputs.iter().any(|i| !i.dry_run) {
        platform::restore_point::create(
            &format!("ClearTool — batch registry ({})", inputs.len()),
            12, true,
        ).ok()
    } else {
        None
    };

    let mut per: Vec<PerTweakResult> = Vec::with_capacity(inputs.len());
    let mut applied = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (i, input) in inputs.iter().enumerate() {
        emit_progress((i + 1) as u32, total, &input.id);

        match apply(input) {
            Ok(_) => {
                applied += 1;
                per.push(PerTweakResult {
                    id: input.id.clone(),
                    status: if input.dry_run { "dry-run".into() } else { "applied".into() },
                    error: None,
                    previous_values: Vec::new(),
                });
            }
            Err(e) => {
                failed += 1;
                per.push(PerTweakResult {
                    id: input.id.clone(),
                    status: "failed".into(),
                    error: Some(format!("{}", e)),
                    previous_values: Vec::new(),
                });
                // No abortamos. Las operaciones son independientes.
            }
        }
    }

    Ok(ApplyTweakBatchReport {
        run_id, total, applied, failed, skipped, restore_point_seq: restore_seq, per_tweak: per,
    })
}
```

#### Paso 2.4 — `revert` (público — el audit tiene su propio revert también)

```rust
pub fn revert(id: &str) -> AppResult<()> {
    // Revertir = aplicar con enable=false
    apply(&ApplyTweakInput { id: id.to_string(), enable: false, dry_run: false })
}
```

### Fase 3 — IPC commands con emit

#### Paso 3.1 — `ipc::registry`

**Archivo:** `src-tauri/src/ipc/registry.rs`

```rust
use crate::core::AppResult;
use crate::domain;
use crate::models::registry::{
    ApplyTweakInput, ApplyTweakBatchInput, ApplyTweakBatchReport, RegistryTweak, TweakState,
    RegistryProgressEvent,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn list_registry_tweaks() -> AppResult<Vec<RegistryTweak>> {
    domain::registry::list_tweaks()
}

#[tauri::command]
pub async fn read_registry_tweak_state(id: String) -> AppResult<TweakState> {
    domain::registry::read_state(&id)
}

#[tauri::command]
pub async fn apply_registry_tweak(input: ApplyTweakInput) -> AppResult<()> {
    domain::registry::apply(&input)
}

#[tauri::command]
pub async fn apply_registry_tweak_batch(
    app: AppHandle,
    input: ApplyTweakBatchInput,
) -> AppResult<ApplyTweakBatchReport> {
    let run_id_capture = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let app_clone = app.clone();
    let cap = run_id_capture.clone();
    let report = domain::registry::apply_batch(
        &input.inputs,
        input.create_restore_point,
        move |processed, total, current_id| {
            let id = cap.lock().unwrap().clone();
            let _ = app_clone.emit("registry:progress", RegistryProgressEvent {
                run_id: id,
                processed, total, current_id: current_id.to_string(),
            });
        }
    )?;
    *run_id_capture.lock().unwrap() = report.run_id.clone();
    Ok(report)
}

#[tauri::command]
pub async fn revert_registry_tweak(id: String) -> AppResult<()> {
    domain::registry::revert(&id)
}
```

> **Nota:** `Arc<Mutex<String>>` para que el closure pueda leer el run_id que solo se genera **dentro** del batch. Si se necesitan cambios más limpios, generar el run_id en el IPC layer y pasarlo al domain.

### Fase 4 — Frontend

#### Paso 4.1 — Hooks

**Archivo:** `src/features/registry-tweaks/use-registry-tweaks.ts` (nuevo)

```ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import {
  listRegistryTweaks,
  readRegistryTweakState,
  applyRegistryTweakBatch,
  type RegistryTweak,
  type TweakState,
  type ApplyTweakBatchInput,
} from "../../api";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface RegistryProgressEvent {
  runId: string;
  processed: number;
  total: number;
  currentId: string;
}

export function useRegistryTweaks() {
  return useQuery({
    queryKey: ["registry", "tweaks"],
    queryFn: listRegistryTweaks,
  });
}

export function useTweakState(id: string) {
  return useQuery({
    queryKey: ["registry", "state", id],
    queryFn: () => readRegistryTweakState(id),
    staleTime: 30_000,
  });
}

export function useApplyTweakBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: ApplyTweakBatchInput) => applyRegistryTweakBatch(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["registry"] });
      qc.invalidateQueries({ queryKey: ["audit"] });
    },
  });
}

export function useRegistryProgress() {
  const [event, setEvent] = useState<RegistryProgressEvent | null>(null);

  useEffect(() => {
    let un: UnlistenFn | null = null;
    (async () => {
      un = await listen<RegistryProgressEvent>("registry:progress", (e) => setEvent(e.payload));
    })();
    return () => { un?.(); };
  }, []);

  return event;
}
```

#### Paso 4.2 — Componentes a crear

```
src/features/registry-tweaks/
  registry-page.tsx              ← reescribir
  use-registry-tweaks.ts         ← nuevo
  components/
    tweak-row.tsx                ← una fila con check + nombre + badge + button "ver"
    category-section.tsx         ← agrupador por categoría
    tweak-detail-modal.tsx       ← modal §5.2 con diff viewer
    batch-confirm-modal.tsx      ← modal §5.3 pre-aplicar
    batch-progress-modal.tsx     ← modal §5.4 durante aplicación
    risk-badge.tsx               ← reusable
    preset-selector.tsx          ← chips Mínimo/Recomendado/Agresivo
```

#### Paso 4.3 — Esqueleto de `registry-page.tsx`

```tsx
import { useMemo, useState } from "react";
import { useRegistryTweaks, useApplyTweakBatch, useRegistryProgress } from "./use-registry-tweaks";
import type { RegistryTweak } from "../../api";
import { Database } from "lucide-react";
import { EmptyState } from "../../components/empty-state";
import { CategorySection } from "./components/category-section";
import { BatchConfirmModal } from "./components/batch-confirm-modal";
import { BatchProgressModal } from "./components/batch-progress-modal";
import { Button } from "../../components/ui/button";

const PRESETS = ["minimal", "recommended", "aggressive"] as const;
type Preset = (typeof PRESETS)[number];

export function RegistryPage() {
  const { data: tweaks = [], isLoading } = useRegistryTweaks();
  const apply = useApplyTweakBatch();
  const progress = useRegistryProgress();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [confirmOpen, setConfirmOpen] = useState(false);

  const byCategory = useMemo(() => {
    const map = new Map<string, RegistryTweak[]>();
    for (const t of tweaks) {
      const arr = map.get(t.category) ?? [];
      arr.push(t);
      map.set(t.category, arr);
    }
    return map;
  }, [tweaks]);

  const togglePreset = (preset: Preset) => {
    const ids = tweaks.filter((t) => t.presets?.includes(preset)).map((t) => t.id);
    setSelected(new Set(ids));
  };

  const handleApply = (createRestorePoint: boolean, dryRun: boolean) => {
    apply.mutate({
      createRestorePoint,
      inputs: Array.from(selected).map((id) => ({ id, enable: true, dryRun })),
    });
    setConfirmOpen(false);
  };

  if (isLoading) return <div className="p-6 text-muted-foreground">Cargando tweaks...</div>;
  if (tweaks.length === 0) {
    return <EmptyState icon={Database} title="Sin tweaks" description="El catálogo está vacío." />;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <header className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Tweaks de Registro</h2>
          <p className="text-muted-foreground text-sm">
            {tweaks.length} tweaks · {selected.size} seleccionados
          </p>
        </div>
        <div className="flex gap-2">
          {PRESETS.map((p) => (
            <Button key={p} variant="outline" size="sm" onClick={() => togglePreset(p)}>
              Preset {p}
            </Button>
          ))}
          <Button disabled={selected.size === 0} onClick={() => setConfirmOpen(true)}>
            Aplicar ({selected.size})
          </Button>
        </div>
      </header>

      <main className="flex-1 overflow-auto space-y-6">
        {Array.from(byCategory.entries()).map(([cat, items]) => (
          <CategorySection
            key={cat}
            category={cat}
            tweaks={items}
            selected={selected}
            onToggle={(id, on) => {
              setSelected((prev) => {
                const next = new Set(prev);
                on ? next.add(id) : next.delete(id);
                return next;
              });
            }}
          />
        ))}
      </main>

      {confirmOpen && (
        <BatchConfirmModal
          selected={selected}
          tweaks={tweaks}
          onCancel={() => setConfirmOpen(false)}
          onApply={handleApply}
        />
      )}
      {apply.isPending && progress && (
        <BatchProgressModal progress={progress} />
      )}
    </div>
  );
}
```

> Los sub-componentes (`CategorySection`, `BatchConfirmModal`, `BatchProgressModal`, `TweakDetailModal`) los construye la IA en una pasada siguiendo los mockups §5.

---

## 7. Tests

### 7.1 Test integration de apply→revert roundtrip

**Archivo:** `src-tauri/tests/registry_apply.rs`

```rust
//! Test que aplica y revierte un tweak real bajo HKCU. Cleanup defensivo.

use cleartool::domain::registry;
use cleartool::models::registry::ApplyTweakInput;
use cleartool::platform::registry as preg;

const TWEAK_TARGET: &str = "explorer-show-extensions";

#[test]
#[ignore]  // Solo manual; toca HKCU del usuario que corre los tests.
fn aplicar_y_revertir_show_extensions() {
    let estado_inicial = registry::read_state(TWEAK_TARGET).unwrap();

    // Aplicar
    registry::apply(&ApplyTweakInput {
        id: TWEAK_TARGET.to_string(),
        enable: true,
        dry_run: false,
    }).expect("apply ok");

    let estado_aplicado = registry::read_state(TWEAK_TARGET).unwrap();
    assert!(estado_aplicado.enabled, "tras apply, el tweak debe estar enabled");

    // Revertir
    registry::apply(&ApplyTweakInput {
        id: TWEAK_TARGET.to_string(),
        enable: false,
        dry_run: false,
    }).expect("revert ok");

    let estado_revertido = registry::read_state(TWEAK_TARGET).unwrap();
    assert!(!estado_revertido.enabled, "tras revert, el tweak no debe estar enabled");
}
```

### 7.2 Test dry-run no escribe

```rust
#[test]
fn dry_run_no_escribe() {
    let before = preg::read_value("HKCU",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
        "HideFileExt").unwrap();

    registry::apply(&ApplyTweakInput {
        id: "explorer-show-extensions".to_string(),
        enable: true,
        dry_run: true,
    }).expect("dry-run ok");

    let after = preg::read_value("HKCU",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
        "HideFileExt").unwrap();
    assert_eq!(before, after, "dry-run no debe modificar el registro");
}
```

### 7.3 Test allowlist bloquea hives

Ya cubierto en `tests/registry.rs::hive_no_permitido_falla`.

---

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Tweak HKLM falla sin admin | Manifest `requireAdministrator` en release. UI muestra banner si no elevado. |
| Apply mid-batch falla, deja sistema inconsistente | Cada tweak loguea su entry; user puede revertir lo que sí aplicó vía audit log |
| Usuario marca tweak `high risk` (e.g. SmartScreen) sin entender | Modal de confirmación lista `consequences`. Risk badge visible. |
| Tweak no compatible con build de Windows | Filtrar por `min_windows_build` en `list_tweaks` usando build del SO actual |
| Reverse value es None → al revertir borra una key del SO | Aceptable: la key no existía antes del tweak, restaurar es "no existía" |
| Mismatch enabled_value/disabled_value en JSON | Tests de schema verifican estructura |

---

## 9. Definition of Done

- [ ] `platform::registry::{read_value, write_value, delete_value, write_value_or_delete}` implementados.
- [ ] `platform::registry::hkey_for` valida contra `ALLOWED_HIVES`.
- [ ] `domain::registry::list_tweaks` carga del catálogo.
- [ ] `domain::registry::read_state` reporta estado correcto (enabled/partial/disabled).
- [ ] `domain::registry::apply` con audit integration.
- [ ] `domain::registry::apply_batch` con progress emit + restore point.
- [ ] `domain::registry::revert` reusable.
- [ ] IPC: 5 comandos respondiendo (list, read_state, apply, apply_batch, revert).
- [ ] UI: lista por categorías, selector de presets, modal detalle con diff, modal preview pre-batch, modal progreso durante batch.
- [ ] Toggle individual + apply individual funciona.
- [ ] Apply batch crea restore point + escribe N entries audit (una por tweak).
- [ ] Tests `registry.rs` (5+ unit) y `registry_apply.rs` (2+ integration) pasan.
- [ ] Test manual VM: aplicar 3 tweaks, verificar diff visual + audit + reverse.
- [ ] Commit `feat(registry): backend completo + UI con diff viewer + batch con audit`.

---

## 10. Próximo archivo

→ [04-SERVICES.md](04-SERVICES.md) — segundo módulo destructivo. Mismo patrón que registry (catálogo + apply + audit + restore), pero contra el Service Control Manager.
