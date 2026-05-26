# v2 · 04 — Debloat Catalog Expansion (P1)

## Estado actual (v0.1)

El catálogo `bloatware-catalog.json` tiene **11 entradas**:
- Microsoft News, MSN Weather, Microsoft Copilot
- Widgets, Microsoft Teams (consumer), OneDrive
- Microsoft Edge, Microsoft Store
- Connected User Experiences and Telemetry, etc.

Un Windows 11 OEM típico (Lenovo, HP, Dell, Asus) tiene fácilmente **60+ candidatos** a debloat que la v0.1 no cubre. El usuario abre ClearTool, ve 11 apps, y piensa "esto no es lo que necesito".

## Objetivo v1.0

Catálogo de **~120 entradas curadas** organizadas en **8 categorías**, cada una con:
- Información clara de qué hace.
- Riesgo realista (no todo es "low risk").
- Consecuencias detalladas (qué deja de funcionar).
- Método de reversa.
- Build mínimo de Windows requerido (algunas apps cambian de nombre entre 22H2 y 23H2).
- Lista de presets a los que pertenece (`minimal`, `recommended`, `total`, `gaming`, `privacy-paranoid`).

## Estructura de categorías

```
1. ms-consumer-app       — Apps consumer de Microsoft  (Solitaire, Mahjong, Skype, etc.)
2. ms-system-app          — Apps "core" desinstalables (Photos, Maps, Voice Recorder)
3. ai-and-telemetry       — Copilot, ConnectedUserExperiencesAndTelemetry, Cortana
4. ui-clutter             — Widgets, Chat, Edge dock, Mail+Calendar tile
5. oem-bloatware          — McAfee, Norton trial, HP Wolf Security, Lenovo Vantage, Dell SupportAssist, ASUS Armoury Crate
6. games                  — Xbox Game Bar, Xbox Live, Solitaire, Candy Crush, Disney Magic Kingdoms
7. third-party-bundled    — Spotify, LinkedIn, Disney+, TikTok, Instagram (vienen preinstalados)
8. dev-tools-unused       — WSL si no se usa, Hyper-V para no power-users, Windows Subsystem for Linux GUI
```

## Estructura de entrada (schema v2)

```json
{
  "id": "ms-xbox-game-bar",
  "displayName": "Xbox Game Bar",
  "description": "Overlay de juego (Win+G). Captura, FPS, chat con amigos Xbox.",
  "category": "games",
  "risk": "low",
  "consequences": [
    "Atajo Win+G deja de funcionar.",
    "No se pueden grabar clips desde el overlay.",
    "Algunos juegos que detectan Game Bar para el modo Game Mode pueden tardar más en aplicar optimizaciones."
  ],
  "removalMethod": "appx-user-and-provisioned",
  "appxPackageFamilyName": "Microsoft.XboxGamingOverlay_8wekyb3d8bbwe",
  "appxProvisionedName": "Microsoft.XboxGamingOverlay",
  "uninstallRegistryPath": null,
  "preservesDataByDefault": true,
  "requiresAdmin": true,
  "reversible": true,
  "reverseRecipe": {
    "kind": "appxReinstall",
    "packageFamilyName": "Microsoft.XboxGamingOverlay_8wekyb3d8bbwe",
    "storeUrl": "ms-windows-store://pdp/?ProductId=9NZKPSTSNW4P"
  },
  "minWindowsBuild": 22000,
  "maxWindowsBuild": null,
  "presets": ["recommended", "total", "gaming"],
  "alternativeApps": ["OBS Studio", "ShadowPlay"],
  "tags": ["xbox", "overlay", "gaming", "Win+G"],
  "iconAppx": true,
  "verified": "2026-05-25",
  "verifiedBy": "@joelsanchez"
}
```

Campos nuevos respecto a v0.1:
- `description` — texto corto para el panel de detalles.
- `maxWindowsBuild` — algunas apps son legacy y no existen en 24H2.
- `presets` — array de presets a los que pertenece (en lugar del bool actual).
- `alternativeApps` — sugerencias del propio ClearTool ("usa OBS en lugar de Game Bar para grabar").
- `tags` — para búsqueda.
- `verified` / `verifiedBy` — qué humano validó esta entrada en qué fecha.

## Estrategias de eliminación expandidas

```typescript
type RemovalMethod =
  | "appx-user"                  // sólo el usuario actual
  | "appx-provisioned"           // del image (afecta nuevos usuarios)
  | "appx-user-and-provisioned"  // ambos
  | "uninstaller-string"         // ejecuta string del registry Uninstall
  | "service-and-files"          // detener servicio + borrar exes + policies
  | "service-stop-only"          // detener servicio, no borrar nada
  | "registry-policy"            // sólo escribe policy keys
  | "scheduled-task-disable"     // deshabilitar tareas programadas asociadas
  | "compound";                  // ejecuta varias removalMethods en secuencia
```

Para `compound`:

