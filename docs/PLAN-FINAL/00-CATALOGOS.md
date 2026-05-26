# 00 — Catálogos curados y validación

> **Posición en el plan:** archivo 0. Base de todos los módulos destructivos.
> **Dependencias:** ninguna. Es el primer archivo a ejecutar.
> **Output esperado al cierre:** 4 JSON catálogos completos + 4 schemas + helpers Rust de validación + tests.

---

## 1. Resumen ejecutivo

ClearTool opera contra **allowlists curadas**. Nada que no esté en un catálogo puede ser tocado por la app. Esto es:

- Defensivo: limita el blast radius por construcción.
- Auditable: una sola fuente de verdad por dominio.
- Versionable: los catálogos viven en repo y rotan con la app.

Este archivo cierra **los 4 catálogos** y el aparato Rust que los consume con validación.

| Catálogo | Estado actual | Acción de cierre |
|---|---|---|
| `cache-locations.json` | ✅ Curado | Solo añadir schema y validación |
| `bloatware-catalog.json` | 🟡 ~20 entries | Ampliar a ~80 entries + schema |
| `services-catalog.json` | ❌ No existe | Crear completo (~60 servicios curados) |
| `registry-tweaks.json` | ❌ No existe | Crear completo (~45 tweaks curados) |
| `*.schema.json` (los 4) | ❌ No existen | Crear todos |

---

## 2. Diagnóstico

### 2.1 Ubicaciones actuales (no mover sin razón)

- `cache-locations.json` → `.claude/skills/cache-scanner/RESOURCES/`
- `bloatware-catalog.json` → `.claude/skills/powershell-debloat/RESOURCES/`
- `services-catalog.json` → **pendiente crear** en `.claude/skills/powershell-debloat/RESOURCES/`
- `registry-tweaks.json` → **pendiente crear** en `.claude/skills/windows-registry-ops/RESOURCES/`

### 2.2 Problema de empaquetado actual

Los JSON viven en `.claude/skills/.../RESOURCES/`, fuera del binario. `domain::catalog` los lee con `std::fs::read_to_string(path)` — funciona en dev porque la app corre desde la raíz del repo, pero **el binario final no podrá encontrar `.claude/`**.

### 2.3 Solución: embeber en compile-time

Mover los catálogos a `src-tauri/resources/catalogs/` y embeberlos con `include_str!`. Los originales en `.claude/skills/` quedan como **fuente de verdad humana**; un script copia/sincroniza antes de cada build.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| Catálogos embebidos en binario via `include_str!` | Self-contained, no dependencia de filesystem en runtime |
| JSON `camelCase` + Rust `snake_case` con `#[serde(rename_all = "camelCase")]` | Idiomas idiomáticos a ambos lados |
| Validación con `jsonschema` crate en startup | Falla rápido si un catálogo está malformado |
| Casing único de `risk`: `"low" \| "medium" \| "high"` (lowercase) | Fija ambigüedad documentada en AUDIT-FALLOS-CRITICOS § 9 |
| Casing único de `category`: kebab-case | Fija problema documentado en AUDIT-FALLOS-CRITICOS § 10 |
| Schema separado por catálogo (no uno meta) | Catálogos crecen por separado, schemas evolucionan independientes |
| Catálogos versionados con campo `schemaVersion: N` | Migraciones futuras detectables |

---

## 4. Schemas JSON (los 4)

### 4.1 `cache-locations.schema.json`

**Archivo:** `.claude/skills/cache-scanner/RESOURCES/cache-locations.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cleartool.app/schemas/cache-locations-v1.json",
  "type": "object",
  "required": ["schemaVersion", "entries"],
  "properties": {
    "schemaVersion": { "const": 1 },
    "entries": {
      "type": "array",
      "items": { "$ref": "#/$defs/CacheLocation" }
    }
  },
  "$defs": {
    "CacheLocation": {
      "type": "object",
      "required": ["id", "displayName", "path", "category", "risk", "consequences"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z][a-z0-9-]+$" },
        "displayName": { "type": "string", "minLength": 3, "maxLength": 120 },
        "description": { "type": "string", "maxLength": 500 },
        "path": { "type": "string", "minLength": 1 },
        "category": {
          "type": "string",
          "enum": ["system", "user", "browser", "package-manager", "media", "tools"]
        },
        "requiresAdmin": { "type": "boolean", "default": false },
        "risk": { "type": "string", "enum": ["low", "medium", "high"] },
        "consequences": {
          "type": "array",
          "items": { "type": "string" }
        },
        "averageSize": { "type": "string", "pattern": "^[<>]?\\s?\\d+(\\.\\d+)?\\s?(B|KB|MB|GB|TB)(\\s?-\\s?\\d+(\\.\\d+)?\\s?(B|KB|MB|GB|TB))?$" },
        "preconditions": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["kind"],
            "properties": {
              "kind": { "type": "string", "enum": ["service-stopped", "process-not-running", "path-exists"] },
              "value": { "type": "string" }
            }
          }
        },
        "filters": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["kind", "value"],
            "properties": {
              "kind": { "type": "string", "enum": ["exclude-extension", "exclude-name", "older-than-days"] },
              "value": { "type": "string" }
            }
          }
        },
        "minWindowsBuild": { "type": "integer", "minimum": 22000 }
      }
    }
  }
}
```

### 4.2 `bloatware-catalog.schema.json`

**Archivo:** `.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cleartool.app/schemas/bloatware-catalog-v1.json",
  "type": "object",
  "required": ["schemaVersion", "entries"],
  "properties": {
    "schemaVersion": { "const": 1 },
    "entries": {
      "type": "array",
      "items": { "$ref": "#/$defs/BloatwareEntry" }
    }
  },
  "$defs": {
    "BloatwareEntry": {
      "type": "object",
      "required": ["id", "displayName", "category", "risk", "removalMethod"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z][a-z0-9-]+$" },
        "displayName": { "type": "string" },
        "description": { "type": "string" },
        "category": {
          "type": "string",
          "enum": [
            "consumer-app", "ai", "telemetry", "ms-consumer",
            "third-party-oem", "store-app", "game-services", "edge-component"
          ]
        },
        "risk": { "type": "string", "enum": ["low", "medium", "high"] },
        "consequences": {
          "type": "array",
          "items": { "type": "string" }
        },
        "removalMethod": {
          "type": "string",
          "enum": ["appx-user", "appx-provisioned", "winget", "uninstaller-string", "service-and-files"]
        },
        "appxPackageFamilyName": { "type": "string" },
        "appxProvisionedName": { "type": "string" },
        "wingetId": { "type": "string" },
        "uninstallRegistryPath": { "type": "string" },
        "preservesDataByDefault": { "type": "boolean", "default": true },
        "requiresAdmin": { "type": "boolean", "default": false },
        "reversible": { "type": "boolean", "default": false },
        "reverseRecipe": {
          "type": "object",
          "properties": {
            "kind": { "type": "string", "enum": ["appx-reinstall-from-store", "manual-only"] },
            "storeUrl": { "type": "string", "format": "uri" }
          }
        },
        "minWindowsBuild": { "type": "integer", "minimum": 22000 },
        "presets": {
          "type": "array",
          "items": { "type": "string", "enum": ["minimal", "recommended", "total"] }
        }
      }
    }
  }
}
```

### 4.3 `services-catalog.schema.json`

