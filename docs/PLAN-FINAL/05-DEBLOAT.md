# 05 — Debloat (Appx + Uninstallers)

> **Posición:** 5/14. Módulo más complejo del set.
> **Dependencias:** [00-CATALOGOS](00-CATALOGOS.md), [01-RESTORE-POINTS](01-RESTORE-POINTS.md), [02-AUDIT-LOG](02-AUDIT-LOG.md).
> **Output:** `domain::debloat` completo (detección + remove), PowerShell embebido seguro, UI con presets + disclaimers + reverse via Store.

---

## 1. Resumen ejecutivo

Estado actual:

| Componente | Estado |
|---|---|
| `domain::debloat::list_catalog` | ✅ Carga del catálogo |
| `domain::debloat::detect_installed` | ❌ Stub |
| `domain::debloat::remove` | ❌ Stub |
| Catálogo `bloatware-catalog.json` | 🟡 Inicial — ampliación en [00-CATALOGOS §7](00-CATALOGOS.md#7-ampliación-del-bloatware-catalogjson) |
| UI `debloat-page.tsx` | 🟡 Lista contra catálogo, sin acción real |
| Scripts PowerShell embebidos | ❌ No existen |
| Disclaimers obligatorios (Edge/Store/OneDrive/Cortana) | ❌ No implementados |

Decisión sesión-2 del CLAUDE.md: **backend híbrido Rust + PowerShell**. PowerShell embebido vía `include_str!` para Appx; Rust para uninstaller registry strings; nunca construir PS dinámico desde input.

---

## 2. Diagnóstico

### 2.1 Tres mecanismos de remoción

| Método | Aplica a | Cómo |
|---|---|---|
| **Appx por usuario** | Apps Store instaladas en el user actual | `Get-AppxPackage` + `Remove-AppxPackage` |
| **Appx provisionado** | Apps preinstaladas que reaparecen en cuentas nuevas | `Get-AppxProvisionedPackage` + `Remove-AppxProvisionedPackage -Online` (admin) |
| **Uninstaller string** | Apps Win32 clásicas (OneDrive, Edge en sus límites, OEM) | Leer `HKLM\...\Uninstall\*\UninstallString`, lanzarlo con flags silent |

Para Edge (que es proceso protegido), el método es **deshabilitar via política**, no desinstalar. Marcar entries con `removalMethod: "service-and-files"` y disclaimers.

### 2.2 Reaparición tras update

Algunos Appx provisionados se reinstalan después de updates de feature (24H2 → 25H1). El cleanup completo requiere **remove provisional + remove user**. La UI los agrupa por `displayName` aunque sean dos entries.

### 2.3 Edge Chromium: caso especial

Edge no es Appx en Win11. Vive en `%PROGRAMFILES%\Microsoft\Edge\Application\`. **Desinstalarlo rompe componentes del SO** (WebView2, Outlook nuevo, etc.). Política recomendada:

- Marcar `risk: high`.
- Disclaimer obligatorio explícito.
- Por default NO incluir en preset `recommended`. Solo `total`.

### 2.4 Cortana

Cortana ya es Appx removible en Win11. Sin caveats especiales más allá del disclaimer "Voice search no funcionará".

### 2.5 Inyección por PowerShell

**Regla absoluta:** los scripts PS se embeben como `include_str!` (`&'static str`). Los argumentos dinámicos pasan por **regex validator** en Rust **antes** de tocar el script.

```rust
fn validate_pkg_family_name(s: &str) -> AppResult<()> {
    let re = regex::Regex::new(r"^[A-Za-z0-9_.-]+_[A-Za-z0-9]{13}$").unwrap();
    if !re.is_match(s) {
        return Err(AppError::Validation(format!("Appx package family name inválido: {}", s)));
    }
    Ok(())
}
```

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| PowerShell embebido `&'static str` con `include_str!` | Cero inyección, scripts auditables |
| Argumentos dinámicos validados con regex antes de invocar | Defense in depth |
| `Get-AppxPackage -AllUsers` para detección (con `Get-AppxPackage` plano si no admin) | Cobertura |
| `Remove-AppxProvisionedPackage` para limpieza profunda (admin) | Evita reaparición |
| Reverse recipe = open Store URL (no auto-reinstall) | Evitar instalar algo sin consentimiento explícito |
| Disclaimers blocking modal para Edge/OneDrive/Store/Cortana | UX legal y honesta |
| Catálogo separa categorías para presets independientes | `consumer-app`, `ai`, `telemetry`, etc. |
| Operaciones secuenciales (no paralelas) en el batch | PowerShell carga ~500ms cada vez; no ganamos paralelizando, sí complica logging |

---

## 4. Modelo de datos

```rust
// models/debloat.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareEntry {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub risk: String,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub removal_method: String,         // appx-user | appx-provisioned | winget | uninstaller-string | service-and-files
    pub appx_package_family_name: Option<String>,
    pub appx_provisioned_name: Option<String>,
    pub winget_id: Option<String>,
    pub uninstall_registry_path: Option<String>,
    #[serde(default = "yes")]
    pub preserves_data_by_default: bool,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub reversible: bool,
    pub reverse_recipe: Option<BloatwareReverse>,
    pub min_windows_build: Option<u32>,
    #[serde(default)]
    pub presets: Vec<String>,
}

fn yes() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareReverse {
    pub kind: String,                   // "appx-reinstall-from-store" | "manual-only"
    pub store_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPackage {
    pub id: String,                     // del catálogo
    pub display_name: String,
    pub installed_for_user: bool,
    pub installed_provisioned: bool,
    pub size_estimate_mb: Option<u32>,
    pub install_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveBloatwareInput {
    pub entry_ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub apply_policies: bool,           // bloquea reinstall (registry policies)
    pub disable_services: bool,         // detiene servicios asociados
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveReport {
    pub run_id: String,
    pub total: u32,
    pub removed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub restore_point_seq: Option<u32>,
    pub per_entry: Vec<PerEntryResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerEntryResult {
    pub id: String,
    pub status: String,                 // "removed" | "skipped" | "failed" | "dry-run" | "not-installed"
    pub method_used: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebloatProgressEvent {
    pub run_id: String,
    pub processed: u32,
    pub total: u32,
    pub current_id: String,
    pub current_display: String,
}
```

---

## 5. Plan UX

### 5.1 Pantalla principal

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Debloat                                                                      │
│ 78 apps en catálogo · 32 detectadas · 24 seleccionadas · ⚠ admin requerido   │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Buscar...]  [Categoría ▾]  [Riesgo ▾]                                       │
│                                                                              │
│ Presets:  [Mínimo (12)]  [Recomendado (48)]  [Total (78)]                    │
│                                                                              │
│ ☑ Limpiar también la versión provisionada (admin)                            │
│ ☑ Aplicar políticas para evitar reinstall                                    │
│ ☐ Detener servicios asociados                                                │
│                                                                              │
│ [Eliminar selección (24)]  [Dry-run]                                         │
│                                                                              │
│ Consumer apps                                                                │
│ ┌──────────────────────────────────────────────────────────────────────────┐│
│ │ ☑ Candy Crush          consumer-app  low   ● user   [Detalles]            ││
│ │ ☑ Spotify (preinstall) consumer-app  low   ● user   [Detalles]            ││
│ │ ☑ Instagram            consumer-app  low   ● user   [Detalles]            ││
│ └──────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│ Microsoft consumer                                                           │
│ ┌──────────────────────────────────────────────────────────────────────────┐│
│ │ ☑ Microsoft Copilot    ai            low   ● user   [Detalles]            ││
│ │ ☐ Microsoft Edge       edge-component HIGH ● win32  [Detalles] ⚠          ││
│ │ ☑ Bing News            ms-consumer   low   ●● both  [Detalles]            ││
│ └──────────────────────────────────────────────────────────────────────────┘│
│ ...                                                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

