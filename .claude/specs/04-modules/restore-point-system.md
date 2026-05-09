# 04.6 — Módulo Restore Points & Audit Log

## Propósito

Garantizar que **toda operación destructiva** crea un punto de restauración del sistema antes de tocar nada **y** queda registrada en un log JSON Lines (`audit.jsonl`) con la información necesaria para auditar y, donde sea posible, **revertir programáticamente** sin recurrir a System Restore.

Este módulo es **transversal**: el cache cleaner, el debloat engine, el service manager y los registry tweaks dependen de él.

## Stakeholders

- Subagente líder: `windows-systems-expert` (WMI/SrClient) + `tauri-rust-backend`.
- Revisión: `security-auditor` para integridad del log y manejo de claves.

## Dos mecanismos complementarios

1. **System Restore Point (SRP)**: punto nativo de Windows. Crítico para deshacer cambios masivos del registro y archivos del sistema. Lento (5–30 s) pero abarca casi todo el SO.
2. **Operation Audit Log**: JSONL append-only en `%APPDATA%\ClearTool\audit.jsonl` con la receta exacta de cada operación. Permite reversa fina, granular y selectiva (revertir un tweak de registro sin afectar el resto).

Ambos se crean para cada operación destructiva. El usuario puede deshacer:

- **Por operación** (preferido): leer el audit entry y aplicar `reverse_operations`.
- **Por punto**: System Restore al SRP que cubre la operación.

## Capabilities Tauri

- `default.json` para `list_restore_points`, `list_audit_log`.
- `elevated.json` para `create_restore_point`, `restore_to_point`, `revert_audit_entry`.

## Comandos

```rust
// commands/restore.rs

#[tauri::command]
pub async fn create_restore_point(input: CreateRPInput) -> Result<RestorePoint, AppError>;

#[tauri::command]
pub async fn list_restore_points() -> Result<Vec<RestorePoint>, AppError>;

#[tauri::command]
pub async fn restore_to_point(sequence_number: u32) -> Result<(), AppError>;

#[tauri::command]
pub async fn ensure_restore_enabled() -> Result<RestoreReadiness, AppError>;

// commands/audit.rs

#[tauri::command]
pub async fn list_audit_log(filter: AuditFilter) -> Result<Vec<AuditEntry>, AppError>;

#[tauri::command]
pub async fn revert_audit_entry(run_id: Uuid, dry_run: bool) -> Result<RevertReport, AppError>;
```

### Modelos

```rust
// models/restore.rs

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,        // ej. "ClearTool: Debloat Total run 8b3a..."
    pub created_at: DateTime<Utc>,
    pub event_type: SrpEventType,   // BeginSystemChange | EndSystemChange | ...
    pub restore_type: SrpRestoreType, // App, Modify, ...
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateRPInput {
    pub description: String,
    pub force: bool, // si true, intenta sortear el limit de 1/24h tweakeando frequency
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RestoreReadiness {
    pub service_running: bool,           // VSS y Volume Shadow Copy Service
    pub system_drive_protection: bool,   // C: protegido en Restore Settings
    pub frequency_tweaked: bool,         // SystemRestorePointCreationFrequency=0
    pub free_disk_bytes: u64,
    pub last_point_at: Option<DateTime<Utc>>,
    pub recommendations: Vec<String>,
}

// models/audit.rs

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditEntry {
    pub run_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kind: AuditKind, // CleanCache | RemoveBloatware | ApplyRegistryTweak | ServicePreset | ManualTweak
    pub user: String,
    pub elevated: bool,
    pub restore_point_seq: Option<u32>,
    pub payload_summary: String,    // human-readable resumen
    pub reversible: bool,
    pub reverse_recipe: Option<ReverseRecipe>, // None si no se puede revertir programáticamente
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReverseRecipe {
    Registry { ops: Vec<RegistryOp> },
    Services { changes: Vec<PerServiceChange> },  // contiene `before` snapshots
    DebloatStoreReinstall { entries: Vec<String> }, // requiere acción manual
    None,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RevertReport {
    pub run_id: Uuid,
    pub status: RevertStatus, // FullyReverted | PartiallyReverted | NotReversible | Failed
    pub steps: Vec<StepLog>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditFilter {
    pub since: Option<DateTime<Utc>>,
    pub kinds: Vec<AuditKind>,
    pub only_reversible: bool,
    pub limit: Option<u32>,
}
```