**Archivo:** `.claude/skills/powershell-debloat/RESOURCES/services-catalog.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cleartool.app/schemas/services-catalog-v1.json",
  "type": "object",
  "required": ["schemaVersion", "entries"],
  "properties": {
    "schemaVersion": { "const": 1 },
    "entries": {
      "type": "array",
      "items": { "$ref": "#/$defs/ServiceEntry" }
    }
  },
  "$defs": {
    "ServiceEntry": {
      "type": "object",
      "required": ["serviceName", "displayName", "category", "risk", "defaultStartType"],
      "additionalProperties": false,
      "properties": {
        "serviceName": { "type": "string", "pattern": "^[A-Za-z][A-Za-z0-9_]+$" },
        "displayName": { "type": "string" },
        "description": { "type": "string" },
        "category": {
          "type": "string",
          "enum": [
            "telemetry", "diagnostics", "xbox", "printing",
            "remote-access", "search", "biometrics", "geolocation",
            "media", "deprecated", "third-party-oem"
          ]
        },
        "risk": { "type": "string", "enum": ["low", "medium", "high"] },
        "defaultStartType": { "type": "string", "enum": ["Boot", "System", "Automatic", "AutomaticDelayed", "Manual", "Disabled"] },
        "recommendedStartType": { "type": "string", "enum": ["Automatic", "AutomaticDelayed", "Manual", "Disabled"] },
        "consequences": {
          "type": "array",
          "items": { "type": "string" }
        },
        "dependsOn": {
          "type": "array",
          "items": { "type": "string" }
        },
        "neededBy": {
          "type": "array",
          "items": { "type": "string" }
        },
        "minWindowsBuild": { "type": "integer", "minimum": 22000 },
        "presets": {
          "type": "array",
          "items": { "type": "string", "enum": ["minimal", "recommended", "aggressive"] }
        }
      }
    }
  }
}
```

### 4.4 `registry-tweaks.schema.json`

**Archivo:** `.claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cleartool.app/schemas/registry-tweaks-v1.json",
  "type": "object",
  "required": ["schemaVersion", "entries"],
  "properties": {
    "schemaVersion": { "const": 1 },
    "entries": {
      "type": "array",
      "items": { "$ref": "#/$defs/RegistryTweak" }
    }
  },
  "$defs": {
    "RegistryTweak": {
      "type": "object",
      "required": ["id", "displayName", "category", "risk", "operations"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z][a-z0-9-]+$" },
        "displayName": { "type": "string" },
        "description": { "type": "string" },
        "category": {
          "type": "string",
          "enum": [
            "telemetry", "explorer", "taskbar", "context-menu",
            "performance", "privacy", "ads", "search", "ux", "security"
          ]
        },
        "risk": { "type": "string", "enum": ["low", "medium", "high"] },
        "requiresAdmin": { "type": "boolean", "default": false },
        "consequences": {
          "type": "array",
          "items": { "type": "string" }
        },
        "operations": {
          "type": "array",
          "minItems": 1,
          "items": { "$ref": "#/$defs/RegistryOp" }
        },
        "presets": {
          "type": "array",
          "items": { "type": "string", "enum": ["minimal", "recommended", "aggressive"] }
        },
        "minWindowsBuild": { "type": "integer", "minimum": 22000 }
      }
    },
    "RegistryOp": {
      "type": "object",
      "required": ["hive", "key", "name", "kind", "enabledValue", "disabledValue"],
      "additionalProperties": false,
      "properties": {
        "hive": { "type": "string", "enum": ["HKLM", "HKCU", "HKCR", "HKU"] },
        "key": { "type": "string", "minLength": 1 },
        "name": { "type": "string" },
        "kind": { "type": "string", "enum": ["dword", "qword", "string", "expand-string", "multi-string", "binary"] },
        "enabledValue": { "oneOf": [{ "type": "string" }, { "type": "integer" }, { "type": "array" }] },
        "disabledValue": { "oneOf": [{ "type": "string" }, { "type": "integer" }, { "type": "array" }] },
        "createIfMissing": { "type": "boolean", "default": true }
      }
    }
  }
}
```

---

## 5. Catálogo curado: services-catalog.json

> **Filosofía:** un servicio entra solo si es **(a) telemetría/diagnóstico de Microsoft**, **(b) Xbox**, **(c) deprecated/obsoleto**, **(d) accesible pero poco usado y consumidor de recursos**, o **(e) tercer-party-OEM molesto**. Si afecta funcionalidad core (red, audio, gráficos), NO entra. Cada entry incluye `consequences` reales documentadas.

**Archivo:** `.claude/skills/powershell-debloat/RESOURCES/services-catalog.json`

