# 04 — Gestión de Servicios de Windows

> **Posición:** 4/14.
> **Dependencias:** [00-CATALOGOS](00-CATALOGOS.md), [01-RESTORE-POINTS](01-RESTORE-POINTS.md), [02-AUDIT-LOG](02-AUDIT-LOG.md).
> **Output:** `domain::services` y `platform::services` completos contra SCM, UI con presets + warning de dependencias.

---

## 1. Resumen ejecutivo

Estado actual:

| Componente | Estado |
|---|---|
| `domain::services::*` | ❌ Todos `NotImplemented` |
| `platform::services` | ❌ Stub |
| Catálogo `services-catalog.json` | ✅ Curado en [00-CATALOGOS](00-CATALOGOS.md) |
| UI `services-page.tsx` | 🟡 Tabla básica contra stub |

Cierre del archivo:
- Listado de servicios reales del SCM, anotados con info del catálogo.
- `set_state` (start type + start/stop) con validación contra catálogo.
- `apply_preset` aplica los 3 presets curados en batch.
- UI con search, filtro por categoría, badges de riesgo, modal de detalle con dependencias.

---

## 2. Diagnóstico

### 2.1 API: SCM directo vs PowerShell vs `windows-service` crate

| Opción | Pros | Contras |
|---|---|---|
| `windows-service` crate | Idiomatic Rust, async-friendly | Solo soporta operaciones limitadas |
| `windows-rs` raw SCM | Control total | Verboso, manejo de handles a mano |
| PowerShell `Get-Service`/`Set-Service` | Más simple | Lento (~200-500ms por invocación), parsing |

**Decisión:** `windows-rs` SCM directo para `list`/`set_start_type`/`start`/`stop`. PowerShell solo como fallback de debug.

### 2.2 Servicios protegidos por SCM

Algunos servicios son `SERVICE_TYPE_KERNEL_DRIVER` o están marcados como protegidos. Cualquier intento de `ChangeServiceConfig` falla con `ERROR_ACCESS_DENIED` aunque seamos admin. **No abortar** — reportar `protected` y continuar.

### 2.3 Dependencias en cascada

Si parar `WSearch` (Windows Search), automáticamente para `OneSyncSvc` que depende. SCM lo hace solo, pero hay que **avisar al usuario antes**.

`platform::services::dependencies_of(name)` con `EnumDependentServicesW` resuelve esto.

### 2.4 Estado vs Start type

Dos conceptos separados:
- **Estado:** Running, Stopped, StartPending, StopPending, Paused.
- **Start type:** Boot, System, Automatic, AutomaticDelayed, Manual, Disabled.

`set_state(name, start_type)` cambia el start type. Si además queremos parar el servicio ahora, llamamos `stop(name)` por separado.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| `windows-rs` direct para SCM | Performance + sin dependencia externa |
| Validación: servicio debe estar en el catálogo (allowlist) | Seguridad por construcción |
| Cambio de start type + opcional stop en la misma operación | UI typical: "disable + stop now" |
| Servicios protegidos reportan `error_kind: protected` no abortan batch | Realista en sistemas con AV/EDR |
| Lista full de servicios anotada (no solo los del catálogo) | UX: el user puede ver TODO, pero solo tunear los curados |

---

## 4. Modelo de datos

```rust
// models/service.rs (ya esbozado en 00-CATALOGOS §8.3.1)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_type: String,
    pub description: Option<String>,
    pub in_catalog: bool,
    pub catalog: Option<ServiceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetServiceStateInput {
    pub name: String,
    pub start_type: String,
    pub stop_now: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyServicePresetInput {
    pub preset: String,         // "minimal" | "recommended" | "aggressive"
    pub stop_now: bool,
    pub dry_run: bool,
    pub create_restore_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPresetReport {
    pub run_id: String,
    pub preset: String,
    pub total: u32,
    pub applied: u32,
    pub failed: u32,
    pub protected: u32,
    pub skipped: u32,
    pub restore_point_seq: Option<u32>,
    pub per_service: Vec<PerServiceResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerServiceResult {
    pub name: String,
    pub status: String,    // "applied" | "skipped" | "failed" | "protected" | "dry-run"
    pub previous_start_type: String,
    pub previous_state: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProgressEvent {
    pub run_id: String,
    pub processed: u32,
    pub total: u32,
    pub current_name: String,
}
```

