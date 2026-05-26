# Paso 07 — Catálogo: añadir `strategy`, `lockedBy`, allowlist

**Área**: 02-cache-engine
**Tiempo estimado**: 3-4 horas (datos)
**Dependencias**: Paso 03 (modelos)

## Qué hacemos

Actualizar `cache-locations.json` para que cada entrada declare su `strategy` y opcionalmente `lockedBy`. Crear una **denylist** explícita de IDs y paths que NUNCA se tocan.

## Por qué

Sin `strategy` por entrada, `analyze_locations` no sabe cómo tratar cada path. Sin denylist, el cleaner puede pisar paths críticos como Windows Terminal (que ya nos pasó).

## Archivos que tocamos

- `.claude/skills/cache-scanner/RESOURCES/cache-locations.json` (modificado)
- `.claude/skills/cache-scanner/RESOURCES/cache-locations.schema.json` (modificado, schema)
- `src-tauri/src/domain/catalog.rs` (asegurar que parsea los nuevos campos)
- `src-tauri/src/models/cache.rs` (si los modelos del catálogo necesitan ampliarse)

## Cómo

### 1. Schema v2

Añadir al schema JSON los nuevos campos:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "cache-locations.schema.json",
  "title": "Cache Locations Catalog v2",
  "type": "object",
  "properties": {
    "schemaVersion": { "const": 2 },
    "entries": {
      "type": "array",
      "items": { "$ref": "#/definitions/CacheEntry" }
    }
  },
  "definitions": {
    "CacheEntry": {
      "type": "object",
      "required": ["id", "displayName", "path", "category", "strategy"],
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z][a-z0-9-]+$" },
        "displayName": { "type": "string" },
        "path": { "type": "string" },
        "category": {
          "enum": ["system-cache", "browser-cache", "uwp-cache", "user-temp",
                   "font", "windows-update", "logs", "other"]
        },
        "strategy": {
          "oneOf": [
            { "const": "direct-delete" },
            { "const": "browser-aware" },
            { "const": "process-locked" },
            { "const": "system-restart-required" },
            { "const": "take-ownership-and-delete" },
            {
              "type": "object",
              "required": ["uwpAppAware"],
              "properties": {
                "uwpAppAware": {
                  "type": "object",
                  "required": ["packageFamilyName"],
                  "properties": {
                    "packageFamilyName": { "type": "string" }
                  }
                }
              }
            }
          ]
        },
        "lockedBy": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Nombres de procesos esperables que bloquean este path (hint, no autoritativo)."
        },
        "risk": { "enum": ["low", "medium", "high"] },
        "requiresAdmin": { "type": "boolean" },
        "preconditions": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["type"],
            "properties": {
              "type": { "enum": ["process-not-running"] },
              "name": { "type": "string" }
            }
          }
        },
        "filters": {
          "type": "object",
          "properties": {
            "olderThanDays": { "type": "integer", "minimum": 0 },
            "exclude": { "type": "array", "items": { "type": "string" } },
            "minSizeKb": { "type": "integer", "minimum": 0 }
          }
        },
        "consequences": { "type": "array", "items": { "type": "string" } },
        "averageSize": { "type": "string" }
      }
    }
  }
}
```

### 2. Entries de ejemplo

```json
{
  "schemaVersion": 2,
  "entries": [
    {
      "id": "windows-temp",
      "displayName": "Windows Temp",
      "path": "%TEMP%",
      "category": "user-temp",
      "strategy": "direct-delete",
      "risk": "low",
      "filters": { "olderThanDays": 1 },
      "averageSize": "100MB-2GB"
    },
    {
      "id": "chrome-cache",
      "displayName": "Chrome — caché",
      "path": "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache",
      "category": "browser-cache",
      "strategy": "browser-aware",
      "lockedBy": ["chrome.exe"],
      "risk": "low",
      "averageSize": "200MB-2GB"
    },
    {
      "id": "uwp-spotify-cache",
      "displayName": "Spotify — caché UWP",
      "path": "%LOCALAPPDATA%\\Packages\\SpotifyAB.SpotifyMusic_zpdnekdrzrea0\\LocalCache",
      "category": "uwp-cache",
      "strategy": {
        "uwpAppAware": {
          "packageFamilyName": "SpotifyAB.SpotifyMusic_zpdnekdrzrea0"
        }
      },
      "lockedBy": ["Spotify.exe"],
      "risk": "low"
    },
    {
      "id": "dism-component-cleanup",
      "displayName": "DISM — componentes en desuso",
      "path": "%SYSTEMROOT%\\WinSxS\\Backup",
      "category": "system-cache",
      "strategy": "take-ownership-and-delete",
      "risk": "medium",
      "requiresAdmin": true,
      "consequences": [
        "No podrás revertir Windows Updates anteriores tras este punto."
      ]
    }
  ]
}
```

### 3. Denylist explícita

Crear archivo nuevo `.claude/skills/cache-scanner/RESOURCES/cache-denylist.json`:

```json
{
  "comment": "Paths que NUNCA se tocan, aunque aparezcan en algún catálogo o sean enviados manualmente al cleaner.",
  "patterns": [
    "Microsoft.WindowsTerminal_",
    "Microsoft.WindowsStore_",
    "Microsoft.DesktopAppInstaller_",
    "Microsoft.WindowsDefender_",
    "Microsoft.SecHealthUI_",
    "Microsoft.WindowsCalculator_",
    "Microsoft.WindowsCamera_",
    "Microsoft.WindowsNotepad_",
    "Microsoft.Paint_",
    "Microsoft.ScreenSketch_",
    "Microsoft.WindowsAlarms_",
    "Microsoft.MicrosoftStickyNotes_"
  ],
  "exactPaths": [
    "%SYSTEMROOT%",
    "%PROGRAMFILES%",
    "%PROGRAMFILES(X86)%",
    "%USERPROFILE%"
  ]
}
```

Esto se carga en Rust al iniciar y se cruza contra cada path antes de tocar nada.

### 4. Cargar la denylist en Rust

```rust
// src-tauri/src/domain/catalog.rs (añadir)

