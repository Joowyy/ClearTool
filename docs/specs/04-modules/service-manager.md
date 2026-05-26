# 04.4 — Módulo Service Manager

## Propósito

Gestionar servicios Windows: enumerar, detener, iniciar, deshabilitar, reactivar y aplicar **listas curadas** de telemetría/servicios redundantes que Windows 11 trae por defecto. Es la dependencia del `cache-cleaner` (ServiceGuard) y del `debloat-engine` (apagado coordinado).

## Stakeholders

- Subagente líder: `windows-systems-expert` + `tauri-rust-backend`.
- Revisión: `security-auditor` (cualquier cambio a la lista negra).

## Capabilities Tauri

- `default.json` para `list_services` (lectura via SCM).
- `elevated.json` para `set_service_state` y `apply_service_preset`.

## Comandos expuestos

```rust
// commands/services.rs

#[tauri::command]
pub async fn list_services(filter: ServiceFilter) -> Result<Vec<ServiceInfo>, AppError>;

#[tauri::command]
pub async fn set_service_state(input: SetServiceStateInput) -> Result<ServiceInfo, AppError>;

#[tauri::command]
pub async fn apply_service_preset(input: ApplyPresetInput) -> Result<PresetReport, AppError>;
```

### Modelos

```rust
// models/service.rs

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct ServiceInfo {
    pub name: String,           // ServiceName
    pub display_name: String,
    pub description: Option<String>,
    pub status: ServiceStatus,  // Stopped | StartPending | Running | StopPending | Paused
    pub start_type: StartType,  // Boot | System | Auto | AutoDelayed | Manual | Disabled
    pub binary_path: Option<String>,
    pub account: Option<String>, // LocalSystem, NetworkService, etc.
    pub depends_on: Vec<String>,
    pub category: Option<ServiceCategory>, // si está en nuestro catálogo
    pub recommended_action: Option<RecommendedAction>, // Disable | KeepManual | KeepAuto | Remove
    pub risk_if_disabled: Risk,
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum ServiceCategory {
    Telemetry,
    Connectivity,
    Update,
    Search,
    Print,
    Xbox,
    Maps,
    Mixedreality,
    Geolocation,
    BiometricSensors,
    OneCore,
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum RecommendedAction {
    Disable,
    Manual,    // dejar en Manual, ahorra arranque
    KeepAuto,  // tocar es mala idea
    Remove,    // sc.exe delete (raro, casi nunca)
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ServiceFilter {
    pub query: Option<String>,
    pub categories: Vec<ServiceCategory>,
    pub only_running: bool,
    pub only_in_catalog: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetServiceStateInput {
    pub name: String,
    pub action: ServiceAction, // Start | Stop | Restart | SetStartType(StartType) | Combined
    pub timeout_secs: u32,     // default 30
    pub dry_run: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApplyPresetInput {
    pub preset: ServicePreset, // TelemetryOff | XboxOff | PrintOff | FullDebloat
    pub dry_run: bool,
    pub create_restore_point: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PresetReport {
    pub run_id: Uuid,
    pub restore_point_id: Option<u32>,
    pub changes: Vec<PerServiceChange>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerServiceChange {
    pub name: String,
    pub before: ServiceSnapshot,  // status + start_type
    pub after: Option<ServiceSnapshot>,
    pub ok: bool,
    pub error: Option<String>,
}
```

## Catálogo

Vive en `.claude/skills/powershell-debloat/RESOURCES/services-catalog.json` (a crear; convive con `bloatware-catalog.json` porque ambos comparten contexto Windows debloat). Estructura:

```json
{
  "version": "0.1.0",
  "presets": {
    "TelemetryOff": ["DiagTrack", "dmwappushservice", "diagnosticshub.standardcollector.service"],
    "XboxOff": ["XblAuthManager", "XblGameSave", "XboxNetApiSvc", "XboxGipSvc"],
    "PrintOff": ["Spooler", "PrintNotify"],
    "FullDebloat": ["DiagTrack", "dmwappushservice", "MapsBroker", "WMPNetworkSvc",
                   "RetailDemo", "WerSvc", "Fax", "lfsvc", "SCardSvr", "WbioSrvc",
                   "TabletInputService", "SharedAccess", "RemoteRegistry", "SSDPSRV",
                   "WSearch?optional"]
  },
  "services": [
    {
      "name": "DiagTrack",
      "display_name": "Connected User Experiences and Telemetry",
      "category": "telemetry",
      "recommended_action": "disable",
      "risk_if_disabled": "low",
      "consequences": ["Apaga la telemetría principal a Microsoft."]
    }
    // ...
  ]
}
```

`?optional` indica que el preset propone pero pide confirmación específica (WSearch desactivado mata la búsqueda en el Explorer; muchos lo quieren conservar en `Manual`).

## Implementación

### Enumeración rápida

```rust
// services/service_manager.rs
use windows_service::service::*;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

pub fn enumerate_all() -> Result<Vec<ServiceInfo>, AppError> {
    let scm = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::ENUMERATE_SERVICE)?;
    // EnumServicesStatusExW (`enumerate_services` del crate envuelve esto).
    // Mapear cada item -> ServiceInfo. Para `description` y `account`, abrir cada servicio
    // con QueryServiceConfig (lazy: solo si el filtro lo requiere) para no enumerar 300x veces.
}
```

Optimización: cacheo en memoria con TTL de 3 segundos. La UI puede pedir refrescos rápidos sin re-bombardear SCM.

### Cambio de estado