---

## 5. Plan UX

### 5.1 Pantalla principal

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Servicios de Windows                                            [Refrescar]  │
│ 248 servicios · 12 en catálogo · 58 tuneables                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Buscar...] [Categoría ▾] [Estado ▾] [Solo catálogo ☑]                       │
│                                                                              │
│ [Preset mínimo (1)] [Preset recomendado (27)] [Preset agresivo (45)]         │
│ [Aplicar selección (15)] [Limpiar selección]                                 │
│                                                                              │
│ ☑ Servicio              State    StartType        Categoría    Riesgo Acción│
│ ─────────────────────────────────────────────────────────────────────────── │
│ ☑ DiagTrack             Running  Automatic        Telemetry    Bajo   [▾]   │
│   Connected User Experiences and Telemetry                                  │
│ ─────────                                                                    │
│ ☐ MapsBroker            Stopped  AutomaticDelayed Media        Bajo   [▾]   │
│   Downloaded Maps Manager                                                    │
│ ─────────                                                                    │
│ ☐ Spooler               Running  Automatic        Printing     ALTO   [▾]   │
│   Print Spooler — ⚠ Necesario para imprimir                                  │
│ ...                                                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Modal de detalle de servicio

```
┌─────────────────────────────────────────────────────────────────────────┐
│  DiagTrack                                                               │
│  Connected User Experiences and Telemetry                                │
├─────────────────────────────────────────────────────────────────────────┤
│  Estado:        Running                                                  │
│  Start type:    Automatic                                                │
│  Categoría:     Telemetry                                                │
│  Riesgo:        Bajo                                                     │
│  Recomendado:   Disabled                                                 │
│                                                                          │
│  Descripción:                                                            │
│   Recolecta y envía datos de uso a Microsoft. Es el servicio central    │
│   de telemetría.                                                         │
│                                                                          │
│  Consecuencias de deshabilitar:                                          │
│   • Algunos diagnósticos de Feedback Hub dejan de funcionar.            │
│   • Reduce tráfico saliente a vortex.data.microsoft.com.                │
│                                                                          │
│  Dependencias:                                                           │
│   ─ Depende de: RpcSs (necesario)                                       │
│   ─ Lo usan: (ninguno)                                                  │
│                                                                          │
│  Cambiar start type a:                                                   │
│   ○ Automatic   ○ Manual   ● Disabled                                    │
│   ☑ Detener ahora si está corriendo                                      │
│                                                                          │
│  [Cancelar]                                          [Aplicar]           │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Modal pre-batch (preset)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Aplicar preset "Recomendado"                                             │
├──────────────────────────────────────────────────────────────────────────┤
│  27 servicios serán modificados:                                          │
│   • 18 → Disabled                                                         │
│   • 9 → Manual                                                            │
│                                                                           │
│  Servicios afectados:                                                     │
│   DiagTrack, dmwappushservice, RetailDemo, MapsBroker, RemoteRegistry,   │
│   WerSvc, XblAuthManager, XblGameSave, ...                                │
│                                                                           │
│  ☑ Detener servicios que estén corriendo ahora                            │
│  ☑ Crear punto de restauración antes                                      │
│  ☐ Dry-run (solo simular)                                                 │
│                                                                           │
│  Estimación: ~30 segundos.                                                │
│                                                                           │
│  [Cancelar]                                              [Aplicar →]      │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.4 Banner si no elevado

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ⚠  Modificar servicios requiere permisos de administrador.                │
│    La app está en modo limitado — solo lectura.                           │
│                                                  [Reiniciar como admin]   │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Plan de implementación por fases

### Fase 1 — `platform::services` con `windows-rs` SCM

#### Paso 1.1 — Dependencias `Cargo.toml`

```toml
[dependencies.windows]
features = [
    # ya añadidos en 01-RESTORE-POINTS
    "Win32_System_Services",
    "Win32_System_Threading",
]
```

#### Paso 1.2 — `list_all`

**Archivo:** `src-tauri/src/platform/services.rs`

```rust
use crate::core::{AppError, AppResult};
use windows::core::*;
use windows::Win32::System::Services::*;
use windows::Win32::Foundation::*;