const DENYLIST_JSON: &str = include_str!("../../../.claude/skills/cache-scanner/RESOURCES/cache-denylist.json");

#[derive(Debug, Deserialize)]
struct DenyList {
    patterns: Vec<String>,
    exact_paths: Vec<String>,
}

pub fn load_denylist() -> AppResult<DenyList> {
    serde_json::from_str(DENYLIST_JSON)
        .map_err(|e| AppError::Catalog(format!("denylist parse: {}", e)))
}

pub fn is_path_denied(path: &str, deny: &DenyList) -> bool {
    let path_lower = path.to_lowercase();
    if deny.patterns.iter().any(|p| path_lower.contains(&p.to_lowercase())) {
        return true;
    }
    // Exact paths: resolver vars y comparar
    for ep in &deny.exact_paths {
        if let Ok(resolved) = std::env::var(strip_percents(ep)) {
            if path_lower.eq_ignore_ascii_case(&resolved.to_lowercase()) {
                return true;
            }
        }
    }
    false
}

fn strip_percents(s: &str) -> &str {
    s.trim_matches('%')
}
```

Y usarla en `analyze_locations`:

```rust
let deny = load_denylist()?;
// ...
if is_path_denied(&resolved, &deny) {
    skipped.push(SkippedLocation { ... });
    continue;
}
```

### 5. Validación al arranque

En `lib.rs::run`:

```rust
.setup(|app| {
    crate::core::settings::init();

    // Validar catálogos
    domain::catalog::validate_all().expect("catálogos embebidos inválidos");
    domain::catalog::load_denylist().expect("denylist inválida");

    // ...
});
```

## Criterio de done

- [ ] Schema v2 documenta los campos nuevos.
- [ ] `cache-locations.json` actualizado con `strategy` poblada para todas las entradas existentes.
- [ ] Cada entrada UWP tiene `lockedBy` o `strategy.uwpAppAware.packageFamilyName`.
- [ ] `cache-denylist.json` creado y validable.
- [ ] Rust carga denylist al arrancar.
- [ ] `is_path_denied` se llama en `analyze_locations` antes de procesar.
- [ ] Test: forzar pasar `Microsoft.WindowsTerminal_` → va a `skipped` con `DisallowedByAllowlist`.
