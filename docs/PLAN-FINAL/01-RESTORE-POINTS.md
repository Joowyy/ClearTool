# 01 — Sistema de Puntos de Restauración

> **Posición:** 1/14. Prerequisito de los módulos 03, 04, 05, 06.
> **Dependencias:** [00-CATALOGOS](00-CATALOGOS.md) (allowlists ya cargan al startup).
> **Output esperado al cierre:** `domain::restore` y `platform::restore_point` completos. UI funcional. Comando reusable `ensure_or_create_restore_point()` que llaman los demás módulos.

---

## 1. Resumen ejecutivo

Cada operación destructiva en ClearTool **debe** crear un Restore Point del sistema antes de actuar. Decisión arquitectónica fijada en `CLAUDE.md` (no negociable).

Estado actual:

| Componente | Estado |
|---|---|
| `domain::restore::ensure_enabled` | ❌ Stub |
| `domain::restore::create` | ❌ Stub |
| `domain::restore::list` | ❌ Stub |
| `domain::restore::restore_to` | ❌ Stub |
| `platform::restore_point` | 🟡 Archivo existe, vacío de implementación |
| UI `restore-points` | 🟡 Llama al backend pero recibe `NotImplemented` |

---

## 2. Diagnóstico

### 2.1 APIs Windows disponibles

| API | Pros | Contras |
|---|---|---|
| `SRSetRestorePointW` (advapi32 → srrstr.dll) | Documentado, oficial, rápido | Requiere admin |
| WMI `SystemRestore.CreateRestorePoint` | Más fácil de leer en PowerShell | Más lento, requiere instanciación de COM |
| `vssadmin create shadow` | Snapshot VSS, más profundo | NO crea Restore Point — es otro mecanismo, no apto |

**Decisión:** `SRSetRestorePointW` para crear. WMI para listar y restaurar (más simple que `SRGetRestorePointList`).

### 2.2 Throttling de System Restore

Windows tiene **throttling de 24h** entre Restore Points creados por la misma "razón" (event type). Si ClearTool crea 5 puntos en una sesión, solo el primero queda.

**Mitigación:** registry key `SystemRestorePointCreationFrequency=0` en `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore`. Esto bypasa el throttling. **PERO** modificar esa key persiste, contaminando el sistema. La técnica correcta es:

1. Guardar valor previo de `SystemRestorePointCreationFrequency`.
2. Setear a 0.
3. Crear el punto.
4. Restaurar el valor previo.

Implementado en `platform::restore_point::create_with_bypass`.

### 2.3 System Protection puede estar desactivado

En instalaciones limpias de Windows 11, **System Protection está OFF por default** para C:\. Hay que activarlo. Esto requiere admin y se hace con `Enable-ComputerRestore -Drive "C:\"` (PowerShell) o vía WMI `SystemRestore.Enable`.

### 2.4 Tamaño mínimo de disco

System Restore necesita un volumen reservado >= 300 MB. En discos muy pequeños puede fallar. Detectar y avisar.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| `SRSetRestorePointW` via `windows-rs` crate | API nativa Win32, no requiere PowerShell |
| WMI `Win32_SystemRestore` para list/restore_to | PowerShell helper, output JSON parseable |
| Bypass del throttling con guardar/restore de key | Permite múltiples puntos en sesión sin contaminar |
| Event type = `BEGIN_NESTED_SYSTEM_CHANGE` (102) | Marca el punto como anidado, agrupable |
| Restore type = `MODIFY_SETTINGS` (12) o `APPLICATION_INSTALL` (0) | Según el módulo invocante |
| Polling sin bloquear: comando async, return inmediato del scheduled id | `create_restore_point` puede tardar 5-30s. UI muestra spinner. |
| `ensure_enabled()` se llama desde la UI en el primer arranque | Aviso si System Protection está OFF, opción de activarlo |

---

## 4. Modelo de datos