Leyenda:
- `● user` — solo instalado para el usuario.
- `●● both` — instalado + provisionado (al limpiar reinstalación al crear cuenta nueva).
- `● win32` — método uninstaller string.

### 5.2 Modal de disclaimer (Edge ejemplo)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ⚠  Microsoft Edge                                                        │
├─────────────────────────────────────────────────────────────────────────┤
│  Edge NO se desinstala completamente porque es componente del SO.        │
│  ClearTool lo hará no-invasivo siguiendo este patrón:                    │
│                                                                          │
│  1. Aplicar política `DisableEdgeDesktopShortcutCreation = 1` (HKLM).   │
│  2. Aplicar política `HubsSidebarEnabled = 0` (HKCU).                   │
│  3. Borrar shortcuts existentes en escritorio y Start.                  │
│  4. NO se borra el binario en `%PROGRAMFILES%\Microsoft\Edge\`.         │
│                                                                          │
│  Componentes del SO que SIGUEN funcionando:                              │
│   • WebView2 (Outlook, Teams, etc.)                                      │
│   • Edge-update process                                                  │
│   • Cualquier app .NET que use Edge embebido                             │
│                                                                          │
│  Para volver atrás:                                                      │
│   1. Revertir desde el log de auditoría.                                 │
│   2. O ejecutar el restore point creado antes.                           │
│                                                                          │
│  ☑ Entiendo las consecuencias                                            │
│                                                                          │
│  [Cancelar]                            [Eliminar Edge (limitado)]        │
└─────────────────────────────────────────────────────────────────────────┘
```