#[derive(Debug, Clone)]
pub struct ServiceRuntime {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_type: String,
    pub description: Option<String>,
}

pub fn list_all() -> AppResult<Vec<ServiceRuntime>> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ENUMERATE_SERVICE)
            .map_err(|e| AppError::Services(format!("OpenSCManagerW: {:?}", e)))?;

        let mut bytes_needed = 0u32;
        let mut services_returned = 0u32;
        let mut resume_handle = 0u32;
        let mut buf: Vec<u8> = vec![0; 1024 * 1024];  // 1MB inicial

        loop {
            let r = EnumServicesStatusExW(
                scm,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32 | SERVICE_DRIVER,
                SERVICE_STATE_ALL,
                Some(&mut buf),
                &mut bytes_needed,
                &mut services_returned,
                Some(&mut resume_handle),
                PCWSTR::null(),
            );

            if r.is_ok() {
                break;
            }
            // Si falta espacio, agrandar buf y reintentar
            let need = (bytes_needed as usize).max(buf.len() * 2);
            if need > 16 * 1024 * 1024 {
                let _ = CloseServiceHandle(scm);
                return Err(AppError::Services("buffer enum > 16MB".into()));
            }
            buf.resize(need, 0);
        }

        let services_ptr = buf.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW;
        let services_slice = std::slice::from_raw_parts(services_ptr, services_returned as usize);

        let mut out = Vec::with_capacity(services_returned as usize);
        for s in services_slice {
            let name = pcwstr_to_string(s.lpServiceName);
            let display = pcwstr_to_string(s.lpDisplayName);
            let state = state_to_str(s.ServiceStatusProcess.dwCurrentState);
            let start_type = query_start_type(scm, &name).unwrap_or_else(|_| "Unknown".to_string());
            let description = query_description(scm, &name).ok();

            out.push(ServiceRuntime {
                name, display_name: display, state, start_type, description,
            });
        }

        let _ = CloseServiceHandle(scm);
        Ok(out)
    }
}

unsafe fn pcwstr_to_string(p: PWSTR) -> String {
    if p.is_null() { return String::new(); }
    let mut len = 0;
    while *p.0.add(len) != 0 { len += 1; }
    let slice = std::slice::from_raw_parts(p.0, len);
    String::from_utf16_lossy(slice)
}

fn state_to_str(s: SERVICE_STATUS_CURRENT_STATE) -> String {
    match s.0 {
        1 => "Stopped",
        2 => "StartPending",
        3 => "StopPending",
        4 => "Running",
        5 => "ContinuePending",
        6 => "PausePending",
        7 => "Paused",
        _ => "Unknown",
    }.to_string()
}

unsafe fn query_start_type(scm: SC_HANDLE, name: &str) -> AppResult<String> {
    let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
    let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_QUERY_CONFIG)
        .map_err(|e| AppError::Services(format!("OpenServiceW({}): {:?}", name, e)))?;

    let mut bytes_needed = 0u32;
    let mut buf: Vec<u8> = vec![0; 4096];
    let r = QueryServiceConfigW(h, Some(buf.as_mut_ptr() as *mut _), buf.len() as u32, &mut bytes_needed);
    if r.is_err() {
        buf.resize(bytes_needed as usize, 0);
        let _ = QueryServiceConfigW(h, Some(buf.as_mut_ptr() as *mut _), buf.len() as u32, &mut bytes_needed);
    }

    let cfg = &*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);
    let st = start_type_to_str(cfg.dwStartType, cfg.dwServiceType);

    let _ = CloseServiceHandle(h);
    Ok(st)
}