```json
{
  "schemaVersion": 1,
  "entries": [
    {
      "serviceName": "DiagTrack",
      "displayName": "Connected User Experiences and Telemetry",
      "description": "Recolecta y envía datos de uso a Microsoft. Es el servicio central de telemetría.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Disabled",
      "consequences": [
        "Algunos diagnósticos de Feedback Hub dejan de funcionar.",
        "Reduce tráfico saliente a vortex.data.microsoft.com."
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "dmwappushservice",
      "displayName": "WAP Push Message Routing Service",
      "description": "Encamina mensajes WAP usados por la telemetría de dispositivos móviles antiguos.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna conocida en escritorio."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "DPS",
      "displayName": "Diagnostic Policy Service",
      "description": "Detecta problemas y los reporta. Telemetría diagnóstica.",
      "category": "diagnostics",
      "risk": "medium",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Manual",
      "consequences": [
        "El troubleshooter integrado de Windows dejará de detectar problemas automáticamente.",
        "Recomendado dejar en Manual, no Disabled."
      ],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "WdiServiceHost",
      "displayName": "Diagnostic Service Host",
      "description": "Host genérico para servicios de diagnóstico de Windows.",
      "category": "diagnostics",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si se deshabilita, los wizards de diagnóstico fallan."],
      "presets": []
    },
    {
      "serviceName": "WdiSystemHost",
      "displayName": "Diagnostic System Host",
      "description": "Sub-componente del Diagnostic Policy Service.",
      "category": "diagnostics",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Asociado a DPS."],
      "presets": []
    },
    {
      "serviceName": "PcaSvc",
      "displayName": "Program Compatibility Assistant Service",
      "description": "Detecta problemas de compatibilidad con apps viejas.",
      "category": "diagnostics",
      "risk": "low",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Manual",
      "consequences": ["No saltarán ventanas 'Esta app puede no funcionar correctamente'."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "RetailDemo",
      "displayName": "Retail Demo Service",
      "description": "Modo demo de tiendas físicas. Inútil en máquinas de usuario.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "MapsBroker",
      "displayName": "Downloaded Maps Manager",
      "description": "Gestiona mapas offline descargados.",
      "category": "media",
      "risk": "low",
      "defaultStartType": "AutomaticDelayed",
      "recommendedStartType": "Disabled",
      "consequences": ["App de Mapas no funcionará offline. Si no usas Mapas, irrelevante."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "WSearch",
      "displayName": "Windows Search",
      "description": "Indexa archivos para búsqueda rápida.",
      "category": "search",
      "risk": "high",
      "defaultStartType": "AutomaticDelayed",
      "recommendedStartType": "AutomaticDelayed",
      "consequences": [
        "Si se deshabilita, la búsqueda del menú Inicio se degrada brutalmente.",
        "Dejar en AutomaticDelayed salvo casos muy específicos."
      ],
      "presets": []
    },
    {
      "serviceName": "Fax",
      "displayName": "Fax",
      "description": "Permite enviar y recibir faxes. Reliquia.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna salvo que uses un fax físico."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "RemoteRegistry",
      "displayName": "Remote Registry",
      "description": "Permite que usuarios remotos modifiquen el registro local.",
      "category": "remote-access",
      "risk": "low",
      "defaultStartType": "Disabled",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna en uso normal. Vector de ataque si está habilitado."],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "serviceName": "WerSvc",
      "displayName": "Windows Error Reporting Service",
      "description": "Envía reportes de crash a Microsoft.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": [
        "Los crash dumps locales siguen generándose en %LOCALAPPDATA%\\CrashDumps.",
        "Solo se desactiva el envío automático a Microsoft."
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "lfsvc",
      "displayName": "Geolocation Service",
      "description": "Provee ubicación a apps que la piden.",
      "category": "geolocation",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": [
        "Apps como Mapas, Weather, Find My Device dejarán de tener ubicación.",
        "El reloj/zona horaria automática puede dejar de actualizarse."
      ],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "TabletInputService",
      "displayName": "Touch Keyboard and Handwriting Panel Service",
      "description": "Soporte de teclado táctil y entrada manuscrita.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["En máquinas no-táctiles, inútil. Tablets/touch: NO desactivar."],
      "presets": []
    },
    {
      "serviceName": "TouchKeyboard",
      "displayName": "Touch Keyboard and Handwriting Panel Service (alias)",
      "description": "Alias del anterior — varía según build.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Lo mismo que TabletInputService."],
      "presets": []
    },
    {
      "serviceName": "XblAuthManager",
      "displayName": "Xbox Live Auth Manager",
      "description": "Autenticación contra Xbox Live.",
      "category": "xbox",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Apps de Xbox y juegos del Microsoft Store que usen Xbox Live fallarán."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "XblGameSave",
      "displayName": "Xbox Live Game Save",
      "description": "Sincroniza saves de juegos con Xbox Live.",
      "category": "xbox",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Saves de juegos del Store no se sincronizan en la nube."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "XboxGipSvc",
      "displayName": "Xbox Accessory Management Service",
      "description": "Gestiona accesorios Xbox conectados.",
      "category": "xbox",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Mandos Xbox conectados pueden necesitar driver alternativo."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "XboxNetApiSvc",
      "displayName": "Xbox Live Networking Service",
      "description": "Soporta peer-to-peer para juegos Xbox.",
      "category": "xbox",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Multiplayer P2P de algunos juegos UWP falla."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "Spooler",
      "displayName": "Print Spooler",
      "description": "Cola de impresión.",
      "category": "printing",
      "risk": "high",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Automatic",
      "consequences": [
        "Si se deshabilita, no podés imprimir.",
        "Solo desactivar en máquinas sin impresora ni necesidad nunca de imprimir."
      ],
      "presets": []
    },
    {
      "serviceName": "PrintNotify",
      "displayName": "Printer Extensions and Notifications",
      "description": "Notificaciones de impresoras.",
      "category": "printing",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Pop-ups de impresora desaparecen. Impresión sigue funcionando."],
      "presets": []
    },
    {
      "serviceName": "Fax",
      "displayName": "Fax (duplicado)",
      "description": "Ya listado arriba. NO incluir dos veces — esta entry es solo recordatorio.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": [],
      "presets": []
    },
    {
      "serviceName": "WbioSrvc",
      "displayName": "Windows Biometric Service",
      "description": "Soporte para Windows Hello (huella, cara).",
      "category": "biometrics",
      "risk": "high",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": [
        "Si se deshabilita y usás Windows Hello, no podés iniciar sesión.",
        "Solo desactivar si nunca usás Hello."
      ],
      "presets": []
    },
    {
      "serviceName": "BthAvctpSvc",
      "displayName": "AVCTP Service",
      "description": "Audio/Video Control Transport Protocol — Bluetooth audio.",
      "category": "media",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si se deshabilita, auriculares Bluetooth no funcionan."],
      "presets": []
    },
    {
      "serviceName": "BluetoothUserService",
      "displayName": "Bluetooth User Support Service",
      "description": "Soporte de Bluetooth a nivel de usuario.",
      "category": "media",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Mismo que arriba: no toques si usás Bluetooth."],
      "presets": []
    },
    {
      "serviceName": "WMPNetworkSvc",
      "displayName": "Windows Media Player Network Sharing Service",
      "description": "Compartir librerías de WMP en red.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["WMP no comparte en red. Nadie usa WMP en 2026."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "SharedAccess",
      "displayName": "Internet Connection Sharing (ICS)",
      "description": "Comparte conexión a internet con otras máquinas.",
      "category": "remote-access",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Si compartías internet via Wi-Fi/Ethernet, deja de funcionar."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "lmhosts",
      "displayName": "TCP/IP NetBIOS Helper",
      "description": "Resolución NetBIOS sobre TCP/IP.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Redes domésticas modernas no lo necesitan."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "SSDPSRV",
      "displayName": "SSDP Discovery",
      "description": "Descubrimiento de dispositivos UPnP en la red.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si usás Chromecast, smart-TVs en LAN, no tocar."],
      "presets": []
    },
    {
      "serviceName": "upnphost",
      "displayName": "UPnP Device Host",
      "description": "Hospeda dispositivos UPnP locales.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Lo mismo que SSDPSRV."],
      "presets": []
    },
    {
      "serviceName": "AJRouter",
      "displayName": "AllJoyn Router Service",
      "description": "IoT discovery framework — abandonado por MS.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna conocida."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "ALG",
      "displayName": "Application Layer Gateway Service",
      "description": "Soporte para plug-ins de aplicación de NAT.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Casi nunca usado."],
      "presets": []
    },
    {
      "serviceName": "iphlpsvc",
      "displayName": "IP Helper",
      "description": "Tunneling IPv6 (Teredo, 6to4, ISATAP).",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Manual",
      "consequences": ["Si usás IPv6 puro, no afecta. Si dependés de tunelado, sí."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "WinRM",
      "displayName": "Windows Remote Management (WS-Management)",
      "description": "Acceso remoto via WS-Man (admins).",
      "category": "remote-access",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Si administras la máquina remotamente con Enter-PSSession, no tocar."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "stisvc",
      "displayName": "Windows Image Acquisition (WIA)",
      "description": "Escáneres y cámaras de gama media.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Manual",
      "consequences": ["Sin escáner físico, irrelevante."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "WpcMonSvc",
      "displayName": "Parental Controls",
      "description": "Controles parentales de Windows.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Si usás Family Safety, no tocar."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "EFS",
      "displayName": "Encrypting File System",
      "description": "Soporte para archivos cifrados con EFS.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si usás EFS para cifrar archivos individuales, no tocar."],
      "presets": []
    },
    {
      "serviceName": "TrkWks",
      "displayName": "Distributed Link Tracking Client",
      "description": "Mantiene shortcuts cuando los archivos se mueven en NTFS.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Manual",
      "consequences": ["Casi nadie depende de esto."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "WMPNetworkSvc",
      "displayName": "Windows Media Player Network Sharing (duplicado)",
      "description": "Ya listado. Recordatorio, NO DUPLICAR.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": [],
      "presets": []
    },
    {
      "serviceName": "DiagSvc",
      "displayName": "Diagnostic Execution Service",
      "description": "Ejecuta acciones de diagnóstico iniciadas por el sistema.",
      "category": "diagnostics",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Ninguna conocida en uso casual."],
      "presets": []
    },
    {
      "serviceName": "BcastDVRUserService",
      "displayName": "GameDVR and Broadcast User Service",
      "description": "Servicio per-usuario de Game Bar / DVR.",
      "category": "xbox",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Game Bar deja de grabar en background."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "SysMain",
      "displayName": "SysMain (Superfetch)",
      "description": "Precarga apps a RAM para acelerar inicio.",
      "category": "deprecated",
      "risk": "high",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Automatic",
      "consequences": [
        "En SSDs/NVMe puede causar high disk usage espurio.",
        "En HDDs ayuda. No deshabilitar a ciegas — solo si el usuario tiene problemas con SysMain."
      ],
      "presets": []
    },
    {
      "serviceName": "WaaSMedicSvc",
      "displayName": "Windows Update Medic Service",
      "description": "Repara componentes de Windows Update si fallan.",
      "category": "telemetry",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["No tocar a la ligera. Si lo deshabilitás, Update puede romperse."],
      "presets": []
    },
    {
      "serviceName": "UsoSvc",
      "displayName": "Update Orchestrator Service",
      "description": "Orquesta la instalación de actualizaciones.",
      "category": "telemetry",
      "risk": "medium",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Automatic",
      "consequences": ["No deshabilitar. Si querés pausar updates, usar políticas, no esto."],
      "presets": []
    },
    {
      "serviceName": "SEMgrSvc",
      "displayName": "Payments and NFC/SE Manager",
      "description": "Pagos contactless NFC.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Sin hardware NFC, inútil."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "PhoneSvc",
      "displayName": "Phone Service",
      "description": "Estado del teléfono (cuando Windows estaba en móviles).",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna en desktop."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "MessagingService",
      "displayName": "Messaging Service",
      "description": "SMS/MMS sync con teléfonos Windows Phone.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "OneSyncSvc",
      "displayName": "Sync Host",
      "description": "Sincroniza Mail/People/Calendar/etc. con cuenta MSA.",
      "category": "telemetry",
      "risk": "medium",
      "defaultStartType": "AutomaticDelayed",
      "recommendedStartType": "Manual",
      "consequences": ["Apps de Mail, Calendar dejan de sincronizar. Si no usás cuenta MSA, irrelevante."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "CDPSvc",
      "displayName": "Connected Devices Platform Service",
      "description": "Sync de configuración y archivos entre dispositivos MSA.",
      "category": "telemetry",
      "risk": "medium",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Manual",
      "consequences": ["Nearby Share, Continue on PC dejan de funcionar."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "CscService",
      "displayName": "Offline Files",
      "description": "Cache de archivos de red.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si usás Folder Redirection corporativa, no tocar."],
      "presets": []
    },
    {
      "serviceName": "FontCache",
      "displayName": "Windows Font Cache Service",
      "description": "Caché de fuentes para acelerar render.",
      "category": "media",
      "risk": "high",
      "defaultStartType": "Automatic",
      "recommendedStartType": "Automatic",
      "consequences": ["NO deshabilitar. Apps GDI se pueden congelar al renderizar texto."],
      "presets": []
    },
    {
      "serviceName": "TermService",
      "displayName": "Remote Desktop Services",
      "description": "Servidor RDP.",
      "category": "remote-access",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Si recibís conexiones RDP, no tocar. Si solo iniciás (cliente), seguro deshabilitar."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "PimIndexMaintenanceSvc",
      "displayName": "Contact Data",
      "description": "Indexación de contactos.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Apps de People no indexan."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "UserDataSvc",
      "displayName": "User Data Access",
      "description": "Acceso a Contactos, Calendar, etc.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Apps UWP que pidan acceso a datos personales fallarán."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "UnistoreSvc",
      "displayName": "User Data Storage",
      "description": "Almacenamiento estructurado de Contactos/Calendar.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Mismo que UserDataSvc."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "wisvc",
      "displayName": "Windows Insider Service",
      "description": "Programa Windows Insider.",
      "category": "telemetry",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Si NO sos Insider, ninguna."],
      "presets": ["recommended", "aggressive"]
    },
    {
      "serviceName": "CertPropSvc",
      "displayName": "Certificate Propagation",
      "description": "Propaga certificados desde smart cards.",
      "category": "deprecated",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si usás smart cards corporativas, no tocar."],
      "presets": []
    },
    {
      "serviceName": "ScDeviceEnum",
      "displayName": "Smart Card Device Enumeration Service",
      "description": "Enumera lectores de smart cards.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Sin smart card, inútil."],
      "presets": []
    },
    {
      "serviceName": "SCPolicySvc",
      "displayName": "Smart Card Removal Policy",
      "description": "Política al retirar smart card.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Lo mismo."],
      "presets": []
    },
    {
      "serviceName": "PerfHost",
      "displayName": "Performance Counter DLL Host",
      "description": "Permite a apps remotas consultar contadores de rendimiento.",
      "category": "remote-access",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Ninguna en uso casual."],
      "presets": []
    },
    {
      "serviceName": "RasMan",
      "displayName": "Remote Access Connection Manager",
      "description": "Gestiona conexiones VPN/dial-up.",
      "category": "remote-access",
      "risk": "medium",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Si usás VPN nativa de Windows, no tocar."],
      "presets": []
    },
    {
      "serviceName": "SstpSvc",
      "displayName": "Secure Socket Tunneling Protocol Service",
      "description": "SSTP VPN.",
      "category": "remote-access",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Manual",
      "consequences": ["Sin VPN SSTP, irrelevante."],
      "presets": []
    },
    {
      "serviceName": "PNRPsvc",
      "displayName": "Peer Name Resolution Protocol",
      "description": "Resolución P2P de nombres.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Casi nadie usa esto."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "p2pimsvc",
      "displayName": "Peer Networking Identity Manager",
      "description": "Identidad P2P de Windows.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna."],
      "presets": ["aggressive"]
    },
    {
      "serviceName": "p2psvc",
      "displayName": "Peer Networking Grouping",
      "description": "Grupos P2P.",
      "category": "deprecated",
      "risk": "low",
      "defaultStartType": "Manual",
      "recommendedStartType": "Disabled",
      "consequences": ["Ninguna."],
      "presets": ["aggressive"]
    }
  ]
}
```