Análogos para OneDrive, Store (mucho más drástico), Cortana.

### 5.3 Modal pre-batch

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Eliminar 24 apps                                                        │
├─────────────────────────────────────────────────────────────────────────┤
│  Métodos a usar:                                                         │
│   • Appx por usuario:        18 apps                                     │
│   • Appx provisionado:       18 apps (limpieza profunda)                 │
│   • Uninstaller string:       6 apps                                     │
│                                                                          │
│  ☑ Crear punto de restauración                                           │
│  ☑ Aplicar políticas anti-reinstall                                      │
│  ☐ Detener servicios asociados                                           │
│  ☐ Dry-run                                                               │
│                                                                          │
│  ⚠  3 apps con HIGH RISK requieren disclaimer:                            │
│      Edge, OneDrive, Store                                               │
│                                                                          │
│  Estimación: ~2 minutos (PowerShell tarda ~500ms por app).               │
│                                                                          │
│  [Cancelar]                                          [Eliminar →]        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Plan de implementación por fases

### Fase 1 — Scripts PowerShell embebidos

**Archivo:** `src-tauri/src/platform/ps-scripts/list-appx.ps1` (nuevo)

```powershell
# list-appx.ps1 — listado de Appx instalados (user + provisional)
$ErrorActionPreference = "Stop"

$user = @(Get-AppxPackage -ErrorAction SilentlyContinue | Select-Object Name, PackageFamilyName, InstallLocation, Version)
$provisional = @()
try {
    $provisional = @(Get-AppxProvisionedPackage -Online -ErrorAction Stop | Select-Object DisplayName, PackageName)
} catch {
    # No admin — provisional queda vacío. Es OK.
}

@{
    user        = $user
    provisional = $provisional
} | ConvertTo-Json -Compress -Depth 4
```

**Archivo:** `src-tauri/src/platform/ps-scripts/remove-appx-user.ps1`

```powershell
# remove-appx-user.ps1 — quita un Appx del user actual.
# Argumento: $env:CT_APPX_PFN (PackageFamilyName, validado en Rust).
$ErrorActionPreference = "Stop"
$pfn = $env:CT_APPX_PFN
if ([string]::IsNullOrWhiteSpace($pfn)) {
    @{ ok = $false; error = "missing CT_APPX_PFN" } | ConvertTo-Json -Compress
    exit 2
}
try {
    $pkg = Get-AppxPackage -ErrorAction Stop | Where-Object { $_.PackageFamilyName -eq $pfn }
    if ($null -eq $pkg) {
        @{ ok = $true; status = "not-installed" } | ConvertTo-Json -Compress
        exit 0
    }
    Remove-AppxPackage -Package $pkg.PackageFullName -ErrorAction Stop
    @{ ok = $true; status = "removed" } | ConvertTo-Json -Compress
} catch {
    @{ ok = $false; error = $_.Exception.Message } | ConvertTo-Json -Compress
    exit 1
}
```

**Archivo:** `src-tauri/src/platform/ps-scripts/remove-appx-provisioned.ps1`

```powershell
# remove-appx-provisioned.ps1 — quita un Appx provisionado (requiere admin).
# Argumento: $env:CT_APPX_PROV_NAME (DisplayName del provisional).
$ErrorActionPreference = "Stop"
$name = $env:CT_APPX_PROV_NAME
if ([string]::IsNullOrWhiteSpace($name)) {
    @{ ok = $false; error = "missing CT_APPX_PROV_NAME" } | ConvertTo-Json -Compress
    exit 2
}
try {
    $prov = Get-AppxProvisionedPackage -Online | Where-Object { $_.DisplayName -eq $name }
    if ($null -eq $prov) {
        @{ ok = $true; status = "not-provisioned" } | ConvertTo-Json -Compress
        exit 0
    }
    Remove-AppxProvisionedPackage -Online -PackageName $prov.PackageName | Out-Null
    @{ ok = $true; status = "removed" } | ConvertTo-Json -Compress
} catch {
    @{ ok = $false; error = $_.Exception.Message } | ConvertTo-Json -Compress
    exit 1
}
```