fn start_type_to_str(start_type: u32, _service_type: u32) -> String {
    match start_type {
        0 => "Boot",
        1 => "System",
        2 => "Automatic",  // puede ser AutomaticDelayed — requeriría query separada
        3 => "Manual",
        4 => "Disabled",
        _ => "Unknown",
    }.to_string()
}

unsafe fn query_description(scm: SC_HANDLE, name: &str) -> AppResult<String> {
    let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
    let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_QUERY_CONFIG)
        .map_err(|e| AppError::Services(format!("OpenServiceW desc: {:?}", e)))?;

    let mut bytes_needed = 0u32;
    let mut buf: Vec<u8> = vec![0; 4096];
    let r = QueryServiceConfig2W(h, SERVICE_CONFIG_DESCRIPTION, Some(&mut buf), &mut bytes_needed);
    let _ = r;

    let desc = &*(buf.as_ptr() as *const SERVICE_DESCRIPTIONW);
    let s = if desc.lpDescription.is_null() {
        String::new()
    } else {
        let mut len = 0;
        while *desc.lpDescription.0.add(len) != 0 { len += 1; }
        let slice = std::slice::from_raw_parts(desc.lpDescription.0, len);
        String::from_utf16_lossy(slice)
    };

    let _ = CloseServiceHandle(h);
    Ok(s)
}
```

> **Nota:** `AutomaticDelayed` requiere query separada de `SERVICE_CONFIG_DELAYED_AUTO_START_INFO`. Para v1.0, reportamos `Automatic` y lo refinamos en v1.1.

#### Paso 1.3 — `set_start_type`

```rust
const START_BOOT: u32 = 0;
const START_SYSTEM: u32 = 1;
const START_AUTO: u32 = 2;
const START_MANUAL: u32 = 3;
const START_DISABLED: u32 = 4;

pub fn set_start_type(name: &str, start_type: &str) -> AppResult<()> {
    let st = match start_type {
        "Boot" => START_BOOT,
        "System" => START_SYSTEM,
        "Automatic" | "AutomaticDelayed" => START_AUTO,
        "Manual" => START_MANUAL,
        "Disabled" => START_DISABLED,
        _ => return Err(AppError::Services(format!("start_type inválido: {}", start_type))),
    };

    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .map_err(|e| AppError::Services(format!("OpenSCManager: {:?}", e)))?;
        let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_CHANGE_CONFIG)
            .map_err(|e| AppError::Services(format!("OpenService change: {:?}", e)))?;

        let r = ChangeServiceConfigW(
            h,
            SERVICE_NO_CHANGE,
            ENUM_SERVICE_TYPE(st),     // dwStartType en realidad — el wrapper de windows-rs lo expone aquí
            SERVICE_NO_CHANGE,
            PCWSTR::null(), PCWSTR::null(), None, PCWSTR::null(), PCWSTR::null(), PCWSTR::null(), PCWSTR::null(),
        );

        let _ = CloseServiceHandle(h);
        let _ = CloseServiceHandle(scm);

        r.map_err(|e| {
            if e.code() == E_ACCESSDENIED {
                AppError::Permission(format!("ChangeServiceConfig {}: protected", name))
            } else {
                AppError::Services(format!("ChangeServiceConfig: {:?}", e))
            }
        })
    }
}
```

> **Caveat:** `ChangeServiceConfigW` espera `dwStartType` como u32 directo, no como enum. Si `windows-rs` lo expone tipado, ajustar el cast. Mantener el patrón.

#### Paso 1.4 — `start` / `stop`

```rust
pub fn start(name: &str) -> AppResult<()> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .map_err(|e| AppError::Services(format!("OpenSCManager: {:?}", e)))?;
        let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_START)
            .map_err(|e| AppError::Services(format!("OpenService start: {:?}", e)))?;
        let r = StartServiceW(h, None);
        let _ = CloseServiceHandle(h);
        let _ = CloseServiceHandle(scm);
        r.map_err(|e| AppError::Services(format!("StartServiceW: {:?}", e)))
    }
}