```rust
// models/restore.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,
    pub creation_time: String,       // ISO 8601
    pub restore_point_type: u32,     // 0..14
    pub event_type: u32,             // 100..103
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRestorePointInput {
    pub description: String,
    /// Opcional. Default: MODIFY_SETTINGS=12
    pub restore_type: Option<u32>,
    /// Si true, intenta bypass del throttling 24h. Default: true.
    #[serde(default = "default_bypass")]
    pub bypass_throttle: bool,
}

fn default_bypass() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub sequence_number: u32,
    pub created_at: String,
    pub description: String,
    pub bypassed_throttle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub run_id: String,
    pub timestamp: String,
    pub module: String,            // "cache" | "debloat" | "registry" | "services"
    pub operation: String,         // "clean" | "remove" | "apply_tweak" | "stop_service"
    pub dry_run: bool,
    pub restore_point_seq: Option<u32>,
    pub items_affected: Vec<String>,
    pub reverse_recipe: serde_json::Value,
    pub status: String,            // "success" | "partial" | "failed"
    pub error: Option<String>,
}
```

---

## 5. Plan UX

### 5.1 Pantalla "Puntos de Restauración"

```
┌──────────────────────────────────────────────────────────────────────┐
│ Puntos de Restauración                       [Refrescar]  [Crear ↻]  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│ ⚠ System Protection está DESACTIVADO para C:\                         │
│   Sin esta protección, ClearTool NO puede crear puntos antes de       │
│   tocar el sistema. Recomendado activar.                              │
│                            [Activar protección para C:\]              │
│                                                                       │
│ ─────────────────────────────────────────────────────────────────    │
│                                                                       │
│ Seq    Fecha              Descripción                  Tipo           │
│ ────   ─────────────────  ──────────────────────────   ───────────    │
│ 142    2026-05-21 14:32   ClearTool — pre-clean        AppInstall     │
│ 141    2026-05-20 23:01   Windows Update auto          ModifySettings │
│ 138    2026-05-15 09:14   Driver Installation          Driver         │
│ 135    2026-05-10 18:55   Manual                       ModifySettings │
│ ...                                                                   │
│                                                                       │
│ ─────────────────────────────────────────────────────────────────    │
│ [Restaurar a seleccionado...]                                         │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.2 Modal "Crear punto"

```
┌──────────────────────────────────────────────────┐
│  Crear punto de restauración                     │
├──────────────────────────────────────────────────┤
│  Descripción:                                    │
│  [ClearTool — punto manual_______________]       │
│                                                  │
│  ☑ Bypass del throttling de 24h                  │
│      (recomendado, restaura el valor previo)     │
│                                                  │
│  Este proceso puede tardar 10-30 segundos.       │
│                                                  │
│  [Cancelar]                  [Crear punto →]     │
└──────────────────────────────────────────────────┘
```

### 5.3 Modal "Restaurar a punto"

```
┌──────────────────────────────────────────────────┐
│  ⚠  Restaurar el sistema                          │
├──────────────────────────────────────────────────┤
│  Vas a restaurar Windows al estado:              │
│      Seq #138 — Driver Installation              │
│      2026-05-15 09:14                            │
│                                                  │
│  Esta acción:                                    │
│  • Reinicia el sistema.                          │
│  • Revierte cambios de registro, drivers, apps.  │
│  • NO afecta documentos del usuario.             │
│  • Tarda 5-15 minutos.                           │
│  • Es IRREVERSIBLE una vez iniciada.             │
│                                                  │
│  [Cancelar]              [Continuar y reiniciar] │
└──────────────────────────────────────────────────┘
```

---

## 6. Plan de implementación por fases

### Fase 1 — Backend: `platform::restore_point`

#### Paso 1.1 — Añadir dependencias a `Cargo.toml`

```toml
[dependencies.windows]
version = "0.58"
features = [
    "Win32_Foundation",
    "Win32_System_Registry",
    "Win32_System_Restore",
    "Win32_System_Wmi",
    "Win32_System_Com",
    "Win32_System_Ole",
    "Win32_System_Variant",
]
```

#### Paso 1.2 — Implementar `create()` con `SRSetRestorePointW`

**Archivo:** `src-tauri/src/platform/restore_point.rs`

```rust
use crate::core::{AppError, AppResult};
use windows::core::*;
use windows::Win32::System::Restore::*;
use winreg::enums::*;
use winreg::RegKey;