### Fase 2 — `platform::debloat` (wrapper Rust)

**Archivo:** `src-tauri/src/platform/debloat.rs` (nuevo)

```rust
use crate::core::{AppError, AppResult};
use crate::platform::powershell;
use std::process::Command;

const SCRIPT_LIST: &str = include_str!("ps-scripts/list-appx.ps1");
const SCRIPT_REMOVE_USER: &str = include_str!("ps-scripts/remove-appx-user.ps1");
const SCRIPT_REMOVE_PROV: &str = include_str!("ps-scripts/remove-appx-provisioned.ps1");

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxUserRow {
    name: String,
    #[serde(rename = "PackageFamilyName")]
    package_family_name: String,
    install_location: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxProvRow {
    display_name: String,
    package_name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AppxListOutput {
    user: Vec<AppxUserRow>,
    provisional: Vec<AppxProvRow>,
}

pub fn list_appx() -> AppResult<(Vec<AppxUser>, Vec<AppxProv>)> {
    let out = powershell::run_script(SCRIPT_LIST)?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: AppxListOutput = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("list-appx: {}", e)))?;
    let users = parsed.user.into_iter().map(|r| AppxUser {
        name: r.name,
        package_family_name: r.package_family_name,
        install_location: r.install_location,
        version: r.version,
    }).collect();
    let provs = parsed.provisional.into_iter().map(|r| AppxProv {
        display_name: r.display_name,
        package_name: r.package_name,
    }).collect();
    Ok((users, provs))
}

pub struct AppxUser {
    pub name: String,
    pub package_family_name: String,
    pub install_location: Option<String>,
    pub version: Option<String>,
}

pub struct AppxProv {
    pub display_name: String,
    pub package_name: String,
}

pub fn remove_appx_user(package_family_name: &str) -> AppResult<String> {
    validate_pkg_family_name(package_family_name)?;
    let out = run_with_env(SCRIPT_REMOVE_USER, &[("CT_APPX_PFN", package_family_name)])?;
    extract_status(&out)
}

pub fn remove_appx_provisioned(provisioned_name: &str) -> AppResult<String> {
    validate_prov_name(provisioned_name)?;
    let out = run_with_env(SCRIPT_REMOVE_PROV, &[("CT_APPX_PROV_NAME", provisioned_name)])?;
    extract_status(&out)
}

fn validate_pkg_family_name(s: &str) -> AppResult<()> {
    let re = regex::Regex::new(r"^[A-Za-z0-9_.-]+_[A-Za-z0-9]{13}$").unwrap();
    if !re.is_match(s) {
        return Err(AppError::Validation(format!("PackageFamilyName inválido: {}", s)));
    }
    Ok(())
}

fn validate_prov_name(s: &str) -> AppResult<()> {
    let re = regex::Regex::new(r"^[A-Za-z0-9_.]+$").unwrap();
    if !re.is_match(s) {
        return Err(AppError::Validation(format!("ProvisionedName inválido: {}", s)));
    }
    Ok(())
}

fn run_with_env(script: &'static str, vars: &[(&str, &str)]) -> AppResult<std::process::Output> {
    // Copia el helper de powershell::run_script pero añade env vars.
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile", "-NonInteractive",
        "-ExecutionPolicy", "Bypass",
        "-Command", script,
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    for (k, v) in vars {
        cmd.env(k, v);
    }
    cmd.output().map_err(|e| AppError::Io(e.to_string()))
}

#[derive(serde::Deserialize)]
struct PsResult {
    ok: bool,
    status: Option<String>,
    error: Option<String>,
}

fn extract_status(out: &std::process::Output) -> AppResult<String> {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: PsResult = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Parse(format!("ps result: {}: {}", e, stdout.trim())))?;
    if !parsed.ok {
        return Err(AppError::PowerShell(parsed.error.unwrap_or_else(|| "unknown".into())));
    }
    Ok(parsed.status.unwrap_or_else(|| "ok".into()))
}
```

### Fase 3 — Uninstaller string (Win32 apps)

**Archivo:** `src-tauri/src/platform/uninstaller.rs` (nuevo)