pub fn stop(name: &str) -> AppResult<()> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .map_err(|e| AppError::Services(format!("OpenSCManager: {:?}", e)))?;
        let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_STOP)
            .map_err(|e| AppError::Services(format!("OpenService stop: {:?}", e)))?;
        let mut status = SERVICE_STATUS::default();
        let r = ControlService(h, SERVICE_CONTROL_STOP, &mut status);
        let _ = CloseServiceHandle(h);
        let _ = CloseServiceHandle(scm);
        r.map_err(|e| AppError::Services(format!("ControlService stop: {:?}", e)))
    }
}
```

#### Paso 1.5 — `dependencies_of`

```rust
pub fn dependencies_of(name: &str) -> AppResult<Vec<String>> {
    // Devuelve los servicios que dependen DE este (no en los que este depende).
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)?;
        let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_ENUM_DEPENDENTS)?;

        let mut bytes_needed = 0u32;
        let mut services_returned = 0u32;
        let mut buf: Vec<u8> = vec![0; 4096];
        loop {
            let r = EnumDependentServicesW(
                h, SERVICE_STATE_ALL, Some(&mut buf), &mut bytes_needed, &mut services_returned,
            );
            if r.is_ok() { break; }
            let need = (bytes_needed as usize).max(buf.len() * 2);
            if need > 1024 * 1024 { break; }
            buf.resize(need, 0);
        }

        let ptr = buf.as_ptr() as *const ENUM_SERVICE_STATUSW;
        let slice = std::slice::from_raw_parts(ptr, services_returned as usize);
        let names = slice.iter().map(|s| pcwstr_to_string(s.lpServiceName)).collect();

        let _ = CloseServiceHandle(h);
        let _ = CloseServiceHandle(scm);
        Ok(names)
    }
}
```

### Fase 2 — `domain::services`

#### Paso 2.1 — `list` anota con catálogo

**Archivo:** `src-tauri/src/domain/services.rs`

```rust
use crate::core::{AppError, AppResult};
use crate::domain::{audit, catalog};
use crate::models::service::{Service, ServiceEntry};
use crate::models::restore::ReverseRecipe;
use crate::platform;
use std::collections::HashMap;

pub fn list() -> AppResult<Vec<Service>> {
    let runtime = platform::services::list_all()?;
    let catalog_entries = catalog::load_services_catalog().unwrap_or_default();
    let cat_map: HashMap<String, ServiceEntry> = catalog_entries
        .into_iter()
        .map(|e| (e.service_name.clone(), e))
        .collect();

    Ok(runtime.into_iter().map(|r| {
        let in_catalog = cat_map.contains_key(&r.name);
        let catalog = cat_map.get(&r.name).cloned();
        Service {
            name: r.name,
            display_name: r.display_name,
            state: r.state,
            start_type: r.start_type,
            description: r.description,
            in_catalog,
            catalog,
        }
    }).collect())
}
```

#### Paso 2.2 — `set_state` con allowlist + audit

```rust
use crate::models::service::{SetServiceStateInput};