const RESTORE_FREQ_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
const RESTORE_FREQ_VAL: &str = "SystemRestorePointCreationFrequency";

/// Crea un Restore Point. Devuelve el SequenceNumber asignado por el SO.
///
/// `restore_type`: típicamente `MODIFY_SETTINGS=12` o `APPLICATION_INSTALL=0`.
/// `event_type`:   `BEGIN_NESTED_SYSTEM_CHANGE=102` (creamos atómico).
pub fn create(description: &str, restore_type: u32, bypass_throttle: bool) -> AppResult<u32> {
    let original_freq = if bypass_throttle {
        set_throttle_bypass()?
    } else {
        None
    };

    let result = create_inner(description, restore_type);

    if bypass_throttle {
        restore_throttle(original_freq);
    }

    result
}

fn create_inner(description: &str, restore_type: u32) -> AppResult<u32> {
    let mut desc_wide: Vec<u16> = description.encode_utf16().collect();
    desc_wide.truncate(64);  // RP_DESCRIPTION_MAX = 64
    desc_wide.push(0);

    let mut info = RESTOREPOINTINFOW {
        dwEventType: BEGIN_NESTED_SYSTEM_CHANGE as u32,
        dwRestorePtType: restore_type,
        llSequenceNumber: 0,
        szDescription: [0u16; 64],
    };
    let copy_len = desc_wide.len().min(64);
    info.szDescription[..copy_len].copy_from_slice(&desc_wide[..copy_len]);

    let mut status = STATEMGRSTATUS::default();

    let ok = unsafe { SRSetRestorePointW(&info, &mut status) };
    if !ok.as_bool() {
        return Err(AppError::RestorePoint(format!(
            "SRSetRestorePointW failed: nStatus={}", status.nStatus
        )));
    }

    // Cerrar el nested change inmediatamente (no estamos esperando más operaciones del SO).
    let mut close_info = RESTOREPOINTINFOW {
        dwEventType: END_NESTED_SYSTEM_CHANGE as u32,
        dwRestorePtType: restore_type,
        llSequenceNumber: status.llSequenceNumber,
        szDescription: [0u16; 64],
    };
    let mut close_status = STATEMGRSTATUS::default();
    let _ = unsafe { SRSetRestorePointW(&close_info, &mut close_status) };

    Ok(status.llSequenceNumber as u32)
}

fn set_throttle_bypass() -> AppResult<Option<u32>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm.create_subkey(RESTORE_FREQ_KEY)
        .map_err(|e| AppError::Registry(format!("open {}: {}", RESTORE_FREQ_KEY, e)))?;
    let previous: Option<u32> = key.get_value(RESTORE_FREQ_VAL).ok();
    key.set_value(RESTORE_FREQ_VAL, &0u32)
        .map_err(|e| AppError::Registry(format!("set {}: {}", RESTORE_FREQ_VAL, e)))?;
    Ok(previous)
}

fn restore_throttle(previous: Option<u32>) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match hklm.open_subkey_with_flags(RESTORE_FREQ_KEY, KEY_SET_VALUE) {
        Ok(k) => k,
        Err(_) => return,
    };
    match previous {
        Some(v) => { let _ = key.set_value(RESTORE_FREQ_VAL, &v); }
        None => { let _ = key.delete_value(RESTORE_FREQ_VAL); }
    }
}
```

#### Paso 1.3 — Implementar `list()` via PowerShell + WMI

WMI directo desde Rust es factible pero verboso. Para `list`, usamos PowerShell embebido (read-only, sin riesgo):

```rust
// platform/restore_point.rs (continúa)

use crate::platform::powershell;

const LIST_SCRIPT: &str = r#"
Get-ComputerRestorePoint | Select-Object SequenceNumber, Description, CreationTime, RestorePointType, EventType | ConvertTo-Json -Compress -Depth 2
"#;