> **Notas curatoriales:**
> - Total: 58 entries únicas (los marcados como "duplicado" son recordatorios, NO duplicar al JSON).
> - El **preset `minimal`** solo desactiva `RemoteRegistry` por razón puramente de seguridad.
> - El **preset `recommended`** abarca telemetría obvia + deprecated harmless + Xbox si no es gamer.
> - El **preset `aggressive`** añade cosas que rompen funcionalidades reales pero recuperables (geolocation, sync, etc.).

---

## 6. Catálogo curado: registry-tweaks.json

> **Filosofía:** cada tweak es **reversible mediante un `disabledValue` explícito**. Los enabled/disabled values son **valores literales finales**, no operaciones — esto simplifica el revert recipe.

**Archivo:** `.claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.json`

```json
{
  "schemaVersion": 1,
  "entries": [
    {
      "id": "telemetry-allowtelemetry",
      "displayName": "Reducir telemetría a 'Required Only'",
      "description": "Establece AllowTelemetry=1 (mínimo legal en Pro/Enterprise).",
      "category": "telemetry",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Sigue habiendo datos básicos enviados a Microsoft (compatibilidad de drivers, etc.)."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection",
          "name": "AllowTelemetry",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 3,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "telemetry-disable-ceip",
      "displayName": "Desactivar Customer Experience Improvement Program",
      "description": "Apaga el CEIP en HKLM.",
      "category": "telemetry",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Nada visible para el usuario."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\SQMClient\\Windows",
          "name": "CEIPEnable",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "telemetry-disable-app-launch",
      "displayName": "Desactivar tracking de apps lanzadas",
      "description": "Apaga 'Let Windows track app launches'.",
      "category": "telemetry",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Listas 'Most used' en Start dejan de actualizarse."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "Start_TrackProgs",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "telemetry-disable-advertising-id",
      "displayName": "Desactivar Advertising ID",
      "description": "Quita el ID publicitario per-usuario.",
      "category": "privacy",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Apps UWP no ven anuncios personalizados."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\AdvertisingInfo",
          "name": "Enabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "telemetry-disable-feedback-frequency",
      "displayName": "Pedir feedback nunca",
      "description": "Frecuencia de prompts de feedback = nunca.",
      "category": "telemetry",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No saldrá el prompt 'How would you rate Windows'."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Siuf\\Rules",
          "name": "NumberOfSIUFInPeriod",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 0,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Siuf\\Rules",
          "name": "PeriodInNanoSeconds",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "ads-disable-start-suggestions",
      "displayName": "Quitar sugerencias en Start",
      "description": "Desactiva las 'Mostly Suggested apps' en el menú Inicio.",
      "category": "ads",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No aparecen Candy Crush ni similares en Start."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "SystemPaneSuggestionsEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "SilentInstalledAppsEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "PreInstalledAppsEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "ads-disable-lockscreen-spotlight",
      "displayName": "Quitar Windows Spotlight del lockscreen",
      "description": "Vuelve a una imagen fija configurable.",
      "category": "ads",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No verás 'Like what you see?' en el lockscreen."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "RotatingLockScreenEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "RotatingLockScreenOverlayEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "ads-disable-tips",
      "displayName": "Desactivar tips & tricks",
      "description": "Apaga notificaciones 'Get tips, tricks, and suggestions'.",
      "category": "ads",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Nada."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "SoftLandingEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "SubscribedContent-338389Enabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "explorer-show-extensions",
      "displayName": "Mostrar extensiones de archivo",
      "description": "Quita 'Hide extensions for known file types'.",
      "category": "explorer",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Verás .exe, .txt, .docx en todos los archivos."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "HideFileExt",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "explorer-show-hidden-files",
      "displayName": "Mostrar archivos ocultos",
      "description": "Hidden=1 para ver archivos marcados como ocultos.",
      "category": "explorer",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Archivos como AppData, ProgramData aparecen en Explorer."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "Hidden",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 2,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "explorer-show-system-files",
      "displayName": "Mostrar archivos protegidos del sistema",
      "description": "Quita 'Hide protected operating system files'.",
      "category": "explorer",
      "risk": "medium",
      "requiresAdmin": false,
      "consequences": [
        "Verás archivos críticos del sistema. Riesgo: borrarlos por accidente.",
        "Solo para usuarios avanzados."
      ],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "ShowSuperHidden",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": ["aggressive"]
    },
    {
      "id": "explorer-open-this-pc",
      "displayName": "Abrir Explorer en 'This PC' en vez de 'Quick Access'",
      "description": "LaunchTo=1.",
      "category": "explorer",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Comportamiento del Windows clásico."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "LaunchTo",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 2,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "explorer-disable-recent-files",
      "displayName": "No mostrar archivos recientes en Quick Access",
      "description": "Quita la lista 'Recent files'.",
      "category": "privacy",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No aparece historial reciente. Privacidad."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer",
          "name": "ShowRecent",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer",
          "name": "ShowFrequent",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["aggressive"]
    },
    {
      "id": "taskbar-align-left",
      "displayName": "Alinear taskbar a la izquierda",
      "description": "TaskbarAl=0 (estilo Win10).",
      "category": "taskbar",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Iconos alineados a la izquierda."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "TaskbarAl",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "taskbar-hide-search",
      "displayName": "Ocultar barra de búsqueda en taskbar",
      "description": "SearchboxTaskbarMode=0.",
      "category": "taskbar",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Más espacio en taskbar. Win+S sigue abriendo búsqueda."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Search",
          "name": "SearchboxTaskbarMode",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "taskbar-hide-widgets",
      "displayName": "Ocultar Widgets en taskbar",
      "description": "TaskbarDa=0.",
      "category": "taskbar",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Botón de noticias/clima desaparece."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "TaskbarDa",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "taskbar-hide-chat",
      "displayName": "Ocultar Chat/Teams en taskbar",
      "description": "TaskbarMn=0.",
      "category": "taskbar",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Botón de Teams Consumer desaparece. La app sigue instalada si no se elimina."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "TaskbarMn",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "taskbar-hide-task-view",
      "displayName": "Ocultar botón Task View",
      "description": "ShowTaskViewButton=0.",
      "category": "taskbar",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Botón de Task View no aparece. Win+Tab sigue funcionando."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "ShowTaskViewButton",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "context-menu-classic",
      "displayName": "Restaurar menú contextual clásico (Win10)",
      "description": "Elimina el menú reducido de Win11.",
      "category": "context-menu",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["El menú right-click muestra todas las opciones directamente."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\\InprocServer32",
          "name": "(Default)",
          "kind": "string",
          "enabledValue": "",
          "disabledValue": "",
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "performance-disable-startup-delay",
      "displayName": "Quitar delay artificial al startup de apps",
      "description": "StartupDelayInMSec=0.",
      "category": "performance",
      "risk": "medium",
      "requiresAdmin": false,
      "consequences": [
        "Apps de inicio cargan inmediatamente. Puede saturar IO en HDDs.",
        "Recomendado solo en SSD/NVMe."
      ],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Serialize",
          "name": "StartupDelayInMSec",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": ["aggressive"]
    },
    {
      "id": "performance-mouse-hover-time",
      "displayName": "Reducir hover time del mouse (tooltips más rápidos)",
      "description": "MouseHoverTime=10ms.",
      "category": "performance",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Tooltips aparecen casi instantáneo."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Control Panel\\Mouse",
          "name": "MouseHoverTime",
          "kind": "string",
          "enabledValue": "10",
          "disabledValue": "400",
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "performance-menu-show-delay",
      "displayName": "Quitar delay en submenús",
      "description": "MenuShowDelay=0.",
      "category": "performance",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Submenús aparecen sin esperar."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Control Panel\\Desktop",
          "name": "MenuShowDelay",
          "kind": "string",
          "enabledValue": "0",
          "disabledValue": "400",
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "performance-disable-fast-startup",
      "displayName": "Desactivar Fast Startup",
      "description": "HiberbootEnabled=0 — apaga el 'fake shutdown'.",
      "category": "performance",
      "risk": "medium",
      "requiresAdmin": true,
      "consequences": [
        "Cada apagado es real. Boot es ~3-5s más lento.",
        "Resuelve problemas de dispositivos USB que no responden tras restart."
      ],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Power",
          "name": "HiberbootEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["aggressive"]
    },
    {
      "id": "search-disable-bing",
      "displayName": "Desactivar Bing en búsqueda de Start",
      "description": "DisableSearchBoxSuggestions=1.",
      "category": "search",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Búsqueda de Start solo encuentra apps/archivos locales."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Policies\\Microsoft\\Windows\\Explorer",
          "name": "DisableSearchBoxSuggestions",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "search-disable-cortana",
      "displayName": "Desactivar Cortana",
      "description": "AllowCortana=0 a nivel de política.",
      "category": "search",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Cortana deja de aparecer. Búsqueda local sigue funcionando."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\Windows Search",
          "name": "AllowCortana",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "ux-disable-news-and-interests",
      "displayName": "Desactivar News and Interests (Widgets)",
      "description": "EnableFeeds=0.",
      "category": "ux",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Widget de noticias/clima deshabilitado a nivel de máquina."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Dsh",
          "name": "AllowNewsAndInterests",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "ux-classic-volume-mixer",
      "displayName": "Volver al volume mixer clásico",
      "description": "EnableMtcUvc=0.",
      "category": "ux",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Mixer con estilo Windows 7."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "Software\\Microsoft\\Windows NT\\CurrentVersion\\MTCUVC",
          "name": "EnableMtcUvc",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "ux-show-seconds-in-clock",
      "displayName": "Mostrar segundos en el reloj de la taskbar",
      "description": "ShowSecondsInSystemClock=1.",
      "category": "ux",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Reloj muestra HH:MM:SS. Leve aumento de redraws."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "ShowSecondsInSystemClock",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "ux-disable-startup-sound",
      "displayName": "Quitar sonido de inicio de Windows",
      "description": "DisableStartupSound=1.",
      "category": "ux",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Boot silencioso."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\LogonUI\\BootAnimation",
          "name": "DisableStartupSound",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "privacy-disable-clipboard-history",
      "displayName": "Desactivar Clipboard History",
      "description": "AllowClipboardHistory=0.",
      "category": "privacy",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Win+V no muestra historial."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\System",
          "name": "AllowClipboardHistory",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "privacy-disable-online-speech",
      "displayName": "Desactivar Online Speech Recognition",
      "description": "HasAccepted=0 a nivel de SpeechModels.",
      "category": "privacy",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No envía audio a Microsoft para reconocimiento online."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Speech_OneCore\\Settings\\OnlineSpeechPrivacy",
          "name": "HasAccepted",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "privacy-disable-timeline",
      "displayName": "Desactivar Activity History (Timeline)",
      "description": "PublishUserActivities=0 y UploadUserActivities=0.",
      "category": "privacy",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Timeline (Win+Tab historial) deja de poblar."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\System",
          "name": "PublishUserActivities",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\System",
          "name": "UploadUserActivities",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "security-disable-smartscreen-explorer",
      "displayName": "Desactivar SmartScreen para Explorer",
      "description": "EnableSmartScreen=0.",
      "category": "security",
      "risk": "high",
      "requiresAdmin": true,
      "consequences": [
        "PELIGRO: pierdes protección contra ejecutables descargados maliciosos.",
        "Solo desactivar si tenés otra solución antivirus + sabés lo que hacés."
      ],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\System",
          "name": "EnableSmartScreen",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "ux-disable-snap-suggestions",
      "displayName": "Desactivar Snap Assist suggestions",
      "description": "SnapAssist=0.",
      "category": "ux",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Snap sigue funcionando, pero no muestra 'qué otra ventana poner al lado'."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "SnapAssist",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "explorer-compact-mode",
      "displayName": "Compact mode en Explorer",
      "description": "UseCompactMode=1 — filas más densas.",
      "category": "explorer",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Más archivos por pantalla, menos padding."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
          "name": "UseCompactMode",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "telemetry-disable-dialer-suggestions",
      "displayName": "Desactivar 'Suggested apps' en notificaciones",
      "description": "DontShowMeThisDialogAgain.",
      "category": "ads",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No saldrán pop-ups 'Recommended apps for you'."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
          "name": "SubscribedContent-353698Enabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "ads-disable-finish-setup",
      "displayName": "Desactivar 'Let's finish setting up your device'",
      "description": "ScoobeSystemSettingEnabled=0.",
      "category": "ads",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["No salta el wizard tras updates pidiéndote configurar OneDrive, etc."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\UserProfileEngagement",
          "name": "ScoobeSystemSettingEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["minimal", "recommended", "aggressive"]
    },
    {
      "id": "performance-disable-transparency",
      "displayName": "Desactivar transparencia (Mica/Acrylic)",
      "description": "EnableTransparency=0.",
      "category": "performance",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["UI sin blur. Ahorra GPU en máquinas modestas."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
          "name": "EnableTransparency",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "performance-disable-animations",
      "displayName": "Desactivar animaciones del Explorer/Window",
      "description": "VisualFXSetting=2 (best performance).",
      "category": "performance",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["UI menos fluida visualmente, más rápida en máquinas lentas."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
          "name": "VisualFXSetting",
          "kind": "dword",
          "enabledValue": 2,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "explorer-restore-old-search-bar",
      "displayName": "Restaurar barra de búsqueda clásica en Explorer",
      "description": "Workaround de Win11 24H2 — usa el dispatcher viejo.",
      "category": "explorer",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Search en Explorer sin sugerencias online."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Policies\\Microsoft\\Windows\\Explorer",
          "name": "DisableSearchBoxSuggestions",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "performance-disable-game-bar",
      "displayName": "Desactivar Game Bar globalmente",
      "description": "AppCaptureEnabled=0.",
      "category": "performance",
      "risk": "low",
      "requiresAdmin": false,
      "consequences": ["Win+G no abre Game Bar. Ahorra recursos en background."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\GameBar",
          "name": "UseNexusForGameBarEnabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        },
        {
          "hive": "HKCU",
          "key": "System\\GameConfigStore",
          "name": "GameDVR_Enabled",
          "kind": "dword",
          "enabledValue": 0,
          "disabledValue": 1,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "privacy-disable-app-camera",
      "displayName": "Desactivar acceso de apps a la cámara",
      "description": "GlobalUserDisabled=1 a nivel de webcam.",
      "category": "privacy",
      "risk": "high",
      "requiresAdmin": false,
      "consequences": [
        "Apps UWP no pueden acceder a webcam.",
        "Apps Win32 (Zoom, Discord, OBS) NO afectadas por esta key."
      ],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\DeviceAccess\\Global\\{E5323777-F976-4f5b-9B55-B94699C46E44}",
          "name": "Value",
          "kind": "string",
          "enabledValue": "Deny",
          "disabledValue": "Allow",
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "privacy-disable-app-microphone",
      "displayName": "Desactivar acceso de apps al micrófono",
      "description": "Mismo patrón que cámara para micrófono.",
      "category": "privacy",
      "risk": "high",
      "requiresAdmin": false,
      "consequences": ["Apps UWP no pueden acceder al micrófono."],
      "operations": [
        {
          "hive": "HKCU",
          "key": "Software\\Microsoft\\Windows\\CurrentVersion\\DeviceAccess\\Global\\{2EEF81BE-33FA-4800-9670-1CD474972C3F}",
          "name": "Value",
          "kind": "string",
          "enabledValue": "Deny",
          "disabledValue": "Allow",
          "createIfMissing": true
        }
      ],
      "presets": []
    },
    {
      "id": "ux-disable-edge-shortcut-on-desktop",
      "displayName": "Evitar que Edge cree shortcut en escritorio",
      "description": "DisableEdgeDesktopShortcutCreation=1.",
      "category": "ads",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["Tras updates, Edge no recrea su shortcut en el escritorio."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Policies\\Microsoft\\EdgeUpdate",
          "name": "DisableEdgeDesktopShortcutCreation",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    },
    {
      "id": "performance-disable-meet-now",
      "displayName": "Ocultar 'Meet Now' del system tray",
      "description": "HideSCAMeetNow=1.",
      "category": "taskbar",
      "risk": "low",
      "requiresAdmin": true,
      "consequences": ["No aparece el icono de Skype Meet Now."],
      "operations": [
        {
          "hive": "HKLM",
          "key": "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer",
          "name": "HideSCAMeetNow",
          "kind": "dword",
          "enabledValue": 1,
          "disabledValue": 0,
          "createIfMissing": true
        }
      ],
      "presets": ["recommended", "aggressive"]
    }
  ]
}
```