```rust
use crate::core::{AppError, AppResult};
use crate::platform::registry as preg;
use std::process::Command;

const UNINSTALL_HIVES: &[(&str, &str)] = &[
    ("HKLM", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ("HKLM", r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
    ("HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
];

/// Busca un programa por DisplayName aproximado en los 3 hives de Uninstall.
/// Devuelve la `UninstallString` si encuentra match.
pub fn find_uninstall_string(display_name_pattern: &str) -> AppResult<Option<String>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let pattern = display_name_pattern.to_lowercase();

    for (hive, key) in UNINSTALL_HIVES {
        let root = match *hive {
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            _ => continue,
        };
        let subkey = match root.open_subkey(key) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub_name in subkey.enum_keys().filter_map(Result::ok) {
            if let Ok(item) = subkey.open_subkey(&sub_name) {
                let name: Result<String, _> = item.get_value("DisplayName");
                let unstr: Result<String, _> = item.get_value("UninstallString");
                if let (Ok(n), Ok(u)) = (name, unstr) {
                    if n.to_lowercase().contains(&pattern) {
                        return Ok(Some(u));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Ejecuta una `UninstallString` con flag silent.
/// Solo se invoca con strings provenientes del registry — no input de usuario.
pub fn run_uninstaller(uninstall_string: &str) -> AppResult<()> {
    // Detectar si es msiexec o exe directo.
    let lower = uninstall_string.to_lowercase();
    let (program, args) = if lower.contains("msiexec.exe") {
        // msiexec.exe /I{GUID} → cambiar /I por /X y añadir /quiet /norestart
        let s = uninstall_string.replace("/I", "/X").replace("/i", "/X");
        ("cmd".to_string(), vec!["/C".to_string(), format!("{} /quiet /norestart", s)])
    } else {
        // Un exe directo. Añadimos silent flags comunes.
        ("cmd".to_string(), vec!["/C".to_string(), format!("{} /S /silent /quiet", uninstall_string)])
    };

    let status = Command::new(&program)
        .args(&args)
        .status()
        .map_err(|e| AppError::Io(format!("spawn uninstaller: {}", e)))?;
    if !status.success() {
        return Err(AppError::Services(format!("uninstaller exit: {:?}", status.code())));
    }
    Ok(())
}
```

### Fase 4 — `domain::debloat`

#### Paso 4.1 — `detect_installed`

**Archivo:** `src-tauri/src/domain/debloat.rs`

```rust
use crate::core::{AppError, AppResult};
use crate::domain::{audit, catalog};
use crate::models::debloat::{
    BloatwareEntry, DetectedPackage, RemoveBloatwareInput, RemoveReport,
    PerEntryResult, DebloatProgressEvent,
};
use crate::models::restore::ReverseRecipe;
use crate::platform;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn list_catalog() -> AppResult<Vec<BloatwareEntry>> {
    catalog::load_bloatware_catalog()
}

pub fn detect_installed() -> AppResult<Vec<DetectedPackage>> {
    let cat = catalog::load_bloatware_catalog()?;
    let (users, provs) = platform::debloat::list_appx()?;

    let users_set: HashSet<String> = users.iter().map(|u| u.package_family_name.clone()).collect();
    let users_loc: HashMap<String, Option<String>> = users.iter()
        .map(|u| (u.package_family_name.clone(), u.install_location.clone()))
        .collect();
    let provs_set: HashSet<String> = provs.iter().map(|p| p.display_name.clone()).collect();

    let mut out = Vec::new();
    for entry in cat {
        let pfn = entry.appx_package_family_name.clone().unwrap_or_default();
        let prov = entry.appx_provisioned_name.clone().unwrap_or_default();
        let in_user = !pfn.is_empty() && users_set.contains(&pfn);
        let in_prov = !prov.is_empty() && provs_set.contains(&prov);
        if !in_user && !in_prov && entry.removal_method != "uninstaller-string" {
            continue;
        }
        out.push(DetectedPackage {
            id: entry.id.clone(),
            display_name: entry.display_name.clone(),
            installed_for_user: in_user,
            installed_provisioned: in_prov,
            size_estimate_mb: None,
            install_location: users_loc.get(&pfn).cloned().flatten(),
        });
    }
    Ok(out)
}
```

#### Paso 4.2 — `remove` con audit