pub fn set_state(input: &SetServiceStateInput) -> AppResult<()> {
    if !catalog::is_service_allowed(&input.name) {
        return Err(AppError::Permission(format!("servicio no en catálogo: {}", input.name)));
    }

    audit::with_audit("services", "setState", input.dry_run, |restore_seq| {
        // Capturar estado previo
        let all = platform::services::list_all()?;
        let prev = all.iter().find(|s| s.name == input.name)
            .ok_or_else(|| AppError::Services(format!("servicio no encontrado: {}", input.name)))?;
        let previous_start = prev.start_type.clone();
        let previous_state = prev.state.clone();

        if !input.dry_run {
            // Aplicar
            platform::services::set_start_type(&input.name, &input.start_type)?;
            if input.stop_now && previous_state == "Running" {
                let _ = platform::services::stop(&input.name);
            }
        }

        Ok(audit::make_entry(
            "services",
            "setState",
            input.dry_run,
            restore_seq,
            vec![input.name.clone()],
            ReverseRecipe::Service {
                service_name: input.name.clone(),
                previous_start_type: previous_start,
                previous_state,
            },
            if input.dry_run { "dry-run" } else { "success" },
            None,
        ))
    }).map(|_| ())
}
```

#### Paso 2.3 — `apply_preset`

```rust
use crate::models::service::{ApplyServicePresetInput, ApplyPresetReport, PerServiceResult};
use uuid::Uuid;

pub fn apply_preset<F>(
    input: &ApplyServicePresetInput,
    mut emit_progress: F,
) -> AppResult<ApplyPresetReport>
where
    F: FnMut(u32, u32, &str),
{
    let entries = catalog::load_services_catalog()?;
    let candidates: Vec<&ServiceEntry> = entries
        .iter()
        .filter(|e| e.presets.iter().any(|p| p == &input.preset))
        .collect();
    let total = candidates.len() as u32;

    let restore_seq = if input.create_restore_point && !input.dry_run {
        platform::restore_point::create(
            &format!("ClearTool — preset servicios '{}'", input.preset),
            12, true,
        ).ok()
    } else { None };

    let mut applied = 0u32;
    let mut failed = 0u32;
    let mut protected = 0u32;
    let mut per: Vec<PerServiceResult> = Vec::with_capacity(candidates.len());

    let all = platform::services::list_all()?;
    let runtime_map: std::collections::HashMap<String, _> =
        all.into_iter().map(|r| (r.name.clone(), r)).collect();

    for (i, entry) in candidates.iter().enumerate() {
        emit_progress((i + 1) as u32, total, &entry.service_name);
        let recommended = entry.recommended_start_type.clone().unwrap_or_else(|| "Manual".into());
        let runtime = runtime_map.get(&entry.service_name);
        let prev_start = runtime.map(|r| r.start_type.clone()).unwrap_or_else(|| "Unknown".into());
        let prev_state = runtime.map(|r| r.state.clone()).unwrap_or_else(|| "Unknown".into());

        let result = set_state(&crate::models::service::SetServiceStateInput {
            name: entry.service_name.clone(),
            start_type: recommended.clone(),
            stop_now: input.stop_now,
            dry_run: input.dry_run,
        });

        match result {
            Ok(_) => {
                applied += 1;
                per.push(PerServiceResult {
                    name: entry.service_name.clone(),
                    status: if input.dry_run { "dry-run".into() } else { "applied".into() },
                    previous_start_type: prev_start, previous_state: prev_state,
                    error: None,
                });
            }
            Err(AppError::Permission(msg)) if msg.contains("protected") => {
                protected += 1;
                per.push(PerServiceResult {
                    name: entry.service_name.clone(),
                    status: "protected".into(),
                    previous_start_type: prev_start, previous_state: prev_state,
                    error: Some(msg),
                });
            }
            Err(e) => {
                failed += 1;
                per.push(PerServiceResult {
                    name: entry.service_name.clone(),
                    status: "failed".into(),
                    previous_start_type: prev_start, previous_state: prev_state,
                    error: Some(format!("{}", e)),
                });
            }
        }
    }

    Ok(ApplyPresetReport {
        run_id: Uuid::new_v4().to_string(),
        preset: input.preset.clone(),
        total, applied, failed, protected, skipped: 0,
        restore_point_seq: restore_seq,
        per_service: per,
    })
}
```

### Fase 3 — IPC

**Archivo:** `src-tauri/src/ipc/services.rs`

```rust
use crate::core::AppResult;
use crate::domain;
use crate::models::service::{
    Service, SetServiceStateInput, ApplyServicePresetInput, ApplyPresetReport, ServiceProgressEvent,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn list_services() -> AppResult<Vec<Service>> {
    domain::services::list()
}

#[tauri::command]
pub async fn set_service_state(input: SetServiceStateInput) -> AppResult<()> {
    domain::services::set_state(&input)
}

#[tauri::command]
pub async fn service_dependencies(name: String) -> AppResult<Vec<String>> {
    crate::platform::services::dependencies_of(&name)
}

#[tauri::command]
pub async fn apply_service_preset(
    app: AppHandle,
    input: ApplyServicePresetInput,
) -> AppResult<ApplyPresetReport> {
    domain::services::apply_preset(&input, move |processed, total, current| {
        let _ = app.emit("services:progress", ServiceProgressEvent {
            run_id: String::new(),  // se rellena al final si hace falta
            processed, total, current_name: current.to_string(),
        });
    })
}
```

### Fase 4 — Frontend

#### Paso 4.1 — Cliente API

```ts
// src/api/client.ts
export const listServices = () => invoke<Service[]>("list_services");
export const setServiceState = (input: SetServiceStateInput) =>
  invoke<void>("set_service_state", { input });
export const serviceDependencies = (name: string) =>
  invoke<string[]>("service_dependencies", { name });
export const applyServicePreset = (input: ApplyServicePresetInput) =>
  invoke<ApplyPresetReport>("apply_service_preset", { input });
```

#### Paso 4.2 — Componentes a crear

```
src/features/services/
  services-page.tsx              ← reescribir
  use-services.ts                ← nuevo
  components/
    service-row.tsx
    service-detail-modal.tsx     ← §5.2
    preset-confirm-modal.tsx     ← §5.3
    progress-modal.tsx
    dependency-warning.tsx
    state-badge.tsx
    risk-badge.tsx               ← compartido con registry (extraer a components/ui/)
```

#### Paso 4.3 — Hook `useServices`

```ts
// src/features/services/use-services.ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  listServices, setServiceState, applyServicePreset,
  type Service, type SetServiceStateInput, type ApplyServicePresetInput,
} from "../../api";