## Implementación SRP

### API: WMI vs PS-wrapper

System Restore se manipula vía la clase WMI `SystemRestore` en `root\default`. Tres opciones:

1. **windows-rs WMI bindings** — viable pero verboso, requiere COM init.
2. **PowerShell `Checkpoint-Computer`** — cubre creación pero no listado/restauración.
3. **`SRClient.dll`** + `SRSetRestorePointW` (Win32) — clásico, único método robusto para crear/restaurar sin PS.

**Decisión**: usar `SRSetRestorePointW` desde Rust con `windows-rs`. Para listar puntos, usar WMI vía COM. Para restaurar, usar `SRSetRestorePointW` + `Restore-Computer` PS para ejecutar el restore (requiere reboot).

```rust
// services/restore_point.rs
use windows::Win32::System::Restore::*;

pub fn create(description: &str) -> Result<u32, AppError> {
    let info = RESTOREPOINTINFOW {
        dwEventType: BEGIN_SYSTEM_CHANGE as u32,
        dwRestorePtType: APPLICATION_INSTALL as u32, // marker que no requiere reboot
        llSequenceNumber: 0,
        szDescription: pwstr_from(description),
    };
    let mut status = STATEMGRSTATUS::default();
    let ok = unsafe { SRSetRestorePointW(&info, &mut status) };
    if !ok.as_bool() {
        return Err(AppError::RestoreUnavailable(format!("SRSetRestorePointW failed; nStatus={}", status.nStatus)));
    }
    let seq = status.llSequenceNumber as u32;
    // Cerrar el cambio (END_SYSTEM_CHANGE) tras la operación destructiva queda como
    // responsabilidad del caller. En la práctica abrimos APPLICATION_INSTALL y no cerramos.
    Ok(seq)
}

pub fn list_all() -> Result<Vec<RestorePoint>, AppError> {
    // COM init -> IWbemLocator -> ConnectServer("root\\default")
    // -> CreateInstanceEnum("SystemRestore") -> iterar IWbemClassObject.
}
```

### Frequency tweak

Microsoft limita Restore a 1 punto por 24 h con la key `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore!SystemRestorePointCreationFrequency`. ClearTool tweakea este valor a `0` (la primera vez que crea un punto y `force = true` o como tweak explícito en `/registry`). Sin esto, dos operaciones destructivas en la misma hora compartirían el mismo punto y la segunda no se podría revertir aisladamente.

### `ensure_restore_enabled`

Antes de operaciones destructivas, la UI llama a este comando y muestra un banner si:

- VSS service no corre.
- C: no tiene protección habilitada.
- Espacio libre < 5 % del drive.

Botón "Habilitar System Restore (3 pasos)" automatiza:

1. Start `vss` y `swprv` (set Auto si Manual).
2. `Enable-ComputerRestore -Drive C:\` (PS).
3. Subir reserva mínima a 5 GB en `vssadmin resize shadowstorage` (PS).

## Implementación Audit Log

### Path y formato

- Path: `%APPDATA%\ClearTool\audit.jsonl`.
- Append-only, una línea = un `AuditEntry` JSON.
- Tamaño máx: 50 MB (truncar y rotar a `audit.jsonl.1` cuando se alcance).
- Encoding: UTF-8 sin BOM, `\n` line endings.

### Escritura

```rust
// services/audit_log.rs
pub fn append(entry: &AuditEntry) -> Result<(), AppError> {
    let path = audit_path()?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let line = serde_json::to_string(entry).map_err(|e| AppError::External(e.to_string()))?;
    let mut f = OpenOptions::new().append(true).create(true).open(&path)?;
    writeln!(f, "{line}")?;
    Ok(())
}
```

Las escrituras se serializan vía un `Mutex` global. Si la aplicación crashea a mitad de una operación destructiva, el caller debe haber escrito el `AuditEntry` con un campo `payload_summary = "interrupted"` antes de iniciar el side-effect — variante "two-phase commit" simplificada para tener al menos un registro de la intención.

### Lectura

`list_audit_log` parsea el archivo línea a línea, filtra y devuelve los últimos `limit` (default 200).

### Integridad opcional

Cada entry incluye un `previous_hash` (SHA-256 del entry anterior, hex). Permite detectar manipulación a posteriori (alguien editando audit.jsonl con notepad). Para MVP: no obligatorio, se prepara el campo y queda en `null` si no hay cadena rota; en futuro se firma.

## `revert_audit_entry`

```rust
pub async fn revert(run_id: Uuid, dry_run: bool) -> Result<RevertReport, AppError> {
    // 1. Localizar el entry con run_id; si no existe, error.
    // 2. Si reversible == false, devolver NotReversible y sugerir restore_to_point.
    // 3. Match sobre reverse_recipe:
    //    - Registry { ops } -> registry::apply_ops_raw(ops, dry_run).
    //    - Services { changes } -> service_manager::revert_preset(changes, dry_run).
    //    - DebloatStoreReinstall -> emit step "manual" con instrucciones; no automatizamos.
    // 4. Persistir un nuevo AuditEntry kind=Revert con run_id_referenced=run_id.
}
```

`revert` siempre crea su propia entrada de audit, nunca borra ni edita la entrada original.

## Eventos

| Evento | Payload | Cuándo |
|---|---|---|
| `restore:create-progress` | `{ phase: "preparing"|"writing"|"finalizing" }` | Durante `create_restore_point` |
| `restore:point-created` | `RestorePoint` | OK |
| `restore:restore-started` | `{ sequence_number }` | Tras llamar `Restore-Computer` |
| `audit:appended` | `AuditEntry` | Tras cada append |

`restore_to_point` es síncrono pero requiere reboot inmediato; el handler emite el evento y la UI muestra modal "Tu PC se reiniciará en 60 s para completar la restauración. Cancelar | Reiniciar ahora".

## Idempotencia

- Crear dos restore points con la misma descripción dentro del mismo segundo: ambos puntos quedan; el segundo recibe un nuevo `sequence_number`.
- `revert_audit_entry` re-ejecutado sobre un entry ya revertido: detecta que el estado actual coincide con el `after` de la reversa; devuelve `FullyReverted` con `steps = []`.

## Errores

| Variante | Causa | UX |
|---|---|---|
| `RestoreUnavailable("disabled")` | Restore deshabilitado en C: | banner + botón "habilitar" |
| `RestoreUnavailable("frequency limit")` | < 24h sin tweak frequency | preguntar si aplicar tweak `restore.frequency` |
| `Io` audit.jsonl | disco lleno / permisos | bloquear operaciones destructivas; "no podemos garantizar reversibilidad" |
| `NotElevated` | crear/restaurar sin admin | banner |

## Tests

- Unit: serializar/deserializar `AuditEntry` con todos los `ReverseRecipe` variants.
- Unit: appender concurrente (10 hilos × 100 entries) → archivo bien formado, conteo correcto, no líneas truncadas.
- Integration VM: crear punto → modificar registro → revert via audit → estado coincide con before.
- Integration VM: crear punto → modificar 50 cosas → `restore_to_point` → reboot → estado coincide con baseline.

## UI (resumen, detalle en spec 03)

- Página `/restore`:
  - Tabla de restore points (sequence, fecha, descripción, tipo).
  - Botón "Crear punto manual" (description input).
  - Botón "Restaurar al punto N" (con confirm + warning de reboot).
- Página `/audit` (subpágina dentro de `/restore` o standalone):
  - Tabla de audit entries, filtros por kind y reversibilidad.
  - Por fila: ver detalle JSON, "Deshacer este cambio".

## Retention

- Restore points: Windows los rota automáticamente según el espacio reservado para Shadow Storage (default 5 %). No tocamos.
- Audit log: rotar a 50 MB; conservar el archivo `.1` anterior. Borrar `.2` en adelante. Configurable en `/settings`.

## Futuro (no en MVP)

- Firmar audit.jsonl con clave generada por instalación.
- Sync opcional del audit log a un fichero del usuario para análisis offline.
- Detector de "drift" — comparar estado del sistema con la suma de audit entries y reportar inconsistencias.