> **Total: 45 entries únicas.** Cubre las categorías más relevantes para una herramienta de debloat/tweaks:
> - **8 telemetría/privacy** core.
> - **5 ads/sugerencias** Start/lockscreen/notificaciones.
> - **7 explorer** UX.
> - **6 taskbar**.
> - **7 performance**.
> - **2 search** (Bing + Cortana).
> - **1 context-menu** clásico.
> - **6 ux** varios.
> - **3 privacy** access devices.

---

## 7. Ampliación del bloatware-catalog.json

> El JSON actual tiene ~20 entries. Faltan apps comunes de OEM + AI nuevas + algunas Store apps.

**Entries a AÑADIR** (no reemplazar — fusionar con las existentes):

```json
[
  {
    "id": "copilot",
    "displayName": "Microsoft Copilot",
    "description": "Asistente AI integrado en Windows 11.",
    "category": "ai",
    "risk": "low",
    "consequences": ["Win+C deja de abrir Copilot. La integración en Edge sigue."],
    "removalMethod": "appx-user",
    "appxPackageFamilyName": "Microsoft.Copilot_8wekyb3d8bbwe",
    "preservesDataByDefault": true,
    "requiresAdmin": false,
    "reversible": true,
    "reverseRecipe": {
      "kind": "appx-reinstall-from-store",
      "storeUrl": "ms-windows-store://pdp/?productid=9NHT9RB2F4HD"
    },
    "minWindowsBuild": 22631,
    "presets": ["recommended", "total"]
  },
  {
    "id": "bing-news",
    "displayName": "Bing News",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Sin app de noticias preinstalada."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.BingNews",
    "presets": ["recommended", "total"]
  },
  {
    "id": "bing-weather",
    "displayName": "Bing Weather",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Sin app de clima preinstalada."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.BingWeather",
    "presets": ["recommended", "total"]
  },
  {
    "id": "feedback-hub",
    "displayName": "Feedback Hub",
    "category": "telemetry",
    "risk": "low",
    "consequences": ["No podés enviar feedback a Microsoft via app."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.WindowsFeedbackHub",
    "presets": ["recommended", "total"]
  },
  {
    "id": "getstarted",
    "displayName": "Get Started / Tips",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Sin tour inicial."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.Getstarted",
    "presets": ["minimal", "recommended", "total"]
  },
  {
    "id": "people",
    "displayName": "People",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["App People desaparece."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.People",
    "presets": ["recommended", "total"]
  },
  {
    "id": "skype",
    "displayName": "Skype",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Si usás Skype, no instalar este preset."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.SkypeApp",
    "presets": ["recommended", "total"]
  },
  {
    "id": "wallet",
    "displayName": "Wallet",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Ninguna."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.Wallet",
    "presets": ["recommended", "total"]
  },
  {
    "id": "yourphone",
    "displayName": "Phone Link (Your Phone)",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Si conectás teléfono Android via Phone Link, no remover."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.YourPhone",
    "presets": ["total"]
  },
  {
    "id": "mixedrealityportal",
    "displayName": "Mixed Reality Portal",
    "category": "deprecated",
    "risk": "low",
    "consequences": ["Ninguna salvo que uses cascos WMR."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.MixedReality.Portal",
    "presets": ["recommended", "total"]
  },
  {
    "id": "3dviewer",
    "displayName": "3D Viewer",
    "category": "deprecated",
    "risk": "low",
    "consequences": ["Ninguna."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.Microsoft3DViewer",
    "presets": ["minimal", "recommended", "total"]
  },
  {
    "id": "paint3d",
    "displayName": "Paint 3D",
    "category": "deprecated",
    "risk": "low",
    "consequences": ["Paint clásico sigue funcionando."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.MSPaint",
    "presets": ["minimal", "recommended", "total"]
  },
  {
    "id": "office-hub",
    "displayName": "Office Hub",
    "category": "ms-consumer",
    "risk": "low",
    "consequences": ["Si usás Microsoft 365, esta app reaparece tras login."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Microsoft.MicrosoftOfficeHub",
    "presets": ["recommended", "total"]
  },
  {
    "id": "candy-crush",
    "displayName": "Candy Crush (preinstalada)",
    "category": "consumer-app",
    "risk": "low",
    "consequences": ["Ninguna."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "king.com.CandyCrushSaga",
    "presets": ["minimal", "recommended", "total"]
  },
  {
    "id": "spotify-preinstall",
    "displayName": "Spotify (preinstalada)",
    "category": "third-party-oem",
    "risk": "low",
    "consequences": ["Si querés Spotify, instalar desde spotify.com."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "SpotifyAB.SpotifyMusic",
    "presets": ["recommended", "total"]
  },
  {
    "id": "instagram-preinstall",
    "displayName": "Instagram (preinstalada)",
    "category": "third-party-oem",
    "risk": "low",
    "consequences": ["Ninguna."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "Facebook.InstagramBeta",
    "presets": ["minimal", "recommended", "total"]
  },
  {
    "id": "tiktok-preinstall",
    "displayName": "TikTok (preinstalada)",
    "category": "third-party-oem",
    "risk": "low",
    "consequences": ["Ninguna."],
    "removalMethod": "appx-provisioned",
    "appxProvisionedName": "BytedancePte.Ltd.TikTok",
    "presets": ["minimal", "recommended", "total"]
  },
  {
    "id": "edge-pwa-installs",
    "displayName": "Edge PWAs preinstaladas",
    "description": "Pinned PWAs como ESPN, Booking.com instaladas por edge://apps.",
    "category": "edge-component",
    "risk": "low",
    "consequences": ["Atajos PWA en Start menu desaparecen."],
    "removalMethod": "service-and-files",
    "preservesDataByDefault": true,
    "presets": ["recommended", "total"]
  }
]
```