```rust
pub fn set_state(input: &SetServiceStateInput) -> Result<ServiceInfo, AppError> {
    // 1. Abrir servicio con SERVICE_QUERY_CONFIG | SERVICE_CHANGE_CONFIG | SERVICE_STOP | SERVICE_START.
    // 2. Si dry_run: solo simular (devolver el estado actual con el cambio "previsto").
    // 3. Stop -> ControlService(Stop) y poll hasta SERVICE_STOPPED o timeout.
    //    Si dependencias activas, encadenar parada de dependientes (con confirmación previa
    //    a nivel comando — el comando debería rechazar si dependientes != [] sin flag explícito).
    // 4. SetStartType -> ChangeServiceConfigW.
    // 5. Start -> StartService + poll.
    // 6. Combined -> stop + setStartType + start (último opcional).
}
```

### Guard pattern para precondiciones

Reutilizado por `cache-cleaner`:

```rust
pub struct ServiceGuard {
    pub name: String,
    pub previous: ServiceSnapshot,
    pub restored: AtomicBool,
}

impl ServiceGuard {
    pub fn stop(name: &str) -> Result<Self, AppError> { /* … */ }
    pub fn restore(self) -> Result<(), AppError> { /* explícito */ }
}

impl Drop for ServiceGuard {
    fn drop(&mut self) {
        if !self.restored.load(Ordering::Relaxed) {
            // Best-effort restore. Loguear con tracing::warn si falla.
        }
    }
}
```

Preferimos `restore()` explícito en el happy path; el `Drop` cubre panics o `?` early-return.

## Catálogo y recomendaciones

Reglas que asignan `recommended_action` por servicio:

- `DiagTrack`, `dmwappushservice` → `Disable`. Riesgo bajo.
- `WSearch` → `Manual`. Riesgo medio (afecta búsqueda Explorer).
- `Spooler` → `Disable` solo si el usuario nunca imprime; default `Manual`.
- `Fax`, `RetailDemo`, `WMPNetworkSvc`, `XblGameSave` → `Disable`. Riesgo bajo.
- `wuauserv`, `bits`, `WaaSMedicSvc` → `KeepAuto`. Tocar = romper Windows Update.
- `WinDefend`, `Sense`, `WdNisSvc`, `SecurityHealthService` → `KeepAuto`. **Nunca** se ofrece deshabilitar desde la UI.

La lista completa vive en el catálogo y debe pasar por `security-auditor` antes de cada PR.

## Eventos

| Evento | Payload | Cuándo |
|---|---|---|
| `services:state-changed` | `ServiceInfo` | Tras `set_service_state` exitoso |
| `services:preset-progress` | `{ run_id, processed, total, current }` | Durante `apply_service_preset` |
| `services:preset-done` | `PresetReport` | Al terminar |

## Preset "FullDebloat"

Cuando el usuario aplica este preset:

1. Se solicita restore point (vía restore-point service).
2. Para cada servicio en el preset:
   - Si está corriendo, intentar Stop con timeout 30 s.
   - Si tiene dependientes corriendo no listados en el preset, **se omite** y se reporta como `dependencies_blocked`.
   - SetStartType = Disabled.
3. Se guarda un snapshot completo `before/after` en el reporte y en audit log.

El preset es **reversible** vía System Restore o regenerando el snapshot inverso desde el JSON del reporte:

```rust
pub fn revert_preset(report: &PresetReport) -> Result<(), AppError> {
    for change in &report.changes {
        if !change.ok { continue; }
        let target = &change.before;
        set_state(&SetServiceStateInput {
            name: change.name.clone(),
            action: ServiceAction::Combined { start: target.status == ServiceStatus::Running, start_type: target.start_type },
            timeout_secs: 30,
            dry_run: false,
        })?;
    }
    Ok(())
}
```

Expuesto como `commands::audit::revert_audit_entry` cuando el entry es de tipo `service_preset`.

## Idempotencia

- Aplicar el mismo preset dos veces: la segunda devuelve `before == after` para servicios ya en el estado deseado, `ok = true`.
- Detener un servicio ya detenido: `ok = true`, `before = after = Stopped`.

## Errores

| Variante | Causa | UX |
|---|---|---|
| `NotElevated` | comando destructivo sin admin | banner |
| `External("Access denied")` | servicio protegido (Defender) | la UI debió bloquear esa acción; reportar bug |
| `Io(Timeout)` | servicio no detuvo en N segundos | botón "forzar (kill PID)" como fallback secundario |
| `Permission` | servicio fuera de catálogo y action != lectura | abortar |

## Tests

- Unit: parser de catálogo (schema strict).
- Unit Rust con un servicio mock (`spawn` un proceso de prueba registrado como servicio en VM). Verificar lifecycle stop/start/disable.
- Integration VM: aplicar `TelemetryOff`, verificar `DiagTrack` en `Stopped + Disabled`. Reaplicar — sin cambios.
- Test de no-regresión: lista de servicios "intocables" (Defender etc.) nunca aparece en presets.

## UI (resumen, detalle en spec 03)

- Tabla virtualizada con columnas: nombre, displayName, status (chip), startType (chip), categoría, recomendación, riesgo.
- Toolbar: filtros, búsqueda, presets aplicables (`TelemetryOff`, `XboxOff`, `FullDebloat`).
- Por fila: dropdown action {Start, Stop, Restart, SetStartType(...)}.
- Modal de detalle: dependencias entrantes/salientes (grafo simple), ruta del binario, descripción.

## Futuro (no en MVP)

- Detección y disable de **scheduled tasks** asociadas a telemetría (`Microsoft\Windows\Customer Experience Improvement Program\*`).
- Visualización de impacto en boot time (medir antes/después con `bootvis`-like).