```rust
pub fn remove<F>(
    input: &RemoveBloatwareInput,
    mut emit_progress: F,
) -> AppResult<RemoveReport>
where
    F: FnMut(u32, u32, &str, &str),
{
    let cat = catalog::load_bloatware_catalog()?;
    let cat_by_id: HashMap<String, BloatwareEntry> = cat.into_iter().map(|e| (e.id.clone(), e)).collect();

    let total = input.entry_ids.len() as u32;

    let restore_seq = if input.create_restore_point && !input.dry_run {
        platform::restore_point::create(
            &format!("ClearTool — debloat {} apps", input.entry_ids.len()),
            0,  // APPLICATION_INSTALL
            true,
        ).ok()
    } else { None };

    let mut removed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut per: Vec<PerEntryResult> = Vec::with_capacity(input.entry_ids.len());

    for (i, id) in input.entry_ids.iter().enumerate() {
        let entry = match cat_by_id.get(id) {
            Some(e) => e,
            None => {
                skipped += 1;
                per.push(PerEntryResult {
                    id: id.clone(),
                    status: "skipped".into(),
                    method_used: None,
                    error: Some("not-in-catalog".into()),
                });
                continue;
            }
        };
        emit_progress((i + 1) as u32, total, id, &entry.display_name);

        if input.dry_run {
            per.push(PerEntryResult {
                id: id.clone(),
                status: "dry-run".into(),
                method_used: Some(entry.removal_method.clone()),
                error: None,
            });
            removed += 1;
            continue;
        }

        let result = remove_single(entry);

        // Audit individual (granular: una entry por app eliminada)
        let recipe = make_reverse_recipe(entry);
        let audit_entry = audit::make_entry(
            "debloat",
            "remove",
            false,
            restore_seq,
            vec![id.clone()],
            recipe,
            if result.is_ok() { "success" } else { "failed" },
            result.as_ref().err().map(|e| format!("{}", e)),
        );
        let _ = audit::write_entry(&audit_entry);

        match result {
            Ok(method) => {
                removed += 1;
                per.push(PerEntryResult {
                    id: id.clone(),
                    status: "removed".into(),
                    method_used: Some(method),
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                per.push(PerEntryResult {
                    id: id.clone(),
                    status: "failed".into(),
                    method_used: Some(entry.removal_method.clone()),
                    error: Some(format!("{}", e)),
                });
            }
        }
    }

    Ok(RemoveReport {
        run_id: Uuid::new_v4().to_string(),
        total, removed, failed, skipped,
        restore_point_seq: restore_seq,
        per_entry: per,
    })
}

fn remove_single(entry: &BloatwareEntry) -> AppResult<String> {
    match entry.removal_method.as_str() {
        "appx-user" => {
            let pfn = entry.appx_package_family_name.clone()
                .ok_or_else(|| AppError::Validation(format!("entry {} sin pfn", entry.id)))?;
            let st = platform::debloat::remove_appx_user(&pfn)?;
            Ok(format!("appx-user:{}", st))
        }
        "appx-provisioned" => {
            let name = entry.appx_provisioned_name.clone()
                .ok_or_else(|| AppError::Validation(format!("entry {} sin prov name", entry.id)))?;
            let st = platform::debloat::remove_appx_provisioned(&name)?;
            Ok(format!("appx-prov:{}", st))
        }
        "uninstaller-string" => {
            let pat = entry.display_name.clone();
            let s = platform::uninstaller::find_uninstall_string(&pat)?
                .ok_or_else(|| AppError::Services(format!("no uninstall string para: {}", pat)))?;
            platform::uninstaller::run_uninstaller(&s)?;
            Ok("uninstaller".into())
        }
        "service-and-files" => {
            // Edge-like: aplicar políticas + borrar shortcuts.
            // Implementación específica por entry — para v1.0 solo Edge.
            handle_edge_special(entry)
        }
        other => Err(AppError::Validation(format!("removal_method desconocido: {}", other))),
    }
}

fn handle_edge_special(_entry: &BloatwareEntry) -> AppResult<String> {
    // 1. Aplicar políticas (vía registry::write_value directo).
    platform::registry::write_value("HKLM", r"SOFTWARE\Policies\Microsoft\EdgeUpdate",
        "DisableEdgeDesktopShortcutCreation", "dword", &serde_json::json!(1), true)?;
    platform::registry::write_value("HKCU", r"Software\Policies\Microsoft\Edge",
        "HubsSidebarEnabled", "dword", &serde_json::json!(0), true)?;
    // 2. Borrar shortcuts conocidos (Desktop, Start).
    let candidates = [
        format!(r"{}\Microsoft Edge.lnk", std::env::var("PUBLIC").unwrap_or_default() + r"\Desktop"),
        format!(r"{}\Microsoft Edge.lnk", std::env::var("USERPROFILE").unwrap_or_default() + r"\Desktop"),
    ];
    for path in candidates {
        let _ = std::fs::remove_file(&path);
    }
    Ok("policies-only".into())
}

fn make_reverse_recipe(entry: &BloatwareEntry) -> ReverseRecipe {
    match (&entry.removal_method[..], &entry.appx_package_family_name, &entry.reverse_recipe) {
        ("appx-user", Some(pfn), Some(r)) => ReverseRecipe::AppxReinstall {
            package_family_name: pfn.clone(),
            store_url: r.store_url.clone(),
        },
        ("appx-provisioned", Some(pfn), Some(r)) => ReverseRecipe::AppxReinstall {
            package_family_name: pfn.clone(),
            store_url: r.store_url.clone(),
        },
        _ => ReverseRecipe::Noop {
            reason: format!("método {} no reversible automáticamente", entry.removal_method),
        },
    }
}
```