```json
{
  "id": "ms-copilot-total",
  "displayName": "Copilot — eliminación total",
  "removalMethod": "compound",
  "compoundSteps": [
    { "method": "appx-user", "appxPackageFamilyName": "Microsoft.Copilot_..." },
    { "method": "registry-policy", "writes": [
      { "hive": "HKLM", "key": "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsCopilot",
        "name": "TurnOffWindowsCopilot", "type": "REG_DWORD", "value": 1 }
    ]},
    { "method": "scheduled-task-disable", "taskName": "\\Microsoft\\Windows\\Copilot\\..." }
  ]
}
```

## Curación de entradas: lista priorizada

### Tier 1 — Imprescindibles (50 entradas)
Lo que CUALQUIER Windows 11 OEM tiene y que el power user normalmente quita:

**ms-consumer-app:**
- Microsoft Solitaire Collection, Mahjong, Sticky Notes (si no se usa), Microsoft To Do, Whiteboard, Office Hub, Get Help, Tips, Feedback Hub, Microsoft Wallet, Microsoft 365 (LinkedIn included).

**ms-system-app (cuidado, algunos son útiles):**
- 3D Viewer (si nadie lo usa), Mixed Reality Portal, Quick Assist, Movies & TV, Groove Music, Voice Recorder, Camera (si no hay webcam), Maps, Mail and Calendar, People, Alarms & Clock, Power Automate Desktop, Paint 3D.

**ai-and-telemetry:**
- Copilot (UI + service), Cortana, ConnectedUserExperiencesAndTelemetry, dmwappushservice, DiagTrack service, Customer Experience Improvement Program scheduled tasks.

**ui-clutter:**
- Widgets (WebExperiencePack), Chat (Teams consumer), News widget, Search box → solo icono, Edge dock icons, Microsoft Edge desktop shortcut.

**games (M$ first-party):**
- Xbox app, Xbox Game Bar, Xbox Live, Xbox Identity Provider, Solitaire, Minecraft for Windows trial.

### Tier 2 — OEM bloatware (40 entradas)
Detectado por presencia y eliminado:

**Lenovo:** Vantage, Lenovo Smart Communication, McAfee LiveSafe (trial), Lenovo Welcome.
**HP:** HP Wolf Security, HP JumpStart, HP Audio Switch, HP Customer Experience, HP Connection Optimizer, HP Documentation, HP Smart, HP System Event Utility, HP Sure Apps.
**Dell:** SupportAssist, Dell Optimizer, Dell Mobile Connect, Dell Customer Connect, Dell Power Manager.
**Asus:** Armoury Crate, MyAsus, ASUS GiftBox.
**Acer:** Care Center, Quick Access, Configuration Manager.
**Samsung:** Samsung Notes, Samsung Settings.
**Razer:** Razer Synapse (si no usa periféricos Razer), Razer Cortex.

### Tier 3 — Third-party preinstalados (20 entradas)
Apps que vienen desde la imagen del fabricante o vía Microsoft Marketing Suite:

Spotify, LinkedIn, Disney+, TikTok, Instagram, WhatsApp Desktop (UWP), Twitter, Netflix, Adobe Creative Cloud (trial), Booking.com, ESPN, Amazon Prime Video.

### Tier 4 — Avanzado (10 entradas)
Para power users, con risk medium/high:

- Windows Subsystem for Linux (si no se usa).
- Hyper-V (libera RAM y CPU).
- WindowsBackup (la nueva forzada).
- BingWeather.
- StorageSpaces (si no se usa).
- MixedReality / Holographic.

## Algoritmo de detección mejorado

### Detección por múltiples métodos
Hoy en día, `detect_installed_bloatware` sólo comprueba `Get-AppxPackage`. Eso falla con:
- Apps no-UWP (Adobe trial, McAfee).
- Servicios disfrazados de "no-app".

v2 combina:

```rust
async fn detect_installed_v2() -> AppResult<Vec<DetectedPackage>> {
    let cat = load_bloatware_catalog()?;
    let appx = list_appx().await?;
    let registry_uninstallers = list_registry_uninstallers().await?;
    let services = list_services().await?;
    let scheduled_tasks = list_scheduled_tasks().await?;

    let mut detected = Vec::new();
    for entry in cat {
        match entry.removal_method.as_str() {
            "appx-user" | "appx-provisioned" | "appx-user-and-provisioned" => {
                if let Some(p) = appx.user.iter().find(|p| p.package_family_name == entry.appx_pfn) {
                    detected.push(DetectedPackage::from_appx(p, &entry));
                }
            }
            "uninstaller-string" => {
                let matches = match_uninstaller(&registry_uninstallers, &entry);
                if !matches.is_empty() {
                    detected.push(DetectedPackage::from_uninstaller(matches, &entry));
                }
            }
            "service-and-files" | "service-stop-only" => {
                if entry.services.iter().any(|s| services.contains(s)) {
                    detected.push(DetectedPackage::from_service(&entry));
                }
            }
            _ => {}
        }
    }
    Ok(detected)
}
```

### Match flexible para uninstaller-string

