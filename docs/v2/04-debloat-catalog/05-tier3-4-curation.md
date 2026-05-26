# Paso 05 — Curación Tier 3+4 (30 entradas)

**Área**: 04-debloat-catalog
**Tiempo estimado**: 4-6 horas
**Dependencias**: Paso 01

## Qué hacemos

Curar las últimas 30 entradas: third-party preinstalado + power-user avanzado.

## Tier 3 — Third-party preinstalado (20)

Apps que vienen preinstaladas vía Marketing Suite de Microsoft o OEM agreements:

1. Spotify (UWP) — `SpotifyAB.SpotifyMusic_*`
2. LinkedIn
3. Disney+ (en algunos países)
4. TikTok
5. Instagram
6. WhatsApp Desktop (UWP, no la versión Win32)
7. Twitter / X
8. Netflix
9. Adobe Creative Cloud (trial)
10. Booking.com
11. ESPN
12. Amazon Prime Video
13. Hulu (US)
14. Roblox (en algunas builds)
15. Candy Crush Saga
16. Candy Crush Soda Saga
17. Disney Magic Kingdoms
18. March of Empires
19. Royal Revolt 2
20. Asphalt 8

Categoría: `third-party-bundled`. Risk: `low` (no son críticos). Removal: `appx-user-and-provisioned`.

Ejemplo:
```json
{
  "id": "third-party-spotify-uwp",
  "displayName": "Spotify (UWP preinstalado)",
  "description": "Versión UWP de Spotify que viene preinstalada. NO es la versión .exe descargada de spotify.com.",
  "category": "third-party-bundled",
  "risk": "low",
  "consequences": [
    "Pierdes la versión UWP. Puedes descargar Spotify clásico desde spotify.com.",
    "Música offline almacenada en LocalCache se pierde."
  ],
  "removalMethod": "appx-user-and-provisioned",
  "appxPackageFamilyName": "SpotifyAB.SpotifyMusic_zpdnekdrzrea0",
  "appxProvisionedName": "SpotifyAB.SpotifyMusic",
  "preservesDataByDefault": false,
  "requiresAdmin": true,
  "reversible": true,
  "reverseRecipe": {
    "kind": "appxReinstall",
    "packageFamilyName": "SpotifyAB.SpotifyMusic_zpdnekdrzrea0",
    "storeUrl": "ms-windows-store://pdp/?ProductId=9NCBCSZSJRSB"
  },
  "minWindowsBuild": 22000,
  "presets": ["minimal", "recommended", "total"],
  "tags": ["third-party", "music", "preinstalled"],
  "alternativeApps": ["Spotify .exe desde spotify.com", "MusicBee"],
  "verified": "2026-05-25"
}
```

## Tier 4 — Avanzado (10 — riesgo medium/high)

Solo para power users. Cada uno con disclaimer obligatorio.

21. **Windows Subsystem for Linux (WSL)** — `risk: "medium"`. Eliminar si nadie usa Linux en el PC. Quita Kernel, distros, vhd.
22. **Hyper-V** — `risk: "high"`. Libera RAM/CPU pero rompe Docker Desktop si lo usas. Disclaimer explícito.
23. **WindowsBackup** (nuevo en 24H2 forzado) — `risk: "low"`. Backup automático a OneDrive.
24. **BingWeather** — `risk: "low"`.
25. **StorageSpaces UI** — `risk: "medium"`. Solo si no se usan storage pools.
26. **MixedReality / Holographic** — `risk: "low"`. Si no tienes HoloLens / Reality headset.
27. **Internet Explorer mode in Edge** (registry tweak) — `risk: "medium"`. Algunas intranets viejas lo necesitan.
28. **Recall** (Copilot+ feature en algunos modelos) — `risk: "medium"`. AI memory de actividad.
29. **PenWorkspace / InkWorkspace** — `risk: "low"`. Solo útil con stylus.
30. **Microsoft Family** — `risk: "low"`. Control parental.

Ejemplo Tier 4:

```json
{
  "id": "windows-hyperv",
  "displayName": "Hyper-V (virtualización)",
  "description": "Hipervisor de Windows. Da soporte a Docker Desktop con WSL2, Windows Sandbox, máquinas virtuales.",
  "category": "dev-tools-unused",
  "risk": "high",
  "consequences": [
    "Docker Desktop deja de funcionar con WSL2.",
    "Windows Sandbox no inicia.",
    "Liberas ~500 MB RAM y mejora tiempo de arranque en 1-2s.",
    "Pierdes máquinas virtuales creadas con Hyper-V Manager."
  ],
  "removalMethod": "compound",
  "compoundSteps": [
    {
      "method": "registry-policy",
      "writes": [
        {
          "hive": "HKLM",
          "key": "SYSTEM\\CurrentControlSet\\Services\\HvHost",
          "name": "Start",
          "type": "REG_DWORD",
          "value": 4
        }
      ]
    }
  ],
  "preservesDataByDefault": true,
  "requiresAdmin": true,
  "reversible": true,
  "reverseRecipe": {
    "kind": "registry",
    "operations": [
      {
        "hive": "HKLM",
        "key": "SYSTEM\\CurrentControlSet\\Services\\HvHost",
        "name": "Start",
        "type": "REG_DWORD",
        "value": 2
      }
    ]
  },
  "presets": ["total"],
  "tags": ["virtualization", "advanced", "dev"],
  "disclaimerLevel": "danger",
  "disclaimerText": "Si usas Docker Desktop, NO deshabilitar Hyper-V. Romperá tu setup de desarrollo.",
  "learnMoreUrl": "https://learn.microsoft.com/en-us/virtualization/hyper-v-on-windows/quick-start/enable-hyper-v",
  "verified": "2026-05-25"
}
```

## Criterio de done

- [ ] 20 entradas Tier 3 (third-party) añadidas.
- [ ] 10 entradas Tier 4 (avanzado) añadidas.
- [ ] Cada Tier 4 tiene `disclaimerLevel: "danger"` o `"warning"`.
- [ ] Cada Tier 4 tiene `presets: ["total"]` o `["privacy-paranoid"]` únicamente (NO en recommended).
- [ ] Schema validation pasa.
- [ ] Total catálogo ≥ 120 entradas.
- [ ] Smoke test backend: load + validate + detect en VM sin crash.

## Cierre del trabajo de curación

Total entradas v2.0:
- Tier 1: 50
- Tier 2: 40
- Tier 3: 20
- Tier 4: 10
- **Total**: 120

Vínculo con `02-detect-multi-metodo.md`: la detect mejorada ya cubre los 5 métodos que necesitamos.

Vínculo con `06-disclaimers-y-user-merge.md`: ahí la UI implementa los modales de disclaimer.
