# Paso 01 — Schema v2 + migration del JSON

**Área**: 04-debloat-catalog
**Tiempo estimado**: 2-3 horas
**Dependencias**: ninguna

## Qué hacemos

Actualizar el schema JSON del catálogo de bloatware para v2: nuevos campos (`description`, `presets`, `tags`, `verified`, `disclaimerLevel`, `disclaimerText`, `learnMoreUrl`, `alternativeApps`, `maxWindowsBuild`, `compoundSteps`, `displayNamePattern`).

## Archivos que tocamos

- `.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.schema.json` (modificado)
- `.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json` (migrado a v2)
- `src-tauri/src/models/debloat.rs` (extender struct)
- `src-tauri/src/domain/catalog.rs` (validación v2)

## Cómo

### 1. Schema v2

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "bloatware-catalog.schema.json",
  "title": "Bloatware Catalog v2",
  "type": "object",
  "required": ["schemaVersion", "entries"],
  "properties": {
    "schemaVersion": { "const": 2 },
    "entries": {
      "type": "array",
      "items": { "$ref": "#/definitions/Entry" }
    }
  },
  "definitions": {
    "Entry": {
      "type": "object",
      "required": ["id", "displayName", "category", "risk", "removalMethod"],
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z][a-z0-9-]+$" },
        "displayName": { "type": "string" },
        "description": { "type": "string" },
        "category": { "enum": [
          "ms-consumer-app", "ms-system-app", "ai-and-telemetry",
          "ui-clutter", "oem-bloatware", "games",
          "third-party-bundled", "dev-tools-unused"
        ] },
        "risk": { "enum": ["low", "medium", "high"] },
        "consequences": { "type": "array", "items": { "type": "string" } },
        "removalMethod": { "enum": [
          "appx-user", "appx-provisioned", "appx-user-and-provisioned",
          "uninstaller-string", "service-and-files", "service-stop-only",
          "registry-policy", "scheduled-task-disable", "compound"
        ] },
        "appxPackageFamilyName": { "type": "string" },
        "appxProvisionedName": { "type": "string" },
        "wingetId": { "type": "string" },
        "uninstallRegistryPath": { "type": "string" },
        "displayNamePattern": { "type": "string", "description": "Regex/wildcard para uninstaller-string matching" },
        "publisher": { "type": "string" },
        "compoundSteps": {
          "type": "array",
          "items": { "$ref": "#/definitions/CompoundStep" }
        },
        "preservesDataByDefault": { "type": "boolean", "default": true },
        "requiresAdmin": { "type": "boolean", "default": false },
        "reversible": { "type": "boolean", "default": false },
        "reverseRecipe": { "$ref": "#/definitions/ReverseRecipe" },
        "minWindowsBuild": { "type": "integer" },
        "maxWindowsBuild": { "type": "integer" },
        "presets": {
          "type": "array",
          "items": { "enum": ["minimal", "recommended", "total", "gaming", "privacy-paranoid"] }
        },
        "tags": { "type": "array", "items": { "type": "string" } },
        "alternativeApps": { "type": "array", "items": { "type": "string" } },
        "learnMoreUrl": { "type": "string", "format": "uri" },
        "disclaimerLevel": { "enum": ["info", "warning", "danger"] },
        "disclaimerText": { "type": "string" },
        "iconAppx": { "type": "boolean", "default": false },
        "verified": { "type": "string", "format": "date" },
        "verifiedBy": { "type": "string" }
      }
    },
    "CompoundStep": {
      "type": "object",
      "required": ["method"],
      "properties": {
        "method": { "enum": ["appx-user", "appx-provisioned", "registry-policy", "scheduled-task-disable", "service-stop-only"] },
        "appxPackageFamilyName": { "type": "string" },
        "writes": { "type": "array", "items": { "$ref": "#/definitions/RegistryWrite" } },
        "taskName": { "type": "string" },
        "serviceName": { "type": "string" }
      }
    },
    "RegistryWrite": {
      "type": "object",
      "required": ["hive", "key", "name", "type", "value"],
      "properties": {
        "hive": { "enum": ["HKLM", "HKCU"] },
        "key": { "type": "string" },
        "name": { "type": "string" },
        "type": { "enum": ["REG_DWORD", "REG_SZ", "REG_BINARY"] },
        "value": {}
      }
    },
    "ReverseRecipe": {
      "oneOf": [
        {
          "type": "object",
          "required": ["kind", "packageFamilyName"],
          "properties": {
            "kind": { "const": "appxReinstall" },
            "packageFamilyName": { "type": "string" },
            "storeUrl": { "type": "string", "format": "uri" }
          }
        },
        {
          "type": "object",
          "required": ["kind", "operations"],
          "properties": {
            "kind": { "const": "registry" },
            "operations": { "type": "array", "items": { "$ref": "#/definitions/RegistryWrite" } }
          }
        },
        {
          "type": "object",
          "required": ["kind", "serviceName", "previousStartType"],
          "properties": {
            "kind": { "const": "service" },
            "serviceName": { "type": "string" },
            "previousStartType": { "type": "string" }
          }
        },
        {
          "type": "object",
          "required": ["kind", "reason"],
          "properties": {
            "kind": { "const": "noop" },
            "reason": { "type": "string" }
          }
        }
      ]
    }
  }
}
```

### 2. Migración del JSON existente

Editar `bloatware-catalog.json`. Wrap el array existente:

```json
{
  "schemaVersion": 2,
  "entries": [
    {
      "id": "ms-copilot",
      "displayName": "Microsoft Copilot",
      "description": "Asistente IA integrado en la barra de tareas.",
      "category": "ai-and-telemetry",
      "risk": "low",
      "consequences": [
        "Se quita el icono Copilot de la barra de tareas.",
        "Se desactiva el servicio asociado."
      ],
      "removalMethod": "appx-user",
      "appxPackageFamilyName": "Microsoft.Copilot_8wekyb3d8bbwe",
      "preservesDataByDefault": true,
      "requiresAdmin": true,
      "reversible": true,
      "reverseRecipe": {
        "kind": "appxReinstall",
        "packageFamilyName": "Microsoft.Copilot_8wekyb3d8bbwe",
        "storeUrl": "ms-windows-store://pdp/?ProductId=..."
      },
      "minWindowsBuild": 22000,
      "presets": ["recommended", "total", "privacy-paranoid"],
      "tags": ["ai", "copilot"],
      "alternativeApps": ["ChatGPT desktop", "Claude desktop"],
      "verified": "2026-05-25",
      "verifiedBy": "@joelsanchez"
    }
    // ... resto de entradas existentes migradas
  ]
}
```

### 3. Extender el struct Rust

```rust
// src-tauri/src/models/debloat.rs (modificar)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareEntry {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub category: String,
    pub risk: String,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub removal_method: String,
    #[serde(default)]
    pub appx_package_family_name: Option<String>,
    #[serde(default)]
    pub appx_provisioned_name: Option<String>,
    #[serde(default)]
    pub winget_id: Option<String>,
    #[serde(default)]
    pub uninstall_registry_path: Option<String>,
    #[serde(default)]
    pub display_name_pattern: Option<String>,    // ← NUEVO
    #[serde(default)]
    pub publisher: Option<String>,                // ← NUEVO
    #[serde(default)]
    pub compound_steps: Vec<CompoundStep>,        // ← NUEVO
    #[serde(default = "default_preserves")]
    pub preserves_data_by_default: bool,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub reversible: bool,
    #[serde(default)]
    pub reverse_recipe: Option<ReverseRecipeEntry>,   // ← AHORA discriminated union
    #[serde(default)]
    pub min_windows_build: Option<u32>,
    #[serde(default)]
    pub max_windows_build: Option<u32>,            // ← NUEVO
    #[serde(default)]
    pub presets: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,                          // ← NUEVO
    #[serde(default)]
    pub alternative_apps: Vec<String>,              // ← NUEVO
    #[serde(default)]
    pub learn_more_url: Option<String>,             // ← NUEVO
    #[serde(default)]
    pub disclaimer_level: Option<String>,           // ← NUEVO
    #[serde(default)]
    pub disclaimer_text: Option<String>,            // ← NUEVO
    #[serde(default)]
    pub icon_appx: bool,
    #[serde(default)]
    pub verified: Option<String>,                   // ← NUEVO
    #[serde(default)]
    pub verified_by: Option<String>,                // ← NUEVO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompoundStep {
    pub method: String,
    #[serde(default)] pub appx_package_family_name: Option<String>,
    #[serde(default)] pub writes: Vec<RegistryWrite>,
    #[serde(default)] pub task_name: Option<String>,
    #[serde(default)] pub service_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryWrite {
    pub hive: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReverseRecipeEntry {
    AppxReinstall { package_family_name: String, store_url: Option<String> },
    Registry { operations: Vec<RegistryWrite> },
    Service { service_name: String, previous_start_type: String },
    Noop { reason: String },
}
```

### 4. Validación al cargar

`catalog::load_bloatware_catalog` ya valida. Asegurar que falla con mensaje claro si encuentra `schemaVersion != 2`:

```rust
fn validate_schema_version(json: &serde_json::Value) -> AppResult<()> {
    let ver = json.get("schemaVersion").and_then(|v| v.as_u64()).unwrap_or(0);
    if ver != 2 {
        return Err(AppError::Catalog(format!(
            "bloatware-catalog schema version inválida: esperado 2, encontrado {}",
            ver
        )));
    }
    Ok(())
}
```

### 5. AJV check en CI (opcional aquí, formal en `13-testing-ci`)

```yaml
# .github/workflows/validate-catalogs.yml
- run: npm i -g ajv-cli ajv-formats
- run: ajv validate -s .claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.schema.json -d .claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json --spec=draft7 -c ajv-formats
```

## Criterio de done

- [ ] Schema v2 publicado y formato válido (AJV check pasa).
- [ ] `bloatware-catalog.json` migrado a v2 con `schemaVersion: 2`.
- [ ] Todas las entradas existentes migradas sin perder info.
- [ ] Struct Rust extendido con los nuevos campos opcionales.
- [ ] `cargo check` pasa.
- [ ] `catalog::validate_all()` pasa con el nuevo catálogo.
- [ ] CI tiene job que valida el JSON contra el schema.
