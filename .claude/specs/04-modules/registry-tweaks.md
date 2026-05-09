# 04.5 — Módulo Registry Tweaks

## Propósito

Aplicar y revertir un conjunto curado de tweaks al registro Windows: ocultar bloat de la UI (Bing en búsqueda, anuncios de "Sugerencias", widgets, lock-screen ads), endurecer telemetría y exponer opciones de UX (alinear menú a la izquierda, reactivar context menu clásico, deshabilitar Edge precarga, etc.).

Cada tweak es **una operación atómica con reversa explícita y hash esperado del estado original**, no un mutation libre del registro.

## Stakeholders

- Subagente líder: `windows-systems-expert`.
- Apoyo: `tauri-rust-backend`.
- Revisión obligatoria: `security-auditor` para todo cambio al catálogo de tweaks.

## Capabilities Tauri

- `default.json` para `list_registry_tweaks` (enumera + lee estado actual).
- `elevated.json` para `apply_registry_tweak` y `revert_registry_tweak` (escribe).

## Comandos expuestos

```rust
// commands/registry.rs

#[tauri::command]
pub async fn list_registry_tweaks() -> Result<Vec<RegistryTweak>, AppError>;

#[tauri::command]
pub async fn read_registry_tweak_state(id: String) -> Result<TweakState, AppError>;

#[tauri::command]
pub async fn apply_registry_tweak(input: ApplyTweakInput) -> Result<TweakResult, AppError>;

#[tauri::command]
pub async fn revert_registry_tweak(id: String, dry_run: bool) -> Result<TweakResult, AppError>;

#[tauri::command]
pub async fn apply_registry_tweak_batch(input: ApplyTweakBatchInput) -> Result<BatchResult, AppError>;
```

### Modelos

```rust
// models/registry.rs

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct RegistryTweak {
    pub id: String,                 // ej. "ui.start.left-align"
    pub display_name: String,
    pub description: String,
    pub category: TweakCategory,    // Ui | Telemetry | Performance | Privacy | Search | ContextMenu
    pub risk: Risk,
    pub requires_admin: bool,
    pub requires_reboot: bool,
    pub min_build: Option<u32>,
    pub max_build: Option<u32>,
    pub operations: Vec<RegistryOp>, // pueden ser varias (set + delete, etc.)
    pub reverse_operations: Vec<RegistryOp>,
    pub consequences: Vec<String>,
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct RegistryOp {
    pub hive: Hive,                 // HKLM | HKCU | HKCR | HKU | HKCC
    pub path: String,               // sin hive prefix
    pub name: Option<String>,       // None = la "(Default)"; usar "" en RegOp Set para borrar el valor (Default)
    pub op: RegOp,
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum RegOp {
    /// Asegura que el path exista.
    EnsureKey,
    /// Set DWORD/QWORD/SZ/EXPAND_SZ/MULTI_SZ/BINARY (con `expected_type` y `value`).
    SetValue { kind: RegKind, value: RegValue },
    /// Eliminar el valor (no la key).
    DeleteValue,
    /// Eliminar key. Solo permitido si la key cae bajo allowlist.
    DeleteKey,
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum RegKind { Dword, Qword, Sz, ExpandSz, MultiSz, Binary }

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
#[serde(untagged)]
pub enum RegValue {
    Number(i64),
    Text(String),
    Lines(Vec<String>),
    Bytes(Vec<u8>),
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TweakState {
    pub id: String,
    pub status: TweakStatus, // Applied | NotApplied | Mixed | Unknown
    pub current_values: Vec<CurrentValue>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CurrentValue {
    pub op_index: usize,
    pub hive: Hive,
    pub path: String,
    pub name: Option<String>,
    pub kind: Option<RegKind>,
    pub value: Option<RegValue>,
    pub exists: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApplyTweakInput {
    pub id: String,
    pub dry_run: bool,
    pub create_restore_point: bool, // default true cuando dry_run=false
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApplyTweakBatchInput {
    pub ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub stop_on_error: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TweakResult {
    pub id: String,
    pub status: ApplyStatus, // Applied | AlreadyApplied | Reverted | Failed
    pub changes: Vec<RegistryChange>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RegistryChange {
    pub hive: Hive,
    pub path: String,
    pub name: Option<String>,
    pub before: Option<RegValueSnapshot>, // None si no existía
    pub after: Option<RegValueSnapshot>,  // None si se borró
    pub ok: bool,
}
```