export function useServicesList() {
  return useQuery({
    queryKey: ["services"],
    queryFn: listServices,
    staleTime: 10_000,
  });
}

export function useSetServiceState() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: SetServiceStateInput) => setServiceState(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["services"] }),
  });
}

export function useApplyPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: ApplyServicePresetInput) => applyServicePreset(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["services"] });
      qc.invalidateQueries({ queryKey: ["audit"] });
    },
  });
}

export interface ServicesProgress {
  processed: number;
  total: number;
  currentName: string;
}

export function useServicesProgress() {
  const [event, setEvent] = useState<ServicesProgress | null>(null);
  useEffect(() => {
    let un: UnlistenFn | null = null;
    (async () => {
      un = await listen<ServicesProgress>("services:progress", (e) => setEvent(e.payload));
    })();
    return () => { un?.(); };
  }, []);
  return event;
}
```

#### Paso 4.4 — Tabla virtualizada

`@tanstack/react-virtual` ya está. Implementar virtual scroll en `services-page.tsx` porque 248 filas son demasiadas para renderizar todas:

```tsx
import { useVirtualizer } from "@tanstack/react-virtual";

const rowVirtualizer = useVirtualizer({
  count: filtered.length,
  getScrollElement: () => scrollRef.current,
  estimateSize: () => 56,
  overscan: 8,
});
```

---

## 7. Tests

### 7.1 Test del platform layer

**Archivo:** `src-tauri/tests/services_platform.rs`

```rust
use cleartool::platform::services as ps;

