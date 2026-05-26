# Paso 01 — Definir los 3 presets en JSON

**Área**: 09-privacy-hardening
**Tiempo estimado**: 3-4 horas (datos)
**Dependencias**: ninguna

## Qué hacemos

Crear un archivo JSON que define qué hace cada nivel de privacidad (`Balanced`, `Strict`, `Paranoid`), agregando IDs de registry tweaks + services + debloat entries existentes.

## Archivo nuevo

`.claude/skills/windows-registry-ops/RESOURCES/privacy-presets.json`

## Cómo

### Schema

```json
{
  "schemaVersion": 1,
  "presets": {
    "balanced": {
      "displayName": "Equilibrado",
      "description": "Quita publicidad y telemetría no esencial. No rompe nada.",
      "registryTweaks": ["tweak-id-1", "tweak-id-2"],
      "services": [
        { "name": "DiagTrack", "startType": "Disabled" }
      ],
      "scheduledTasks": ["\\Microsoft\\Windows\\..."],
      "debloatEntries": ["ms-copilot", "ms-news-widget"]
    },
    "strict": { ... },
    "paranoid": { ... }
  }
}
```

### Contenido por preset

#### Balanced (~15 cambios)

**Registry tweaks**:
- Advertising ID off (`disable-advertising-id`)
- App suggestions off (`disable-app-suggestions`)
- Tailored experiences off (`disable-tailored-experiences`)
- Lock screen tips off (`disable-lockscreen-spotlight`)
- Search Bing off (`disable-search-bing`)
- Telemetry = 1 (basic) — no a 0, eso rompe Update; `set-telemetry-basic`
- Widgets off (`disable-widgets-button`)

**Services**:
- DiagTrack → Disabled
- dmwappushsvc → Disabled

**Scheduled tasks** (3-5):
- `\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser`
- `\Microsoft\Windows\Customer Experience Improvement Program\Consolidator`
- `\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip`

**Debloat**:
- Copilot
- News widget package
- Office Hub (Microsoft 365 hub)

#### Strict (~25 cambios)

Todo lo de Balanced más:
- Activity history off
- Cortana off (registry + appx)
- Search box → solo icono
- WebSearch off
- Cloud delivered protection notification off
- Telemetry = 0 (security only — pero WARNING: bloquea Insider builds y algunas updates) — preset `set-telemetry-security`
- Edge dock icons off
- Más scheduled tasks (telemetry-related)
- Bing Weather appx

#### Paranoid (~40 cambios)

Todo lo de Strict más:
- SmartScreen reducido (WARNING — peor protección anti-malware)
- WindowsErrorReporting → Disabled
- WerSvc → Disabled
- All Customer Experience scheduled tasks → Disabled
- Maps activity tracking off
- Location service → Disabled
- Microphone access reducido (sólo apps que explícitamente lo pidan)
- Biometrics off (si no se usa Windows Hello)
- More appx debloat (Movies & TV, Maps, Mail, etc.)
- File explorer telemetry off
- AutoLogger directory disable

### Disclaimers obligatorios por nivel

```json
"strict": {
  "disclaimer": "Algunas búsquedas web pueden dejar de funcionar integradas. Cortana se desactivará completamente.",
  ...
},
"paranoid": {
  "disclaimer": "Modo agresivo. Lee con cuidado:\n- SmartScreen reduce protección anti-malware.\n- Insider builds y algunas updates pueden no llegar.\n- Windows Hello biométrico se desactiva.\n- Algunas apps cloud pueden no funcionar.\nSe creará restore point antes.",
  ...
}
```

### Ejemplo completo de archivo