## Catálogo

Vive en `.claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.json` (a crear). Los tweaks iniciales recomendados (lista corta, no exhaustiva):

| `id` | Qué hace | Hive | Path | Nombre | Tipo | Valor | Reverse |
|---|---|---|---|---|---|---|---|
| `search.disable-bing` | Quita Bing del search | HKCU | `Software\Policies\Microsoft\Windows\Explorer` | `DisableSearchBoxSuggestions` | DWORD | 1 | DeleteValue |
| `taskbar.left-align` | Alinea el menú Inicio a la izquierda | HKCU | `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarAl` | DWORD | 0 | SetValue 1 |
| `context-menu.classic` | Restaura el context menu clásico de Win10 | HKCU | `Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32` | `(Default)` | SZ | "" | DeleteKey |
| `widgets.disable` | Desactiva widgets en taskbar | HKCU | `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarDa` | DWORD | 0 | SetValue 1 |
| `lockscreen.no-spotlight` | Quita anuncios "Spotlight" en lockscreen | HKLM | `SOFTWARE\Policies\Microsoft\Windows\CloudContent` | `DisableWindowsConsumerFeatures` | DWORD | 1 | DeleteValue |
| `start.no-suggestions` | Quita "Sugerencias" en Start | HKCU | `Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager` | `SubscribedContent-338388Enabled` | DWORD | 0 | SetValue 1 |
| `edge.no-startup-boost` | Apaga el precarga de Edge | HKLM | `SOFTWARE\Policies\Microsoft\Edge` | `StartupBoostEnabled` | DWORD | 0 | DeleteValue |
| `explorer.show-extensions` | Muestra extensiones | HKCU | `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `HideFileExt` | DWORD | 0 | SetValue 1 |
| `explorer.show-hidden` | Muestra archivos ocultos | HKCU | `Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `Hidden` | DWORD | 1 | SetValue 2 |
| `telemetry.allow-zero` | AllowTelemetry=0 (Pro/Enterprise) | HKLM | `SOFTWARE\Policies\Microsoft\Windows\DataCollection` | `AllowTelemetry` | DWORD | 0 | DeleteValue |
| `cortana.disable` | Apaga Cortana via policy | HKLM | `SOFTWARE\Policies\Microsoft\Windows\Windows Search` | `AllowCortana` | DWORD | 0 | DeleteValue |
| `restore.frequency` | SystemRestorePointCreationFrequency=0 | HKLM | `SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore` | `SystemRestorePointCreationFrequency` | DWORD | 0 | DeleteValue |

Cada entrada es `RegistryTweak` completo en JSON con `consequences`, `risk`, etc.

## Implementación segura

### Allowlist de paths

`services::registry::PATH_ALLOWLIST` define los prefijos permitidos por cada tweak. Ningún `RegistryOp` puede tocar paths fuera de la allowlist global:

```rust
const ALLOWED_PREFIXES: &[(Hive, &str)] = &[
    (Hive::Hklm, "SOFTWARE\\Policies\\"),
    (Hive::Hklm, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer"),
    (Hive::Hklm, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore"),
    (Hive::Hkcu, "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer"),
    (Hive::Hkcu, "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager"),
    (Hive::Hkcu, "Software\\Policies\\Microsoft\\Windows\\Explorer"),
    (Hive::Hkcu, "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}"),
    // ...
];
```

Validación al cargar el catálogo: cualquier tweak cuyas `operations` toquen fuera de la allowlist se rechaza con error de inicio (la app no levanta).

### Backend