---

## 8. Implementación Rust — validación y carga

### 8.1 Fase 1 — Mover catálogos a `src-tauri/resources/catalogs/`

#### Paso 8.1.1 — Crear estructura

```
src-tauri/
  resources/
    catalogs/
      cache-locations.json
      cache-locations.schema.json
      bloatware-catalog.json
      bloatware-catalog.schema.json
      services-catalog.json
      services-catalog.schema.json
      registry-tweaks.json
      registry-tweaks.schema.json
```

#### Paso 8.1.2 — Script de sincronización (`scripts/sync-catalogs.ps1`)

```powershell
$ErrorActionPreference = "Stop"

$catalogs = @(
    @{ src = ".claude/skills/cache-scanner/RESOURCES/cache-locations.json";        dst = "src-tauri/resources/catalogs/cache-locations.json" },
    @{ src = ".claude/skills/cache-scanner/RESOURCES/cache-locations.schema.json"; dst = "src-tauri/resources/catalogs/cache-locations.schema.json" },
    @{ src = ".claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json"; dst = "src-tauri/resources/catalogs/bloatware-catalog.json" },
    @{ src = ".claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.schema.json"; dst = "src-tauri/resources/catalogs/bloatware-catalog.schema.json" },
    @{ src = ".claude/skills/powershell-debloat/RESOURCES/services-catalog.json"; dst = "src-tauri/resources/catalogs/services-catalog.json" },
    @{ src = ".claude/skills/powershell-debloat/RESOURCES/services-catalog.schema.json"; dst = "src-tauri/resources/catalogs/services-catalog.schema.json" },
    @{ src = ".claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.json"; dst = "src-tauri/resources/catalogs/registry-tweaks.json" },
    @{ src = ".claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.schema.json"; dst = "src-tauri/resources/catalogs/registry-tweaks.schema.json" }
)

New-Item -ItemType Directory -Force "src-tauri/resources/catalogs" | Out-Null

foreach ($c in $catalogs) {
    if (Test-Path $c.src) {
        Copy-Item $c.src $c.dst -Force
        Write-Host "OK $($c.src) -> $($c.dst)"
    } else {
        Write-Warning "MISSING $($c.src)"
    }
}
```