### Fase 5 — IPC

**Archivo:** `src-tauri/src/ipc/debloat.rs`

```rust
use crate::core::AppResult;
use crate::domain;
use crate::models::debloat::{
    BloatwareEntry, DetectedPackage, RemoveBloatwareInput, RemoveReport, DebloatProgressEvent,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn list_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    domain::debloat::list_catalog()
}

#[tauri::command]
pub async fn detect_installed_bloatware() -> AppResult<Vec<DetectedPackage>> {
    domain::debloat::detect_installed()
}

#[tauri::command]
pub async fn remove_bloatware(
    app: AppHandle,
    input: RemoveBloatwareInput,
) -> AppResult<RemoveReport> {
    domain::debloat::remove(&input, move |processed, total, current_id, current_display| {
        let _ = app.emit("debloat:progress", DebloatProgressEvent {
            run_id: String::new(),
            processed, total,
            current_id: current_id.to_string(),
            current_display: current_display.to_string(),
        });
    })
}
```

### Fase 6 — Frontend

#### Paso 6.1 — API client

```ts
export const listBloatwareCatalog = () => invoke<BloatwareEntry[]>("list_bloatware_catalog");
export const detectInstalledBloatware = () => invoke<DetectedPackage[]>("detect_installed_bloatware");
export const removeBloatware = (input: RemoveBloatwareInput) =>
  invoke<RemoveReport>("remove_bloatware", { input });
```

#### Paso 6.2 — Componentes a crear

```
src/features/debloat/
  debloat-page.tsx               ← reescribir
  use-debloat.ts                 ← nuevo
  components/
    bloatware-row.tsx
    bloatware-detail-modal.tsx
    high-risk-disclaimer.tsx     ← §5.2
    batch-confirm-modal.tsx      ← §5.3
    progress-modal.tsx
    preset-selector.tsx
    method-badge.tsx
```

#### Paso 6.3 — Disclaimer modal obligatorio

```tsx
// components/high-risk-disclaimer.tsx
import { useState } from "react";
import type { BloatwareEntry } from "../../../api";

const HIGH_RISK_IDS = ["edge", "edge-pwa-installs", "onedrive", "store", "cortana"];

export function HighRiskDisclaimer(props: {
  entries: BloatwareEntry[];
  onAccept: () => void;
  onCancel: () => void;
}) {
  const highRisk = props.entries.filter((e) => HIGH_RISK_IDS.includes(e.id));
  const [accepted, setAccepted] = useState<Set<string>>(new Set());

  if (highRisk.length === 0) {
    return null;
  }

  const allAccepted = highRisk.every((e) => accepted.has(e.id));

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-card border border-destructive rounded-lg p-6 max-w-2xl">
        <h3 className="text-xl font-bold mb-2">⚠ Apps de alto riesgo seleccionadas</h3>
        <p className="text-sm text-muted-foreground mb-4">
          Las siguientes apps tienen consecuencias importantes. Debés aceptar cada una antes de continuar.
        </p>
        <div className="space-y-3 max-h-96 overflow-auto">
          {highRisk.map((e) => (
            <div key={e.id} className="border border-border rounded p-3">
              <div className="font-bold">{e.displayName}</div>
              <ul className="text-xs text-muted-foreground list-disc pl-5 mt-1">
                {e.consequences.map((c, i) => <li key={i}>{c}</li>)}
              </ul>
              <label className="flex items-center gap-2 mt-2 text-sm">
                <input
                  type="checkbox"
                  checked={accepted.has(e.id)}
                  onChange={(ev) => {
                    setAccepted((prev) => {
                      const next = new Set(prev);
                      ev.target.checked ? next.add(e.id) : next.delete(e.id);
                      return next;
                    });
                  }}
                />
                Entiendo y acepto eliminar {e.displayName}
              </label>
            </div>
          ))}
        </div>
        <div className="flex gap-2 justify-end mt-4">
          <button className="px-3 py-1 border" onClick={props.onCancel}>Cancelar</button>
          <button
            className="px-3 py-1 bg-destructive text-white disabled:opacity-50"
            disabled={!allAccepted}
            onClick={props.onAccept}
          >
            Continuar
          </button>
        </div>
      </div>
    </div>
  );
}
```