```json
{
  "schemaVersion": 1,
  "presets": {
    "balanced": {
      "displayName": "Equilibrado",
      "description": "Quita publicidad y telemetría no esencial. No rompe nada visible.",
      "disclaimer": "Recomendado. Sin consecuencias funcionales.",
      "estimatedChanges": 15,
      "registryTweaks": [
        "disable-advertising-id",
        "disable-app-suggestions",
        "disable-tailored-experiences",
        "disable-lockscreen-spotlight",
        "disable-search-bing",
        "set-telemetry-basic",
        "disable-widgets-button"
      ],
      "services": [
        { "name": "DiagTrack", "startType": "Disabled" },
        { "name": "dmwappushsvc", "startType": "Disabled" }
      ],
      "scheduledTasks": [
        "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip"
      ],
      "debloatEntries": [
        "ms-copilot",
        "ms-news",
        "ms-office-hub"
      ]
    },
    "strict": {
      "displayName": "Estricto",
      "description": "Privacidad fuerte. Cortana off, búsqueda web off.",
      "disclaimer": "Algunas búsquedas integradas dejan de funcionar. Cortana se desactiva completamente.",
      "estimatedChanges": 25,
      "registryTweaks": [
        "disable-advertising-id", "disable-app-suggestions",
        "disable-tailored-experiences", "disable-lockscreen-spotlight",
        "disable-search-bing", "set-telemetry-basic",
        "disable-widgets-button", "disable-activity-history",
        "disable-cortana-policy", "search-icon-only",
        "disable-cloud-protection-notif", "set-telemetry-security",
        "disable-edge-dock"
      ],
      "services": [
        { "name": "DiagTrack", "startType": "Disabled" },
        { "name": "dmwappushsvc", "startType": "Disabled" },
        { "name": "RetailDemo", "startType": "Disabled" }
      ],
      "scheduledTasks": [
        "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
        "\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip",
        "\\Microsoft\\Windows\\Autochk\\Proxy",
        "\\Microsoft\\Windows\\Feedback\\Siuf\\DmClient",
        "\\Microsoft\\Windows\\Feedback\\Siuf\\DmClientOnScenarioDownload"
      ],
      "debloatEntries": [
        "ms-copilot", "ms-news", "ms-office-hub", "ms-cortana", "ms-bing-weather"
      ]
    },
    "paranoid": {
      "displayName": "Paranoid",
      "description": "Privacidad máxima. Rompe cosas.",
      "disclaimer": "MODO AGRESIVO. Lee con cuidado:\n- SmartScreen reduce protección anti-malware.\n- Algunas updates pueden no llegar.\n- Windows Hello biométrico se desactiva.\n- Maps y Location apagados.\n- Restore point obligatorio creado antes.",
      "estimatedChanges": 40,
      "registryTweaks": [
        "disable-advertising-id", "disable-app-suggestions",
        "disable-tailored-experiences", "disable-lockscreen-spotlight",
        "disable-search-bing", "set-telemetry-basic",
        "disable-widgets-button", "disable-activity-history",
        "disable-cortana-policy", "search-icon-only",
        "disable-cloud-protection-notif", "set-telemetry-security",
        "disable-edge-dock", "disable-smartscreen-explorer",
        "disable-wer", "disable-location", "disable-biometrics",
        "disable-explorer-telemetry", "disable-autologger"
      ],
      "services": [
        { "name": "DiagTrack", "startType": "Disabled" },
        { "name": "dmwappushsvc", "startType": "Disabled" },
        { "name": "RetailDemo", "startType": "Disabled" },
        { "name": "WerSvc", "startType": "Disabled" },
        { "name": "WSearch", "startType": "Disabled" }
      ],
      "scheduledTasks": [
        "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
        "\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater",
        "\\Microsoft\\Windows\\Application Experience\\StartupAppTask",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Uploader",
        "\\Microsoft\\Windows\\Autochk\\Proxy",
        "\\Microsoft\\Windows\\Feedback\\Siuf\\DmClient",
        "\\Microsoft\\Windows\\Maps\\MapsUpdateTask",
        "\\Microsoft\\Windows\\Maps\\MapsToastTask"
      ],
      "debloatEntries": [
        "ms-copilot", "ms-news", "ms-office-hub", "ms-cortana",
        "ms-bing-weather", "ms-maps", "ms-mail-calendar",
        "ms-movies-tv", "ms-people", "ms-feedback-hub"
      ]
    }
  }
}
```

## Criterio de done

- [ ] Archivo `privacy-presets.json` creado con 3 presets.
- [ ] Cada IDs referenciado existe en su catálogo respectivo (registry-tweaks, debloat).
- [ ] Disclaimers redactados claros.
- [ ] Estimated changes contado.
- [ ] Schema validado.