#### Paso 8.1.3 — Hook en `build.rs`

```rust
// Al principio de build.rs:
fn main() {
    println!("cargo:rerun-if-changed=resources/catalogs/");
    sync_catalogs();
    // ... el resto del build.rs existente
}

fn sync_catalogs() {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-File", "../scripts/sync-catalogs.ps1"])
        .output();
    match out {
        Ok(o) if o.status.success() => {},
        Ok(o) => println!("cargo:warning=Catalog sync failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => println!("cargo:warning=Catalog sync error: {}", e),
    }
}
```

### 8.2 Fase 2 — `domain::catalog` refactor con `include_str!`

**Archivo:** `src-tauri/src/domain/catalog.rs`

```rust
use crate::core::{AppError, AppResult};
use crate::models::cache::CacheLocation;
use crate::models::debloat::BloatwareEntry;
use crate::models::registry::RegistryTweak;
use crate::models::service::ServiceEntry;

// Embebidos en compile-time. El build.rs garantiza que están en su sitio.
const CACHE_LOCATIONS_JSON: &str = include_str!("../../resources/catalogs/cache-locations.json");
const BLOATWARE_CATALOG_JSON: &str = include_str!("../../resources/catalogs/bloatware-catalog.json");
const SERVICES_CATALOG_JSON: &str = include_str!("../../resources/catalogs/services-catalog.json");
const REGISTRY_TWEAKS_JSON: &str = include_str!("../../resources/catalogs/registry-tweaks.json");

#[derive(serde::Deserialize)]
struct CatalogWrapper<T> {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    entries: Vec<T>,
}

fn parse<T: serde::de::DeserializeOwned>(json: &str, name: &str, expected_version: u32) -> AppResult<Vec<T>> {
    let wrapper: CatalogWrapper<T> = serde_json::from_str(json)
        .map_err(|e| AppError::Catalog(format!("[{}] parse error: {}", name, e)))?;
    if wrapper.schema_version != expected_version {
        return Err(AppError::Catalog(format!(
            "[{}] expected schemaVersion {}, got {}", name, expected_version, wrapper.schema_version
        )));
    }
    Ok(wrapper.entries)
}

pub fn load_cache_locations() -> AppResult<Vec<CacheLocation>> {
    parse(CACHE_LOCATIONS_JSON, "cache-locations", 1)
}

pub fn load_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    parse(BLOATWARE_CATALOG_JSON, "bloatware-catalog", 1)
}

pub fn load_services_catalog() -> AppResult<Vec<ServiceEntry>> {
    parse(SERVICES_CATALOG_JSON, "services-catalog", 1)
}

pub fn load_registry_tweaks() -> AppResult<Vec<RegistryTweak>> {
    parse(REGISTRY_TWEAKS_JSON, "registry-tweaks", 1)
}

/// Llamado al arrancar la app — verifica que todos los catálogos parsean correctamente.
/// Si alguno falla, panic con mensaje claro (es bug de build, no de runtime).
pub fn validate_all_at_startup() {
    for r in [
        load_cache_locations().map(|_| "cache-locations"),
        load_bloatware_catalog().map(|_| "bloatware-catalog"),
        load_services_catalog().map(|_| "services-catalog"),
        load_registry_tweaks().map(|_| "registry-tweaks"),
    ] {
        match r {
            Ok(name) => log::info!("catalog ok: {}", name),
            Err(e) => panic!("catalog load failed: {}", e),
        }
    }
}
```

### 8.3 Fase 3 — Models que faltan

#### Paso 8.3.1 — `models/service.rs` ampliar

Reemplazar el actual `Service` struct (que es runtime state) por dos structs separados:

```rust
// models/service.rs

use serde::{Deserialize, Serialize};

/// Catálogo curado — viene del JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEntry {
    pub service_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub risk: String,
    pub default_start_type: String,
    pub recommended_start_type: Option<String>,
    #[serde(default)]
    pub consequences: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub needed_by: Vec<String>,
    pub min_windows_build: Option<u32>,
    #[serde(default)]
    pub presets: Vec<String>,
}

/// Estado runtime — viene del SCM, no del catálogo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    pub display_name: String,
    pub state: String,        // Running, Stopped, ...
    pub start_type: String,   // Automatic, Manual, Disabled, ...
    pub description: Option<String>,
    /// True si el servicio aparece en el catálogo (es seguro tunearlo).
    pub in_catalog: bool,
    /// Si in_catalog, datos del catálogo.
    pub catalog: Option<ServiceEntry>,
}
```