---

## 7. Tests

### 7.1 Test validators regex

```rust
#[test]
fn pkg_family_name_validador() {
    use cleartool::platform::debloat::*;
    // En el módulo, expone validate_pkg_family_name pub(crate) para test.
    assert!(validate_pkg_family_name("Microsoft.Copilot_8wekyb3d8bbwe").is_ok());
    assert!(validate_pkg_family_name("evil; Remove-Item C:\\").is_err());
    assert!(validate_pkg_family_name("").is_err());
}
```

### 7.2 Test integration en VM

1. VM Win11 limpia + admin + ya logueado en cuenta Microsoft.
2. `/debloat` → `Detectar`.
3. **Esperado:** lista incluye Cortana, Bing News, Bing Weather (apps por default).
4. Marcar Bing News + Bing Weather + dry-run.
5. Aceptar → ningún cambio aplicado.
6. Desmarcar dry-run, aceptar.
7. **Esperado:** Restore point creado. Apps eliminadas.
8. `Get-AppxPackage -Name *bing*` en PowerShell debe estar vacío.
9. Audit log debe tener 2 entries.
10. Revertir Bing News desde audit → abre Store URL.

### 7.3 Test handle_edge_special

```rust
#[test]
#[ignore]
fn edge_aplica_politicas() {
    let entry = cleartool::models::debloat::BloatwareEntry {
        id: "edge".into(),
        display_name: "Microsoft Edge".into(),
        category: "edge-component".into(),
        risk: "high".into(),
        removal_method: "service-and-files".into(),
        appx_package_family_name: None,
        appx_provisioned_name: None,
        winget_id: None, uninstall_registry_path: None,
        preserves_data_by_default: true, requires_admin: true,
        reversible: false, reverse_recipe: None,
        consequences: vec![], description: None, min_windows_build: None,
        presets: vec![],
    };
    cleartool::domain::debloat::handle_edge_special(&entry).expect("ok");
    // verificar que la key HKLM\...\EdgeUpdate\DisableEdgeDesktopShortcutCreation = 1
}
```

---

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Inyección via PackageFamilyName malicioso | Regex validator en Rust antes de invocar PS |
| Provisional removal sin admin | Falla con error claro, UI deshabilita el toggle |
| Uninstaller MSI/exe lento (>30s) | Async + progress event + timeout configurable |
| Borrar Edge accidentalmente | Disclaimer obligatorio + checkbox individual |
| Apps reaparecen tras feature update | Usar `apply_policies: true` (default) |
| Algunos Appx no se pueden eliminar (system-critical) | PS reporta error, audit entry `failed` |
| Cortana cambia de package family name entre builds | Mantener catálogo actualizado, fallback a búsqueda parcial |

---

## 9. Definition of Done

- [ ] 3 scripts PowerShell embebidos (`list-appx`, `remove-appx-user`, `remove-appx-provisioned`).
- [ ] `platform::debloat::{list_appx, remove_appx_user, remove_appx_provisioned}` con regex validators.
- [ ] `platform::uninstaller::{find_uninstall_string, run_uninstaller}`.
- [ ] `domain::debloat::list_catalog`, `detect_installed`, `remove` completos.
- [ ] Audit individual por app eliminada (no batch-level).
- [ ] Reverse recipe Store URL para Appx, Noop para uninstaller string.
- [ ] IPC: 3 comandos respondiendo.
- [ ] UI: lista categorizada, presets, modal detalle, **disclaimer modal obligatorio** para HIGH risk.
- [ ] Progress modal durante batch.
- [ ] Test regex validator pasa.
- [ ] Test manual VM: 2 apps eliminadas + audit + revert via Store funciona.
- [ ] Commit `feat(debloat): backend Appx + uninstaller + UI con disclaimers`.

---

## 10. Próximo archivo

→ [06-CACHE-FINAL.md](06-CACHE-FINAL.md) — cierre del único módulo destructivo que ya estaba funcional. Faltan: presets, filtros avanzados, integración con audit, mejoras UX.