OEM bloatware como "HP Wolf Security" aparece en el registro con nombres como `HP Wolf Security - 1.0.0.123` (con versión). El catálogo necesita un `displayNamePattern` con regex/wildcards:

```json
{
  "id": "hp-wolf-security",
  "removalMethod": "uninstaller-string",
  "displayNamePattern": "HP Wolf Security.*",
  "publisher": "HP Inc."
}
```

## Reverse recipes para los nuevos

Cada entry **necesita** una reverse recipe. Las opciones son:

```typescript
type ReverseRecipe =
  | { kind: "appxReinstall", packageFamilyName: string, storeUrl: string | null }
  | { kind: "uninstallerRollback", scriptToRecover: string }
  | { kind: "registry", operations: RegistryRevertOp[] }
  | { kind: "service", serviceName: string, previousStartType: string, previousState: string }
  | { kind: "noop", reason: string };   // ← documentación: por qué no es reversible
```

**Regla**: si una entrada tiene `kind: "noop"`, el `reason` debe explicar qué hacer manualmente.

Ejemplo:
```json
"reverseRecipe": {
  "kind": "noop",
  "reason": "HP Wolf Security se reinstala descargando el instalador desde support.hp.com → buscar 'HP Wolf Security Console' → seleccionar tu modelo."
}
```

## Disclaimers obligatorios (mostrar antes de aplicar)

Algunos packages tienen consecuencias dramáticas. Mostrar modal con confirmación:

| Package | Disclaimer |
|---------|------------|
| Microsoft Store | "Sin Store, no podrás reinstalar Appx fácilmente. Recomendamos NO eliminar." |
| Microsoft Edge | "Apps con WebView2 pueden romperse. Outlook PWA, algunos instaladores, etc." |
| OneDrive | "Archivos en %USERPROFILE%\OneDrive seguirán ahí pero sin sync." |
| Cortana | "Búsqueda de Windows puede degradarse en builds antiguas." |
| HP Wolf Security | "Sin antivirus tras desinstalar — Defender se activa automáticamente." |

Implementación: campo `disclaimerLevel: "info" | "warning" | "danger"` y `disclaimerText: string` en cada entry.

## Schema validation

`bloatware-catalog.schema.json` valida estrictamente:
- IDs únicos
- Slugs en kebab-case
- Risk en `["low", "medium", "high"]`
- Category en el enum de 8 categorías
- ReverseRecipe presente y válida
- `verified` formato ISO date

CI corre `ajv` contra el schema en cada PR que toque el catálogo.

## Localización

Cada entry tiene campos `displayName`, `description`, `consequences[]` en **español** por defecto, con override `displayName_en`, `description_en`, etc. para inglés (futuro i18n).

## Source of truth para el catálogo

El catálogo embebido (`include_str!`) sigue siendo la fuente de verdad. PERO el sistema permite cargar `bloatware-catalog.user.json` desde `%APPDATA%\ClearTool\catalogs\` para:
- Añadir entradas custom (poweruser).
- Override entries existentes (corrección rápida sin recompilar).

Merge order: user > embebido. Las entradas user se marcan con badge "Custom" en la UI.

## Verificación contra Microsoft Learn

Cada entrada apunta a su documentación oficial cuando exista:

```json
"learnMoreUrl": "https://learn.microsoft.com/en-us/windows/application-management/remove-provisioned-apps-during-update"
```

UI: enlace "ⓘ Más info" → abre browser con ese URL.

## Catálogo en CI

GitHub Actions corre dos checks:
1. Schema validation con AJV.
2. Smoke test: en una VM Windows 11 clean, `detect_installed_bloatware` corre y NO crashea. Los resultados se guardan como artefacto JSON.

## Criterio de "done"

- [ ] 120+ entradas curadas en `bloatware-catalog.json`.
- [ ] Schema actualizado y validado en CI.
- [ ] Detection cubre los 5 métodos (appx user, appx provisioned, uninstaller-string, service, scheduled-task).
- [ ] Reverse recipes específicas para cada entrada (no "noop genérico").
- [ ] Disclaimers UI implementados para los packages destructivos.
- [ ] Smoke test en VM Win 11 OEM (Lenovo / HP) detecta >40 de las 120.
- [ ] `bloatware-catalog.user.json` se carga y mergea correctamente.

## Trabajo de curación (tarea manual, multi-sesión)

Esta es la única spec que requiere **trabajo humano** intensivo — no se puede generar. Plan:

1. Spin up VM Win 11 22H2 OEM (cualquier ISO Lenovo/HP fácil de conseguir).
2. Listar todos los packages con `Get-AppxPackage | ft Name, PackageFamilyName`.
3. Listar uninstallers con `Get-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\* | Select DisplayName, Publisher`.
4. Para cada candidato, decidir tier (1/2/3/4), riesgo, y consecuencias.
5. Documentar reverse recipe (idealmente probarla en la VM).
6. Mergear con el catálogo existente.

Estimación: 16-24h de trabajo de curación humana. No se puede automatizar.
