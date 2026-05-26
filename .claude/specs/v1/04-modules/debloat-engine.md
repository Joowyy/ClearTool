# 04.3 — Módulo Debloat Engine

## Propósito

Eliminar de forma agresiva pero auditable todo el ecosistema de aplicaciones preinstaladas, paquetes Microsoft no esenciales, telemetría y componentes UWP que el usuario no quiere. Soporta:

1. **Apps consumer** (Candy Crush, TikTok, Spotify, etc.) — quitar Appx por usuario y provisionado.
2. **Microsoft consumer** (News, Weather, Solitaire, Sticky Notes, Maps, Mail/Calendar, Xbox stack…).
3. **Microsoft "core"** (Teams system-wide, Cortana, Widgets, Copilot, OneDrive, Edge donde sea legal).
4. **Telemetría** (DiagTrack, dmwappushservice, RetailDemo) — coordina con el `service-manager`.

## Postura por defecto

**Perfil "Total" activado por defecto.** El usuario verá todo el catálogo marcado al entrar. Tres botones de preset reordenan la selección:

- `Mínimo` → solo `category in (consumer-app, ai)` y `risk = low`.
- `Recomendado` → todo lo anterior + `category in (telemetry, ms-consumer)`.
- `Total` (default) → toda la lista, incluyendo `ms-core` y entradas con `risk = high`.

Cada vez que el usuario cambia un preset, se muestra inline cuántas entradas eso marcaría/desmarcaría y un disclaimer rojo si la selección incluye `Edge`, `Store`, `Photos` o `WinSecurity`.

## Stakeholders

- Subagente líder: `debloat-specialist`.
- Revisión obligatoria: `security-auditor` y `qa-tester` antes de cada release.
- Apoyo: `windows-systems-expert` para entradas de registro y servicios coordinados.

## Capabilities Tauri

- `default.json` para `list_bloatware_catalog` y `detect_installed_bloatware` (lectura).
- `elevated.json` para `remove_bloatware`.

## Comandos expuestos

```rust
// commands/debloat.rs

#[tauri::command]
pub async fn list_bloatware_catalog() -> Result<Vec<BloatwareEntry>, AppError>;

#[tauri::command]
pub async fn detect_installed_bloatware() -> Result<Vec<DetectedPackage>, AppError>;

#[tauri::command]
pub async fn remove_bloatware(
    input: RemoveBloatwareInput,
) -> Result<RemoveReport, AppError>;
```

### Modelos

```rust
// models/debloat.rs

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BloatwareEntry {
    pub id: String,
    pub display_name: String,
    pub category: BloatCategory,
    pub removal_strategy: RemovalStrategy,
    pub package_names: Vec<String>,    // patrones Appx
    pub registry_keys: Vec<RegistryKey>,
    pub services: Vec<String>,         // servicios a detener/deshabilitar
    pub scheduled_tasks: Vec<String>,
    pub risk: Risk,
    pub consequences: Vec<String>,
    pub reversal: Reversal,
    pub windows_builds: Vec<String>,   // ej. ">=22000"
    pub requires_elevation: bool,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum BloatCategory {
    ConsumerApp,    // Candy Crush, etc.
    Ai,             // Copilot
    MsConsumer,     // Solitaire, Mail, Maps, Xbox*
    MsCore,         // Edge, Store, OneDrive, Cortana, Teams system, Widgets, Photos
    Telemetry,      // DiagTrack & friends
    Oem,            // McAfee, HP Wolf, Lenovo Vantage, Dell SupportAssist
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RemovalStrategy {
    /// Remove-AppxPackage para el usuario actual.
    AppxCurrentUser,
    /// Remove-AppxPackage -AllUsers.
    AppxAllUsers,
    /// Remove-AppxProvisionedPackage para que no se reinstale en cuentas futuras.
    AppxProvisioned,
    /// Combinación CurrentUser + AllUsers + Provisioned (lo más completo).
    AppxFull,
    /// Uninstaller MSI/EXE conocido (Edge, Teams MSI, OneDrive setup /uninstall).
    DedicatedUninstaller(String),
    /// Solo aplicar policies de registro + detener servicios (no se puede desinstalar paquete real).
    PolicyOnly,
    /// Combinación: Appx full + uninstaller + policies.
    Hybrid,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Reversal {
    pub method: ReversalMethod, // StoreReinstall | OfficialInstaller | NotReversible | RestorePoint
    pub details: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DetectedPackage {
    pub entry_id: String,
    pub installed_for_user: bool,
    pub installed_all_users: bool,
    pub provisioned: bool,
    pub package_full_name: Option<String>,
    pub install_location: Option<String>,
    pub size_estimate_bytes: Option<u64>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RemoveBloatwareInput {
    pub entry_ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool, // default true cuando dry_run=false
    pub apply_policies: bool,       // si true, aplica registry_keys del catálogo
    pub disable_services: bool,     // si true, también deshabilita servicios listados
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RemoveReport {
    pub run_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub restore_point_id: Option<u32>,
    pub per_entry: Vec<PerEntryResult>,
    pub total_removed: u32,
    pub total_skipped: u32,
    pub total_failed: u32,
    pub bytes_freed_estimate: u64,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerEntryResult {
    pub entry_id: String,
    pub status: EntryStatus, // Removed | NotPresent | PartiallyRemoved | Failed | Skipped
    pub steps: Vec<StepLog>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StepLog {
    pub kind: StepKind, // AppxRemove | RegistryWrite | ServiceStop | UninstallerRun | TaskDisable
    pub target: String,
    pub ok: bool,
    pub stderr: Option<String>,
    pub duration_ms: u32,
}
```

