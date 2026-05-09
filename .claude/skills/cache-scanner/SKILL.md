---
name: cache-scanner
description: Catálogo curado de ubicaciones de caché conocidas en Windows 11 (Update, DISM, CBS, Prefetch, thumbcache, INetCache, Edge, Appx por usuario, Font cache, WER, etc.) y patrón Rust para escaneo seguro con cálculo de tamaño total, conteo de archivos y filtros de archivos en uso. Usa el JSON en RESOURCES/cache-locations.json como fuente de verdad.
---

# Skill: cache-scanner

## Cuándo usar

- Implementación del módulo `cache-cleaner`.
- Cuando otro agente pregunta qué carpetas son seguras de limpiar.

## Catálogo

Fuente de verdad: `RESOURCES/cache-locations.json`. Cada entry:

```jsonc
{
  "id": "windows-update.softwaredistribution",
  "displayName": "Windows Update — caché de descargas",
  "path": "%SYSTEMROOT%\\SoftwareDistribution\\Download",
  "category": "windows-update",   // "user-temp" | "system-temp" | "windows-update" | "cbs-dism" | "browser-cache" | "appx-cache" | "logs" | "thumbnail" | "font" | "wer"
  "requiresAdmin": true,
  "risk": "low",                  // "low" | "medium" | "high"
  "preconditions": [
    "Detener servicio wuauserv antes de borrar."
  ],
  "filters": {
    "olderThanDays": 7,           // opcional
    "exclude": ["*.lock"]         // opcional, glob
  },
  "consequences": ["La próxima sesión de Windows Update re-descargará lo necesario."],
  "averageSize": "100MB-5GB"
}
```

## Patrón Rust de escaneo

```rust
// services/filesystem.rs
use std::path::Path;

pub struct CacheScanReport {
    pub location_id: String,
    pub resolved_path: PathBuf,
    pub exists: bool,
    pub total_bytes: u64,
    pub file_count: u64,
    pub locked_files: u64,
    pub sample_files: Vec<PathBuf>,
}

pub async fn scan_location(loc: &CacheLocation) -> Result<CacheScanReport, AppError> {
    let resolved = expand_env(&loc.path);
    let canonical = match dunce::canonicalize(&resolved) {
        Ok(p) => p,
        Err(_) => return Ok(CacheScanReport::not_found(loc, resolved)),
    };
    validate_within_safe_roots(&canonical)?;

    tokio::task::spawn_blocking(move || -> Result<_, AppError> {
        let mut total = 0u64;
        let mut count = 0u64;
        let mut locked = 0u64;
        let mut samples = Vec::new();

        for entry in walkdir::WalkDir::new(&canonical).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            match entry.metadata() {
                Ok(m) => {
                    total += m.len();
                    count += 1;
                    if samples.len() < 20 { samples.push(entry.path().to_path_buf()); }
                }
                Err(_) => locked += 1,
            }
        }

        Ok(CacheScanReport { /* ... */ })
    }).await?
}
```

## Validación de paths

```rust
const SAFE_ROOTS: &[&str] = &[
    r"C:\Windows\Temp",
    r"C:\Windows\SoftwareDistribution",
    r"C:\Windows\Logs",
    r"C:\Windows\Prefetch",
    r"C:\Users",                        // gating extra: solo subcarpetas AppData/Local/...
    r"C:\ProgramData",                  // idem
];

pub fn validate_within_safe_roots(p: &Path) -> Result<(), AppError> {
    let s = p.to_string_lossy().to_ascii_lowercase();
    if !SAFE_ROOTS.iter().any(|r| s.starts_with(&r.to_ascii_lowercase())) {
        return Err(AppError::Permission(format!("path outside safe roots: {}", p.display())));
    }
    // Y bloqueamos system32, syswow64, system, drivers, etc.
    const BLOCKED_SUBSTRINGS: &[&str] = &[
        r"\system32\config",
        r"\system32\drivers",
        r"\winsxs",
    ];
    if BLOCKED_SUBSTRINGS.iter().any(|b| s.contains(b)) {
        return Err(AppError::Permission(format!("blocked path: {}", p.display())));
    }
    Ok(())
}
```

## Borrado seguro

Antes de borrar, el orden estricto:

1. **Pre-condiciones del catálogo** (detener servicios necesarios).
2. **Crear restore point** vía `restore-point-manager`.
3. **Detectar archivos en uso** (saltar y reportar como `locked`).
4. **Borrar respetando filtros** (`olderThanDays`, `exclude` glob).
5. **Restaurar pre-condiciones** (re-iniciar servicios).
6. **Loguear evento** en JSONL.

```rust
pub async fn clean_location(
    loc: &CacheLocation,
    dry_run: bool,
) -> Result<CleanReport, AppError> {
    // ...
}
```

## Idempotencia

Si la carpeta no existe o ya está vacía, status `already-clean`. Si algunos archivos quedaron locked, status `partial` con el detalle.

## Lista mínima de inicio

El JSON `cache-locations.json` arranca con estas:

- `%TEMP%`
- `%LOCALAPPDATA%\Temp`
- `C:\Windows\Temp`
- `C:\Windows\SoftwareDistribution\Download`
- `C:\Windows\Logs\CBS`
- `C:\Windows\Logs\DISM`
- `C:\Windows\Prefetch` (marcado `risk: medium` — limpia caché pero impacta arranque corto plazo)
- `%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db`
- `%LOCALAPPDATA%\Microsoft\Windows\WER\ReportArchive`
- `%LOCALAPPDATA%\Microsoft\Windows\INetCache`
- `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Cache`

Para añadir entradas: `debloat-specialist` curating + revisión `security-auditor`.
