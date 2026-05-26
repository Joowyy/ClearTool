# Paso 03 — Curación Tier 1 (50 entradas — MS first-party + UI clutter)

**Área**: 04-debloat-catalog
**Tiempo estimado**: 8-10 horas (curación humana)
**Dependencias**: Paso 01 (schema)

## Qué hacemos

Documentar 50 entradas del catálogo en las categorías más universales: Apps consumer de Microsoft, system apps desinstalables, AI/telemetría, UI clutter, y juegos M$ first-party.

**No es coding. Es investigación + escritura.**

## Por qué

El catálogo actual tiene 11. Para que la app sirva en cualquier Win 11 (no solo OEM), necesitamos cubrir lo que TODA Windows 11 trae preinstalado.

## Cómo

### 1. Prepara una VM Windows 11 limpia

- ISO oficial de Microsoft.
- 8GB RAM, 50GB disco mínimo.
- Sin iniciar sesión con cuenta MS (cuenta local).
- Activar Developer Mode si quieres `Get-AppxPackage -AllUsers` sin warning.

### 2. Listar TODOS los appx instalados

```powershell
Get-AppxPackage -AllUsers |
  Select-Object Name, PackageFamilyName, Publisher, IsBundle |
  Sort-Object Name |
  Format-Table -AutoSize
```

Anota los que aparecen. Win 11 23H2 limpio típico tiene ~70-90 packages.

### 3. Listar provisionados

```powershell
Get-AppxProvisionedPackage -Online |
  Select-Object DisplayName, PackageName |
  Sort-Object DisplayName
```

### 4. Para cada candidato, decide:
- **Quitar = Sí**: bloatware obvio (Solitaire, Office Hub, Tips, etc.).
- **Quitar = Solo en preset agresivo** (`total`): cosas que algunos quieren (Cortana, Movies & TV).
- **Quitar = NUNCA**: críticos (Windows Terminal, Microsoft Store si quieres reinstalar nada, .NET Framework helpers).

### 5. Plantilla por entrada

```json
{
  "id": "ms-solitaire-collection",
  "displayName": "Microsoft Solitaire Collection",
  "description": "Juegos clásicos de cartas con publicidad integrada.",
  "category": "ms-consumer-app",
  "risk": "low",
  "consequences": [
    "Pierdes acceso rápido a Solitario, FreeCell, etc.",
    "Si quieres jugar otra vez, reinstalar desde Microsoft Store."
  ],
  "removalMethod": "appx-user-and-provisioned",
  "appxPackageFamilyName": "Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe",
  "appxProvisionedName": "Microsoft.MicrosoftSolitaireCollection",
  "preservesDataByDefault": true,
  "requiresAdmin": true,
  "reversible": true,
  "reverseRecipe": {
    "kind": "appxReinstall",
    "packageFamilyName": "Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe",
    "storeUrl": "ms-windows-store://pdp/?ProductId=9WZDNCRFHWD2"
  },
  "minWindowsBuild": 22000,
  "presets": ["recommended", "total"],
  "tags": ["games", "consumer", "solitaire"],
  "alternativeApps": ["PySol FC (open source)"],
  "verified": "2026-05-25",
  "verifiedBy": "@joelsanchez"
}
```

### 6. Las 50 entradas Tier 1 (lista priorizada)

**ms-consumer-app (15)**:
1. Microsoft Solitaire Collection
2. Mahjong (Microsoft Mahjong)
3. Microsoft To Do
4. Microsoft Sticky Notes (si user lo usa, presets sólo `total`)
5. Microsoft Whiteboard
6. Office Hub (Microsoft 365)
7. Get Help
8. Tips (Get Started)
9. Feedback Hub
10. Microsoft Wallet
11. LinkedIn (preinstalado en algunas)
12. Microsoft Family Safety
13. Outlook (new UWP)
14. Cortana (deprecated en 24H2 pero aún en 22H2)
15. People

**ms-system-app (12)**:
16. 3D Viewer
17. Mixed Reality Portal
18. Quick Assist
19. Movies & TV
20. Groove Music
21. Voice Recorder
22. Camera (preset solo `total`)
23. Maps
24. Mail and Calendar
25. Alarms & Clock
26. Power Automate Desktop
27. Paint 3D

**ai-and-telemetry (6)**:
28. Microsoft Copilot (Appx + service)
29. Cortana (servicio + appx; ya cubierto pero por completitud)
30. ConnectedUserExperiencesAndTelemetry (servicio)
31. dmwappushservice (servicio)
32. DiagTrack (servicio)
33. Customer Experience Improvement Program (scheduled tasks bundle)

**ui-clutter (8)**:
34. Widgets (WebExperiencePack)
35. Chat (MicrosoftTeams consumer)
36. News widget (NewsAndInterests via registry tweak)
37. Search box → "icono solo" (registry tweak)
38. Edge dock icons (registry)
39. Microsoft Edge desktop shortcut (eliminar .lnk)
40. Lock screen Spotlight ads (registry)
41. Start Menu suggestions (registry)

**games (9 — first-party MS)**:
42. Xbox app
43. Xbox Game Bar (Microsoft.XboxGamingOverlay)
44. Xbox Live (Microsoft.XboxLive)
45. Xbox Identity Provider
46. Xbox Speech to Text Overlay
47. Solitaire (cubierto en ms-consumer)
48. Minecraft for Windows trial
49. Microsoft Tic Tac Toe
50. Microsoft Sudoku (si viene preinstalado)

### 7. Validación

Tras añadir las 50, correr:

```bash
ajv validate -s bloatware-catalog.schema.json -d bloatware-catalog.json
```

Debe pasar. Si no, ver el error y arreglar campos.

### 8. Verificar reverse recipe

Cada entrada con `removalMethod: "appx-*"` debe tener `reverseRecipe.kind: "appxReinstall"`.
Cada entrada de tipo registry/service debe tener `reverseRecipe.kind: "registry"` o `"service"`.
Si no se puede revertir automáticamente → `"noop"` con `reason` claro.

## Criterio de done

- [ ] 50+ entradas Tier 1 añadidas al catálogo v2.
- [ ] `ajv validate` pasa.
- [ ] Cada entrada tiene `verified` con fecha y `verifiedBy`.
- [ ] Cada entrada tiene mínimo 1 `consequence` clara.
- [ ] Cada entrada con appx tiene la `packageFamilyName` exacta verificada en VM.
- [ ] Reverse recipes específicas, no genéricas.
- [ ] `presets` poblado para cada entrada (al menos una de las 5 opciones).
- [ ] Smoke test backend: `detect_installed` en la VM detecta ≥ 30 de las 50.

## Recursos

- [Get-AppxPackage docs](https://learn.microsoft.com/en-us/powershell/module/appx/get-appxpackage)
- [Removing built-in apps](https://learn.microsoft.com/en-us/windows/configuration/remove-builtin-apps)
- [Microsoft Store Product IDs (para storeUrl)](https://learn.microsoft.com/en-us/windows/uwp/publish/launch-store-app)