```rust
// services/registry.rs
use winreg::enums::*;
use winreg::{RegKey, RegValue as WinRegValue};

pub fn apply(tweak: &RegistryTweak, dry_run: bool) -> Result<TweakResult, AppError> {
    let mut changes = vec![];
    // 1. Snapshot ANTES por cada op (para construir reversa runtime y para audit log).
    // 2. Por cada op secuencial:
    //    - dry_run? loguear plan, no escribir.
    //    - real? escribir; capturar cualquier error y abortar el batch del tweak (rollback parcial:
    //      revert con el snapshot ANTES en orden inverso).
    // 3. Verificar `after` post-write.
    // 4. Persistir entry en audit log.
}
```

### Encoding de tipos

- `Dword` → `RegType::REG_DWORD` (4 bytes LE).
- `Qword` → `RegType::REG_QWORD`.
- `Sz` → UTF-16 LE NUL-terminated.
- `ExpandSz` → idem `Sz` con tipo `REG_EXPAND_SZ`.
- `MultiSz` → cada string UTF-16 LE separado por `\0`, doble `\0` final.
- `Binary` → bytes raw.

`winreg` ya hace casi todo; nuestros wrappers solo validan kinds esperados y normalizan errores a `AppError::Registry(String)`.

### Lectura de estado actual

`read_registry_tweak_state` corre las `operations` en modo dry-read y devuelve `TweakState`:

- Si **todos** los valores actuales coinciden con los esperados por las ops → `Applied`.
- Si **ninguno** coincide → `NotApplied`.
- Si parcial → `Mixed`.
- Si error de lectura → `Unknown`.

La UI usa esto al cargar la página `/registry` y al refrescar tras un apply.

## Errores

| Variante | Causa | UX |
|---|---|---|
| `Registry("access denied")` | falta admin para HKLM | banner |
| `Registry("type mismatch")` | el path existe pero con otro `RegKind` | mostrar diálogo "el sistema tiene `X` aquí; ¿forzar?" — no forzar por default |
| `Permission` | path fuera de allowlist | fatal, abortar |
| `RestoreUnavailable` | restore service caído | preguntar al user |

## Idempotencia

- `apply_registry_tweak` sobre un tweak ya aplicado devuelve `ApplyStatus::AlreadyApplied`, `changes` vacío, `ok = true`.
- `revert_registry_tweak` sobre un tweak no aplicado: `Reverted`, `changes` vacío, `ok = true` con warning log.

## Reboot

Algunos tweaks (`taskbar.*`, `explorer.*`) requieren reiniciar `explorer.exe` para verse. Otros (`telemetry.*`, `cortana.*`) requieren reboot completo. Cada tweak declara `requires_reboot: bool`.

`apply_registry_tweak_batch` devuelve un flag agregado `reboot_required` y la UI ofrece:

- "Reiniciar Explorer ahora" (mata `explorer.exe` y lo relanza) — sin admin.
- "Reiniciar Windows ahora" — admin, vía `tauri-plugin-process`.

## Tests

- Unit: validador de allowlist rechaza paths inesperados.
- Unit Rust: round-trip `apply` → `read_state` → `Applied`; luego `revert` → `read_state` → `NotApplied`.
- Unit Rust: si una op falla a mitad de batch, las anteriores se revierten con el snapshot original.
- Snapshot test: serialización JSON del catálogo es estable.
- Integration VM: aplicar batch "UX moderno" → reboot → verificar UI cambió (manual, fuera de CI).

## UI (resumen, detalle en spec 03)

- Tabla con switch por fila (estado actual: aplicado/no/mixto).
- Filtros por categoría y riesgo.
- Tooltip con `description` y `consequences`.
- Botón global "Aplicar paquete recomendado" con preset de tweaks marcados como `risk = low` y `category in (Ui, Privacy, Search)`.
- Modal `<ConfirmDestructive>` cuando se aplica/revierte cualquier tweak con `risk >= medium`.

## Futuro (no en MVP)

- Exportar/importar set de tweaks como `.reg` para usuarios que prefieran aplicar manualmente.
- Comparativa "tu sistema vs preset recomendado" en la home.
