---
name: powershell-debloat
description: Invocación segura de PowerShell desde Rust para operaciones de debloat (Get-AppxPackage, Remove-AppxPackage, Remove-AppxProvisionedPackage, Get-AppxProvisionedPackage). Cómo capturar salida estructurada con ConvertTo-Json, validar inputs contra inyección, y manejar exit codes. Usa el catálogo en RESOURCES/bloatware-catalog.json como fuente de verdad.
---

# Skill: powershell-debloat

Patrones para invocar PowerShell de forma segura desde Rust.

## Reglas que no se rompen

1. **Nunca concatenes input del usuario en el script.** Pásalo como argumentos posicionales y refiérete a `$args[0]`, `$args[1]`...
2. **Valida nombres Appx con regex** antes de pasarlos: `^[A-Za-z0-9._-]{1,128}$`.
3. **Siempre `-NoProfile -NonInteractive -ExecutionPolicy Bypass`** en la invocación.
4. **Captura stdout JSON** con `ConvertTo-Json -Compress -Depth 5`. Nunca parsees texto libre.
5. **Timeout obligatorio.** Cualquier invocación tiene cap de 60s para debloat individual, 5m para batch.

## Wrapper Rust de invocación

```rust
// services/powershell.rs
use serde::de::DeserializeOwned;
use std::process::Command;

pub struct PsResult<T> {
    pub data: T,
    pub stderr: String,
    pub exit_code: i32,
}

pub async fn run_typed<T: DeserializeOwned + Send + 'static>(
    script: &'static str,
    args: Vec<String>,
    timeout: std::time::Duration,
) -> Result<PsResult<T>, AppError> {
    validate_args(&args)?;
    let args_clone = args.clone();
    let script_owned = script.to_string();

    let join = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new("powershell.exe");
        cmd.arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-ExecutionPolicy").arg("Bypass")
            .arg("-Command").arg(&script_owned)
            .arg("-Args");
        for a in &args_clone {
            cmd.arg(a);
        }
        cmd.output()
    });

    let output = tokio::time::timeout(timeout, join)
        .await
        .map_err(|_| AppError::Powershell("timeout".into()))??
        .map_err(|e| AppError::Powershell(format!("io: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let data: T = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Powershell(format!("json: {e} | stdout: {stdout}")))?;

    Ok(PsResult { data, stderr, exit_code: output.status.code().unwrap_or(-1) })
}

fn validate_args(args: &[String]) -> Result<(), AppError> {
    static RE: once_cell::sync::Lazy<regex::Regex> =
        once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[A-Za-z0-9._\-: ]{1,256}$").unwrap());
    for a in args {
        if !RE.is_match(a) {
            return Err(AppError::Permission(format!("invalid arg: {a}")));
        }
    }
    Ok(())
}
```

## Scripts canónicos

### Listar Appx instalados (todos los usuarios)

```powershell
$ErrorActionPreference = 'Stop'
Get-AppxPackage -AllUsers |
  Select-Object Name, PackageFullName, Publisher, Version, NonRemovable, IsFramework |
  ConvertTo-Json -Compress -Depth 3
```

### Quitar Appx por usuario

```powershell
param([string]$Name)
$ErrorActionPreference = 'Stop'
$pkg = Get-AppxPackage -Name $Name
if (-not $pkg) {
  @{ status = 'already-absent'; name = $Name } | ConvertTo-Json -Compress
  exit 0
}
try {
  Remove-AppxPackage -Package $pkg.PackageFullName -ErrorAction Stop
  @{ status = 'removed'; name = $Name; package = $pkg.PackageFullName } | ConvertTo-Json -Compress
} catch {
  @{ status = 'failed'; name = $Name; error = $_.Exception.Message } | ConvertTo-Json -Compress
  exit 1
}
```

### Quitar Appx para todos los usuarios (admin)

```powershell
param([string]$Name)
$ErrorActionPreference = 'Stop'
$pkgs = Get-AppxPackage -AllUsers -Name $Name
if (-not $pkgs) {
  @{ status = 'already-absent'; name = $Name } | ConvertTo-Json -Compress
  exit 0
}
$results = foreach ($p in $pkgs) {
  try {
    Remove-AppxPackage -Package $p.PackageFullName -AllUsers -ErrorAction Stop
    @{ status = 'removed'; package = $p.PackageFullName }
  } catch {
    @{ status = 'failed'; package = $p.PackageFullName; error = $_.Exception.Message }
  }
}
$results | ConvertTo-Json -Compress -Depth 3
```

### Remover provisioned (admin) — bloquea reinstalación al crear nuevos perfiles

```powershell
param([string]$NamePattern)
$ErrorActionPreference = 'Stop'
$prov = Get-AppxProvisionedPackage -Online | Where-Object DisplayName -Like $NamePattern
if (-not $prov) {
  @{ status = 'already-absent'; name = $NamePattern } | ConvertTo-Json -Compress
  exit 0
}
$results = foreach ($p in $prov) {
  try {
    Remove-AppxProvisionedPackage -Online -PackageName $p.PackageName -ErrorAction Stop | Out-Null
    @{ status = 'removed'; package = $p.PackageName }
  } catch {
    @{ status = 'failed'; package = $p.PackageName; error = $_.Exception.Message }
  }
}
$results | ConvertTo-Json -Compress -Depth 3
```

## Manejo del catálogo

El catálogo canónico vive en `RESOURCES/bloatware-catalog.json`. Cada agente que toque debloat lo lee, no lo hardcodea. La schema se documenta en `.claude/agents/debloat-specialist.md`.

## Idempotencia

Cada invocación devuelve `status` ∈ `{removed, already-absent, failed, skipped}`. El frontend agrupa por status para reportar el batch.

## Testing

- Test contra paquetes "fake" creados con `New-AppxPackage` en VM aislada.
- Test de validación de args (rechaza `;`, `&`, `|`, `$`, `\``).
- Test de timeout (script con `Start-Sleep` largo).