#[derive(serde::Deserialize)]
struct PsRestorePoint {
    #[serde(rename = "SequenceNumber")]
    sequence_number: u32,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "CreationTime")]
    creation_time: String,       // WMI datetime: "20260521143200.000000-300"
    #[serde(rename = "RestorePointType")]
    restore_point_type: u32,
    #[serde(rename = "EventType")]
    event_type: u32,
}

pub fn list() -> AppResult<Vec<crate::models::restore::RestorePoint>> {
    let out = powershell::run_script(LIST_SCRIPT)?;
    if !out.status.success() {
        return Err(AppError::PowerShell(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // PowerShell devuelve objeto único si hay solo uno, array si hay múltiples.
    let parsed: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| AppError::Parse(format!("list restore: {}", e)))?;
    let items: Vec<PsRestorePoint> = match parsed {
        serde_json::Value::Array(_) => serde_json::from_value(parsed)
            .map_err(|e| AppError::Parse(format!("array: {}", e)))?,
        serde_json::Value::Object(_) => vec![serde_json::from_value(parsed)
            .map_err(|e| AppError::Parse(format!("single: {}", e)))?],
        _ => Vec::new(),
    };

    Ok(items.into_iter().map(|p| crate::models::restore::RestorePoint {
        sequence_number: p.sequence_number,
        description: p.description.unwrap_or_default(),
        creation_time: parse_wmi_datetime(&p.creation_time),
        restore_point_type: p.restore_point_type,
        event_type: p.event_type,
    }).collect())
}

/// Convierte "20260521143200.000000-300" a "2026-05-21T14:32:00Z" aproximado.
fn parse_wmi_datetime(s: &str) -> String {
    if s.len() < 14 {
        return s.to_string();
    }
    let year = &s[0..4];
    let month = &s[4..6];
    let day = &s[6..8];
    let h = &s[8..10];
    let m = &s[10..12];
    let sec = &s[12..14];
    format!("{}-{}-{}T{}:{}:{}Z", year, month, day, h, m, sec)
}
```

#### Paso 1.4 — Implementar `restore_to()` via PowerShell

```rust
const RESTORE_TO_SCRIPT_TMPL: &str = "Restore-Computer -RestorePoint {SEQ} -Confirm:$false";

pub fn restore_to(sequence_number: u32) -> AppResult<()> {
    // Validar input — solo número entero. No hay inyección posible.
    let script = format!("Restore-Computer -RestorePoint {} -Confirm:$false", sequence_number);
    // Restore-Computer reinicia la máquina. La invocación es "fire and forget".
    let out = powershell::run_script_owned(&script)?;
    if !out.status.success() {
        return Err(AppError::PowerShell(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    Ok(())
}
```

> **Nota:** `run_script_owned` (variante de `run_script` que acepta `&str` no `&'static str`) debe existir en `platform::powershell` para los pocos casos donde el script es seguro pero parametrizado por integer. Validar input es entero ANTES de construir el script.

#### Paso 1.5 — Implementar `ensure_enabled()`

```rust
const CHECK_ENABLED_SCRIPT: &str = r#"
$d = Get-WmiObject -Namespace 'root\default' -Class SystemRestoreConfig -ErrorAction SilentlyContinue
if ($null -eq $d) {
    @{ enabled = $false; reason = 'wmi-class-missing' } | ConvertTo-Json -Compress
    return
}
# RPSessionInterval > 0 = enabled
@{ enabled = ($d.RPSessionInterval -gt 0); reason = 'ok' } | ConvertTo-Json -Compress
"#;

const ENABLE_SCRIPT: &str = "Enable-ComputerRestore -Drive 'C:\\'";

#[derive(serde::Deserialize)]
struct EnabledCheck {
    enabled: bool,
    #[allow(dead_code)]
    reason: String,
}

pub fn is_enabled() -> AppResult<bool> {
    let out = powershell::run_script(CHECK_ENABLED_SCRIPT)?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let chk: EnabledCheck = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("is_enabled: {}", e)))?;
    Ok(chk.enabled)
}

pub fn enable_for_system_drive() -> AppResult<()> {
    let out = powershell::run_script(ENABLE_SCRIPT)?;
    if !out.status.success() {
        return Err(AppError::PowerShell(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    Ok(())
}
```

### Fase 2 — Backend: `domain::restore`

#### Paso 2.1 — Implementar las 4 funciones del domain

**Archivo:** `src-tauri/src/domain/restore.rs`

```rust
use crate::core::{AppError, AppResult};
use crate::models::restore::{CreateRestorePointInput, RestorePoint, RestoreReport};
use crate::platform;
use chrono::Utc;

pub fn ensure_enabled() -> AppResult<bool> {
    platform::restore_point::is_enabled()
}

pub fn enable_for_system_drive() -> AppResult<()> {
    platform::restore_point::enable_for_system_drive()
}

pub fn create(input: &CreateRestorePointInput) -> AppResult<RestoreReport> {
    let restore_type = input.restore_type.unwrap_or(12);  // MODIFY_SETTINGS
    let seq = platform::restore_point::create(
        &input.description,
        restore_type,
        input.bypass_throttle,
    )?;
    Ok(RestoreReport {
        sequence_number: seq,
        created_at: Utc::now().to_rfc3339(),
        description: input.description.clone(),
        bypassed_throttle: input.bypass_throttle,
    })
}

pub fn list() -> AppResult<Vec<RestorePoint>> {
    platform::restore_point::list()
}

pub fn restore_to(sequence_number: u32) -> AppResult<()> {
    platform::restore_point::restore_to(sequence_number)
}

/// Helper reusable: garantiza que existe (o crea) un Restore Point antes de una
/// operación destructiva. Los demás módulos lo llaman al inicio de `clean()`,
/// `apply()`, `remove()`, etc.
///
/// Si System Protection está OFF, devuelve `Ok(None)` y deja que el caller decida
/// si abortar o continuar sin punto.
pub fn ensure_or_create(description: &str) -> AppResult<Option<u32>> {
    if !platform::restore_point::is_enabled()? {
        log::warn!("System Protection OFF — operación sin restore point");
        return Ok(None);
    }
    let report = create(&CreateRestorePointInput {
        description: description.to_string(),
        restore_type: Some(0),  // APPLICATION_INSTALL
        bypass_throttle: true,
    })?;
    Ok(Some(report.sequence_number))
}
```

### Fase 3 — IPC commands

#### Paso 3.1 — Ajustar `ipc::restore`

**Archivo:** `src-tauri/src/ipc/restore.rs`

```rust
use crate::core::AppResult;
use crate::domain;
use crate::models::restore::{CreateRestorePointInput, RestorePoint, RestoreReport};

#[tauri::command]
pub async fn ensure_restore_enabled() -> AppResult<bool> {
    domain::restore::ensure_enabled()
}

#[tauri::command]
pub async fn enable_system_protection() -> AppResult<()> {
    domain::restore::enable_for_system_drive()
}

#[tauri::command]
pub async fn create_restore_point(input: CreateRestorePointInput) -> AppResult<RestoreReport> {
    domain::restore::create(&input)
}

#[tauri::command]
pub async fn list_restore_points() -> AppResult<Vec<RestorePoint>> {
    domain::restore::list()
}

#[tauri::command]
pub async fn restore_to_point(sequence_number: u32) -> AppResult<()> {
    domain::restore::restore_to(sequence_number)
}
```

#### Paso 3.2 — Registrar `enable_system_protection` en `lib.rs`

Buscar el `tauri::generate_handler![...]` y añadir `ipc::restore::enable_system_protection`.

### Fase 4 — Frontend

#### Paso 4.1 — Cliente API

**Archivo:** `src/api/client.ts` — añadir:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { CreateRestorePointInput, RestorePoint, RestoreReport } from "./types";

export const ensureRestoreEnabled = () =>
  invoke<boolean>("ensure_restore_enabled");

export const enableSystemProtection = () =>
  invoke<void>("enable_system_protection");

export const createRestorePoint = (input: CreateRestorePointInput) =>
  invoke<RestoreReport>("create_restore_point", { input });

export const listRestorePoints = () =>
  invoke<RestorePoint[]>("list_restore_points");

export const restoreToPoint = (sequenceNumber: number) =>
  invoke<void>("restore_to_point", { sequenceNumber });
```

#### Paso 4.2 — Tipos TS

**Archivo:** `src/api/types.ts` — añadir:

```ts
export interface RestorePoint {
  sequenceNumber: number;
  description: string;
  creationTime: string;
  restorePointType: number;
  eventType: number;
}

export interface CreateRestorePointInput {
  description: string;
  restoreType?: number;
  bypassThrottle?: boolean;
}

export interface RestoreReport {
  sequenceNumber: number;
  createdAt: string;
  description: string;
  bypassedThrottle: boolean;
}
```

#### Paso 4.3 — Hook `useRestorePoints`

**Archivo:** `src/features/restore-points/use-restore-points.ts` (nuevo)

```ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  ensureRestoreEnabled,
  enableSystemProtection,
  listRestorePoints,
  createRestorePoint,
  restoreToPoint,
  type CreateRestorePointInput,
} from "../../api";

export function useSystemProtectionStatus() {
  return useQuery({
    queryKey: ["restore", "enabled"],
    queryFn: ensureRestoreEnabled,
    staleTime: 60_000,
  });
}

export function useEnableSystemProtection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: enableSystemProtection,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["restore"] });
    },
  });
}