#### Paso 8.3.2 — `models/registry.rs` ampliar

```rust
// models/registry.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTweak {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub risk: String,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub operations: Vec<RegistryOp>,
    #[serde(default)]
    pub presets: Vec<String>,
    pub min_windows_build: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryOp {
    pub hive: String,          // HKLM | HKCU | HKCR | HKU
    pub key: String,
    pub name: String,
    pub kind: String,          // dword | qword | string | expand-string | multi-string | binary
    pub enabled_value: serde_json::Value,
    pub disabled_value: serde_json::Value,
    #[serde(default = "default_create_if_missing")]
    pub create_if_missing: bool,
}

fn default_create_if_missing() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: String,
    pub enabled: bool,
    pub partial: bool,         // alguna operación enabled, otra no
    pub current_values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTweakInput {
    pub id: String,
    pub enable: bool,
    pub dry_run: bool,
}
```

### 8.4 Fase 4 — Allowlists derivadas en runtime

Para que **cualquier intento de tocar algo fuera del catálogo aborte**, exponer helpers:

```rust
// domain/catalog.rs (añadir al final)

use std::collections::HashSet;
use std::sync::OnceLock;

static SERVICE_ALLOWLIST: OnceLock<HashSet<String>> = OnceLock::new();
static REGISTRY_KEY_ALLOWLIST: OnceLock<HashSet<String>> = OnceLock::new();
static BLOATWARE_ALLOWLIST: OnceLock<HashSet<String>> = OnceLock::new();
static CACHE_ID_ALLOWLIST: OnceLock<HashSet<String>> = OnceLock::new();

pub fn is_service_allowed(name: &str) -> bool {
    SERVICE_ALLOWLIST
        .get_or_init(|| {
            load_services_catalog()
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.service_name.to_lowercase())
                .collect()
        })
        .contains(&name.to_lowercase())
}

pub fn is_registry_key_allowed(hive: &str, key: &str) -> bool {
    let needle = format!("{}\\{}", hive.to_uppercase(), key.to_lowercase());
    REGISTRY_KEY_ALLOWLIST
        .get_or_init(|| {
            load_registry_tweaks()
                .unwrap_or_default()
                .iter()
                .flat_map(|t| t.operations.iter())
                .map(|op| format!("{}\\{}", op.hive.to_uppercase(), op.key.to_lowercase()))
                .collect()
        })
        .iter()
        .any(|allowed| needle.starts_with(allowed))
}

pub fn is_bloatware_id_allowed(id: &str) -> bool {
    BLOATWARE_ALLOWLIST
        .get_or_init(|| {
            load_bloatware_catalog()
                .unwrap_or_default()
                .into_iter()
                .map(|e| e.id)
                .collect()
        })
        .contains(id)
}

pub fn is_cache_id_allowed(id: &str) -> bool {
    CACHE_ID_ALLOWLIST
        .get_or_init(|| {
            load_cache_locations()
                .unwrap_or_default()
                .into_iter()
                .map(|c| c.id)
                .collect()
        })
        .contains(id)
}
```

---

## 9. Tests

### 9.1 Test de parsing de los 4 catálogos

**Archivo:** `src-tauri/tests/catalogs.rs` (nuevo)

```rust
//! Tests que validan que los 4 catálogos parsean correctamente con el schema esperado.
//! Si alguno falla, el binario no debe poder buildear (catalog::validate_all_at_startup hace panic).

use cleartool::domain::catalog;

#[test]
fn cache_locations_parsea() {
    let entries = catalog::load_cache_locations().expect("cache-locations parse OK");
    assert!(entries.len() >= 10, "esperado >=10 entries, got {}", entries.len());
}

#[test]
fn bloatware_catalog_parsea() {
    let entries = catalog::load_bloatware_catalog().expect("bloatware-catalog parse OK");
    assert!(entries.len() >= 20);
}

#[test]
fn services_catalog_parsea() {
    let entries = catalog::load_services_catalog().expect("services-catalog parse OK");
    assert!(entries.len() >= 40);
    let names: std::collections::HashSet<_> = entries.iter().map(|s| s.service_name.clone()).collect();
    assert_eq!(names.len(), entries.len(), "service_name duplicados detectados");
}

#[test]
fn registry_tweaks_parsea() {
    let entries = catalog::load_registry_tweaks().expect("registry-tweaks parse OK");
    assert!(entries.len() >= 30);
    let ids: std::collections::HashSet<_> = entries.iter().map(|t| t.id.clone()).collect();
    assert_eq!(ids.len(), entries.len(), "tweak id duplicados detectados");
}

#[test]
fn registry_tweaks_hives_validos() {
    let entries = catalog::load_registry_tweaks().expect("ok");
    for e in entries {
        for op in e.operations {
            assert!(
                matches!(op.hive.as_str(), "HKLM" | "HKCU" | "HKCR" | "HKU"),
                "hive inválido: {} en tweak {}", op.hive, e.id
            );
        }
    }
}

#[test]
fn risk_es_lowercase() {
    let s = catalog::load_services_catalog().unwrap();
    for entry in s {
        assert!(matches!(entry.risk.as_str(), "low" | "medium" | "high"),
            "risk inválido en service {}: {}", entry.service_name, entry.risk);
    }
    let r = catalog::load_registry_tweaks().unwrap();
    for entry in r {
        assert!(matches!(entry.risk.as_str(), "low" | "medium" | "high"),
            "risk inválido en tweak {}: {}", entry.id, entry.risk);
    }
}
```

### 9.2 Test JSON Schema (opcional pero recomendado)

```toml
# Cargo.toml dev-dependencies
jsonschema = "0.18"
```

```rust
#[test]
fn cache_locations_cumple_schema() {
    let schema_str = include_str!("../resources/catalogs/cache-locations.schema.json");
    let data_str = include_str!("../resources/catalogs/cache-locations.json");
    let schema: serde_json::Value = serde_json::from_str(schema_str).unwrap();
    let data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let compiled = jsonschema::JSONSchema::compile(&schema).unwrap();
    let result = compiled.validate(&data);
    if let Err(errors) = result {
        for e in errors {
            eprintln!("schema violation: {}", e);
        }
        panic!("cache-locations.json no cumple schema");
    }
}
```

Replicar para los 4 catálogos.

---

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Un build queda con catálogo malformado | `validate_all_at_startup` hace panic. Tests de parsing en CI. |
| Catálogo crece sin curaduría → entries inseguras | PR review obligatorio cuando se modifica `resources/catalogs/`. |
| Drift entre `.claude/skills/` y `src-tauri/resources/` | Script `sync-catalogs.ps1` corre en cada build. CI verifica que ambos coinciden. |
| Schema cambia y rompe deserialize en clientes viejos | `schemaVersion` ya está en el wrapper. Bump = breaking. |
| El path en `cache-locations.path` contiene env var inexistente | `expand_path` en `domain::cache` deja el token literal; el escaneo después detecta `path.exists() == false` y reporta `missing`. |

---

## 11. Definition of Done

- [ ] Los 4 schemas JSON existen en `.claude/skills/...` y en `src-tauri/resources/catalogs/`.
- [ ] Los 4 catálogos JSON existen y parsean.
- [ ] `domain::catalog` carga los 4 via `include_str!`.
- [ ] `validate_all_at_startup` es llamado en `lib.rs::run`.
- [ ] `is_service_allowed`, `is_registry_key_allowed`, `is_bloatware_id_allowed`, `is_cache_id_allowed` existen y se usan en los comandos destructivos.
- [ ] `cargo test -p cleartool catalogs` pasa los 5+ tests.
- [ ] `scripts/sync-catalogs.ps1` existe y se integra en `build.rs`.
- [ ] `models/service.rs` separa `Service` (runtime) de `ServiceEntry` (catálogo).
- [ ] `models/registry.rs` define `RegistryTweak`, `RegistryOp`, `TweakState`, `ApplyTweakInput`.
- [ ] Commit con mensaje `feat(catalogs): cuatro catálogos curados embebidos + validación`.

---

## 12. Próximo archivo

→ [01-RESTORE-POINTS.md](01-RESTORE-POINTS.md) — implementar el sistema de puntos de restauración, que es prerequisito de cualquier operación destructiva en los módulos siguientes.