#[test]
fn list_all_no_panicea_y_devuelve_lista() {
    let r = ps::list_all().expect("list ok");
    assert!(r.len() > 30, "esperado >30 servicios en Win11, got {}", r.len());
    assert!(r.iter().any(|s| s.name == "RpcSs"), "RpcSs siempre presente");
}

#[test]
#[ignore]  // toca SCM real — admin requerido
fn set_start_type_servicio_test() {
    // Crear un servicio dummy para test sería ideal; en su defecto, usar uno benigno.
    // Esta prueba es manual.
}
```

### 7.2 Test del domain layer (mock)

Si se quiere test sin tocar SCM, abstraer `platform::services` detrás de un trait. **Pero** para v1.0, los tests integration sobre VM son suficientes:

```rust
#[test]
fn allowlist_rechaza_servicio_fuera() {
    let r = cleartool::domain::services::set_state(&cleartool::models::service::SetServiceStateInput {
        name: "ServicioInventado123XYZ".into(),
        start_type: "Disabled".into(),
        stop_now: false,
        dry_run: true,
    });
    assert!(matches!(r, Err(cleartool::core::AppError::Permission(_))));
}
```

### 7.3 Test manual VM

1. VM Win11 limpia + admin.
2. Abrir `/services` → debe listar ~250 servicios.
3. Filtro "Solo catálogo" → ~58 servicios.
4. Click "Preset recomendado" → modal § 5.3 muestra ~27 servicios.
5. Aceptar → restore point creado, progress modal va corriendo.
6. Verificar en `services.msc` que `DiagTrack` cambió a Disabled.
7. Ir a `/audit` → ver 27 entries.
8. Revertir una → `DiagTrack` vuelve a Automatic + Running.

---

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| `ERROR_ACCESS_DENIED` en servicios protegidos del SO | Categorizar como `protected`, continuar batch |
| Parar servicios críticos (Spooler con impresora) | Catálogo curado los marca `risk: high`. UI muestra badge "ALTO". Disclaimer obligatorio. |
| Cambios en SCM corrompen el sistema | Restore point obligatorio antes de batch |
| `EnumDependentServicesW` falla con servicios complejos | Try-catch, devuelve lista vacía. UI no muestra warning si no hay datos. |
| Operación lenta en Win11 H2 (~5s por servicio) | Progress modal con feedback inmediato, async |
| `set_start_type` exitoso pero `stop` falla | Reportar `partial` en audit, dejar start type cambiado (next boot lo aplica) |

---

## 9. Definition of Done

- [ ] `platform::services::list_all` lista todos los servicios con state + start_type + description.
- [ ] `platform::services::set_start_type` cambia el start type.
- [ ] `platform::services::start` y `stop` funcionan.
- [ ] `platform::services::dependencies_of` lista dependientes.
- [ ] `domain::services::list` anota con info de catálogo.
- [ ] `domain::services::set_state` valida allowlist + audit.
- [ ] `domain::services::apply_preset` con progress emit + restore point.
- [ ] IPC: 4 comandos (list, set_state, dependencies, apply_preset).
- [ ] UI: pantalla con tabla virtualizada, search, filtros, presets, modales detalle/confirm/progress.
- [ ] Test platform layer: `list_all_no_panicea_y_devuelve_lista` pasa.
- [ ] Test allowlist: `allowlist_rechaza_servicio_fuera` pasa.
- [ ] Test manual VM: preset recomendado aplica 27 servicios, audit registra todos.
- [ ] Commit `feat(services): backend SCM + UI con presets y audit`.

---

## 10. Próximo archivo

→ [05-DEBLOAT.md](05-DEBLOAT.md) — el módulo más complejo: Appx packages + uninstaller strings + PowerShell embebido. Reusa el patrón ya probado en registry y services.