## Eventos

| Evento | Payload | Cuándo |
|---|---|---|
| `debloat:detect-progress` | `{ scanned, total }` | Cada 100 ms durante `detect_installed_bloatware` |
| `debloat:remove-start` | `{ run_id, entries, dry_run }` | Al iniciar |
| `debloat:remove-step` | `{ run_id, entry_id, step }` | Tras cada `StepLog` |
| `debloat:remove-entry-done` | `PerEntryResult` | Al cerrar cada entrada |
| `debloat:remove-done` | `RemoveReport` | Al terminar |

## Motor híbrido Rust + PowerShell

### Por qué híbrido

- Appx está expuesto como cmdlets PS (`Get-AppxPackage`, `Remove-AppxPackage`, `Get-AppxProvisionedPackage`, `Remove-AppxProvisionedPackage`). El equivalente COM (`PackageManager` de WinRT) existe pero es verboso, frágil entre builds y no cubre `Provisioned` cómodamente.
- Registro y servicios → 100% Rust (`winreg` + `windows-service`).
- Uninstallers MSI/EXE dedicados → Rust spawneando el proceso, capturando stdout/stderr.
- Scheduled tasks → 100% Rust con `schtasks.exe` o `ITaskService` COM (preferido).

### Wrapper PowerShell

```rust
// services/powershell.rs

pub async fn run_ps_json<T: DeserializeOwned>(
    script: &str,
    args: &[(&str, &str)],
) -> Result<T, AppError> {
    // 1. Validar que script viene de un set conocido (allowlist por ID).
    // 2. Validar args: ningún valor puede contener `;`, `|`, `&`, backtick salvo si está en
    //    una lista blanca por arg específico (ej. el packageFullName puede tener guiones).
    // 3. Construir invocación: powershell.exe -NoProfile -NonInteractive
    //    -ExecutionPolicy Bypass -EncodedCommand <base64(UTF-16LE)>.
    // 4. Stdout obligatoriamente JSON. Si stderr no vacío, registrar como warning si exit=0
    //    o como AppError::Powershell si exit!=0.
    // 5. Timeout 60 s default; configurable por script.
}
```

Scripts embebidos en `src-tauri/src/services/ps_scripts/` y compilados al binario via `include_str!`. **Nunca** se generan dinámicamente strings PS desde input del usuario.

Scripts iniciales:

- `appx_list.ps1` — enumera todos los paquetes Appx para el usuario y AllUsers; emite JSON.
- `appx_provisioned.ps1` — enumera provisionados.
- `appx_remove.ps1` — recibe `-Pattern` y `-Scope` (CurrentUser/AllUsers/Provisioned); ejecuta y emite JSON con resultado por paquete.
- `task_disable.ps1` — disable scheduled tasks (fallback si no usamos COM).

Cada script termina con:

```powershell
$result | ConvertTo-Json -Depth 6 -Compress
```

## Flujo `detect_installed_bloatware`

1. Cargar catálogo.
2. `appx_list.ps1` → snapshot de Appx instalado.
3. `appx_provisioned.ps1` → snapshot provisionados.
4. Para cada entrada del catálogo:
   - Match de `package_names` (con wildcards estilo Appx) contra ambos snapshots.
   - Resolver `installed_for_user` / `installed_all_users` / `provisioned`.
   - Estimar tamaño con `(Get-AppxPackage).InstallLocation` + `du` rápido.
5. Devolver `DetectedPackage[]` ordenado por catálogo.

## Flujo `remove_bloatware`

```
1. Validar entry_ids ⊆ catálogo.
2. Si dry_run = false:
   a. Verificar is_elevated; si false, abortar con NotElevated.
   b. Crear restore point (si create_restore_point).
3. Por cada entry (secuencial):
   a. Emitir debloat:remove-step con kind=AppxRemove (o el primer paso).
   b. Ejecutar pasos según RemovalStrategy:
      - AppxCurrentUser  -> appx_remove.ps1 -Scope CurrentUser
      - AppxAllUsers     -> appx_remove.ps1 -Scope AllUsers
      - AppxProvisioned  -> appx_remove.ps1 -Scope Provisioned
      - AppxFull         -> los tres en orden
      - DedicatedUninstaller(cmd) -> spawn con args validados; capturar output.
      - PolicyOnly       -> escribir registry_keys + detener services.
      - Hybrid           -> Appx + Uninstaller + Policies + Tasks + Services.
   c. Si apply_policies: escribir registry_keys (atómico, batch).
   d. Si disable_services: detener y poner StartType=Disabled (delegado a service-manager).
   e. Para scheduled_tasks: deshabilitar y luego eliminar.
   f. Loguear cada StepLog.
4. Persistir RemoveReport en audit log.
```

### Cláusula "no fallar duro a mitad"

Si un paso falla, marcar `step.ok = false`, registrar stderr, **continuar con el siguiente paso de la misma entrada** (no abortar). El status final de la entrada se calcula:

- `Removed` → todos los pasos `ok = true`.
- `PartiallyRemoved` → mezcla.
- `Failed` → todos los pasos `ok = false` o el primer paso crítico falló.
- `NotPresent` → detección dijo que no estaba instalado.
- `Skipped` → usuario no la había marcado pero quedó en input por error.

## Casos especiales

### Edge

Edge moderno **no se desinstala** por Appx. Estrategia:

- Si `WindowsBuild < 22631`: usar `setup.exe --uninstall --system-level --verbose-logging --force-uninstall` desde `%PROGRAMFILES(X86)%\Microsoft\Edge\Application\<ver>\Installer\setup.exe`.
- Si build ≥ 22631 y la región no permite: **PolicyOnly** + ocultar de inicio + eliminar shortcuts. Avisar en consequences que "Edge no se elimina del todo en builds recientes; queda inerte".

Reversal: instalador oficial Microsoft Edge.

### Microsoft Store

- Catálogo lo marca como `risk = high`, `category = MsCore`.
- Estrategia `AppxFull` para `Microsoft.WindowsStore`.
- Aviso explícito: "Sin Store, reinstalar apps requerirá scripts manuales o WinGet."

### OneDrive

- Strategy `Hybrid`:
  - Stop & kill `OneDrive.exe`.
  - Ejecutar `%SYSTEMROOT%\\SysWOW64\\OneDriveSetup.exe /uninstall` (o `System32` en builds recientes).
  - Limpiar `%LOCALAPPDATA%\Microsoft\OneDrive`, `%PROGRAMDATA%\Microsoft OneDrive`.
  - Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\OneDrive!DisableFileSyncNGSC=1`.
  - Eliminar entradas del Explorer namespace (CLSID OneDrive).

### Cortana

- AppxFull `Microsoft.549981C3F5F10`.
- Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search!AllowCortana=0`.