export function useRestorePointsList() {
  return useQuery({
    queryKey: ["restore", "points"],
    queryFn: listRestorePoints,
    staleTime: 30_000,
  });
}

export function useCreateRestorePoint() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateRestorePointInput) => createRestorePoint(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["restore", "points"] });
    },
  });
}

export function useRestoreToPoint() {
  return useMutation({
    mutationFn: (seq: number) => restoreToPoint(seq),
  });
}
```

#### Paso 4.4 — Reescribir `restore-page.tsx`

**Archivo:** `src/features/restore-points/restore-page.tsx`

Estructura nueva (ver mockup §5.1). Componentes a extraer:

- `SystemProtectionBanner` — muestra el aviso si OFF + botón "Activar".
- `RestorePointsTable` — tabla virtualizada.
- `CreateRestorePointModal`.
- `ConfirmRestoreModal`.

(El detalle del JSX lo construye la IA en sesión 2 — los hooks y la API ya están en su sitio.)

### Fase 5 — Integración con otros módulos

#### Paso 5.1 — Exportar helper `ensure_or_create`

Ya está creado en `domain::restore::ensure_or_create`. Los módulos siguientes (03, 04, 05, 06) lo llaman así:

```rust
// Ejemplo desde domain::registry::apply
pub fn apply(input: &ApplyTweakInput) -> AppResult<()> {
    if !input.dry_run {
        let seq = crate::domain::restore::ensure_or_create(&format!(
            "ClearTool — pre-tweak {}", input.id
        ))?;
        // ... más adelante el audit log usa esto:
        // audit_entry.restore_point_seq = seq;
    }
    // ... operación
    Ok(())
}
```

---

## 7. Tests

### 7.1 Test unitario `platform::restore_point::parse_wmi_datetime`

```rust
#[test]
fn parse_wmi_datetime_basico() {
    let s = "20260521143200.000000-300";
    assert_eq!(super::parse_wmi_datetime(s), "2026-05-21T14:32:00Z");
}
```

### 7.2 Test integration en VM

**Archivo:** `src-tauri/tests/restore_integration.rs`

```rust
//! Solo corre en Windows con `--features integration-tests`.
//! No en CI default — requiere admin + System Protection ON.