### Teams (system-wide MSI Teams Machine-Wide Installer)

- DedicatedUninstaller via MSIEXEC `/x {GUID} /quiet /norestart`.
- También quitar Appx `MSTeams` (consumer) y `MicrosoftTeams`.

### Widgets / Copilot

- AppxFull + Policies (ver catálogo).

### Windows Security (Defender UI)

- **No se ofrece desinstalación**. Solo se permite ocultar del Start si el usuario insiste (PolicyOnly muy limitado). Riesgo `high`, advertencia explícita.

### OEM

- McAfee, HP Wolf Pro Security, Lenovo Vantage, Dell SupportAssist:
  - Detectar via `Get-Package` (no Appx) o `Get-WmiObject Win32_Product` (lento).
  - DedicatedUninstaller con MSI GUID por OEM. Si no hay GUID: caer a uninstaller del Add/Remove Programs (`Reg query` `Uninstall` y leer `UninstallString`).

## Coordinación con otros módulos

- **Restore points**: `remove_bloatware` solicita un punto al `restore_point` service. Si retorna `RestoreUnavailable`, preguntar al usuario si desea continuar sin punto (UI confirm).
- **Service manager**: detener/disable de servicios delegado.
- **Registry tweaks**: aplicación de `registry_keys` delegada al `registry` service con whitelist de hives y paths.
- **Audit log**: cada `RemoveReport` persistido como JSONL en `%APPDATA%\ClearTool\audit.jsonl`.

## Idempotencia

- `remove_bloatware` re-ejecutado con las mismas entradas: las ya removidas devuelven `NotPresent`; el run completo tarda < 2 s.
- Las policies se reescriben sin problema (idempotent writes).
- Los servicios deshabilitados que ya están deshabilitados no fallan.

## Errores

| Variante | Causa | UX |
|---|---|---|
| `NotElevated` | dry_run=false sin admin | banner "reiniciar como admin" |
| `Powershell(stderr)` | script PS falló | step.ok=false, continuar |
| `External(code)` | uninstaller MSI no-cero | step.ok=false, registrar exit code |
| `RestoreUnavailable` | System Restore caído | preguntar al user |
| `Permission` | entry_id fuera de catálogo | abortar run |

## Tests

- Unit: catalog loader rechaza entradas malformadas (schema-validation).
- Unit: Pattern matcher Appx con wildcards.
- Integration en VM: cada `entry` se aplica de cero, se verifica detección post-remoción, se restaura via punto y se vuelve a detectar.
- Golden test: para perfil "Total" en una VM 22H2 limpia, verificar que el set resultante de Appx instalado coincide con el snapshot esperado (commitable como JSON).
- Smoke E2E Playwright: abrir `/debloat`, click "Total", click "Aplicar", flujo de confirm, esperar reporte.

## UI (resumen, detalle en spec 03)

- Tabla virtualizada con checkbox por fila + selección masiva.
- Columnas: nombre, categoría (chip), riesgo (chip color-coded), strategy, instalado (badge `usuario`/`all-users`/`provisioned`), reversal.
- Filtros: por categoría, por riesgo, "solo instalados", "solo reversibles".
- Toolbar: presets `Mínimo`/`Recomendado`/`Total` + checkboxes globales `apply_policies`, `disable_services`.
- Modal `<ConfirmDestructive>` con tabs Resumen / Detalle (con consequences expandibles) / Riesgos / Reversa.
- Post-run: redirección a `/restore` mostrando el punto creado y el `audit run_id`.

## Futuro (no en MVP)

- WinGet pin/unpin para evitar reinstalación silenciosa por Microsoft Store.
- Detección de "rebloat" (paquetes que vuelven después de Feature Updates) → tarea recurrente.
- Marketplace comunitario de catálogos (firma criptográfica).