#[cfg(all(windows, feature = "integration-tests"))]
mod windows_only {
    use cleartool::domain::restore;

    #[test]
    fn is_enabled_no_panicea() {
        let _ = restore::ensure_enabled();
    }

    #[test]
    fn list_no_panicea() {
        let _ = restore::list();
    }

    /// Crea un punto, lo verifica en list, NO lo borra (no hay API limpia para eso).
    #[test]
    #[ignore]  // Solo manual — modifica el SO.
    fn ciclo_completo() {
        let input = cleartool::models::restore::CreateRestorePointInput {
            description: "ClearTool TEST — borrar manualmente".to_string(),
            restore_type: Some(0),
            bypass_throttle: true,
        };
        let report = restore::create(&input).expect("create ok");
        assert!(report.sequence_number > 0);

        let points = restore::list().expect("list ok");
        assert!(points.iter().any(|p| p.sequence_number == report.sequence_number));
    }
}
```

### 7.3 Test manual en VM limpia (procedimiento)

1. VM Windows 11 limpia, **System Protection OFF** por defecto.
2. Abrir ClearTool → ir a `/restore`.
3. **Esperado:** banner amarillo "System Protection está DESACTIVADO".
4. Click "Activar protección". UAC salta.
5. **Esperado:** banner desaparece, lista se refresca.
6. Click "Crear punto". Modal aparece.
7. Aceptar con descripción default.
8. **Esperado:** spinner ~15s, luego el punto aparece arriba de la lista.
9. Verificar en `SystemPropertiesProtection.exe` (panel nativo) que el punto está visible.

---

## 8. Errores comunes y resolución

| Error | Causa | Fix |
|---|---|---|
| `SRSetRestorePointW failed: nStatus=-2147219712` | System Protection OFF | Llamar `enable_for_system_drive()` primero |
| `SRSetRestorePointW failed: nStatus=-2147024891` (E_ACCESSDENIED) | App no elevada | Manifest `requireAdministrator` en release |
| `Restore-Computer: The system cannot find the file` | El SequenceNumber no existe | Refrescar lista antes de restore |
| `list()` devuelve `[]` siempre | Política de grupo deshabilita Restore | Documentar limitación; no es bug |
| Múltiples puntos creados en sesión, solo aparece uno | Throttling 24h activo | `bypass_throttle: true` (default) |

---

## 9. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Cambiar `SystemRestorePointCreationFrequency` y olvidar restaurarlo | `restore_throttle()` se llama en `defer`-like (siempre tras `create_inner`, éxito o fallo) |
| `Restore-Computer` reinicia mid-operación de otro módulo | Restore solo se dispara desde la pantalla `/restore` por acción explícita |
| Lectura de WMI lenta (>5s) | `staleTime: 30_000` en React Query |
| Falla silenciosa: `is_enabled()` devuelve Err → UI asume disabled | Mostrar mensaje claro de error en banner |

---

## 10. Definition of Done

- [ ] `platform::restore_point::create` funciona en VM Windows 11 con admin.
- [ ] `platform::restore_point::list` devuelve los puntos visibles en `SystemPropertiesProtection.exe`.
- [ ] `platform::restore_point::is_enabled` y `enable_for_system_drive` funcionan.
- [ ] `domain::restore::ensure_or_create` reusable por otros módulos.
- [ ] Throttling bypass funcional: 3 puntos creados en 1 minuto, los 3 visibles.
- [ ] Throttling key restaurada a su valor original tras cada create (verificable con `Get-ItemProperty`).
- [ ] Comandos Tauri respondiendo (5 comandos).
- [ ] UI muestra banner si protección OFF, lista de puntos, modales create/restore.
- [ ] Test `parse_wmi_datetime` pasa.
- [ ] Test manual en VM limpia documentado y ejecutado al menos una vez.
- [ ] Commit `feat(restore): puntos de restauración funcionales + helper ensure_or_create`.

---

## 11. Próximo archivo

→ [02-AUDIT-LOG.md](02-AUDIT-LOG.md) — el otro brazo de la reversibilidad: el audit log JSONL con reverse recipes que permite deshacer operaciones **una por una** sin restaurar todo el SO.
