# Paso 04 — Curación Tier 2 (40 entradas — OEM)

**Área**: 04-debloat-catalog
**Tiempo estimado**: 8 horas
**Dependencias**: Paso 01

## Qué hacemos

Curar 40 entradas de bloatware específico de fabricantes (Lenovo, HP, Dell, Asus, Acer, Samsung, Razer, MSI).

## Por qué

El usuario que más impacto ve de ClearTool es quien compra un portátil OEM con 30 apps preinstaladas que no quiere. Si ClearTool no detecta HP Wolf Security ni Lenovo Vantage, no le sirve.

## Estrategia de curación

Estos paquetes NO son Appx normalmente. Son installers MSI clásicos. Se detectan por:
- `uninstaller-string` con `displayNamePattern`.
- A veces `service` (HP Wolf Security tiene servicios).

## Las 40 entradas (por fabricante)

### Lenovo (8)
1. Lenovo Vantage — `displayNamePattern: "Lenovo Vantage*"`, publisher: "Lenovo"
2. Lenovo Smart Communication
3. Lenovo Welcome
4. Lenovo Pen Settings
5. Lenovo Mod Pack
6. Lenovo Service Bridge
7. McAfee LiveSafe (trial preinstalado) — `displayNamePattern: "McAfee LiveSafe*"`, publisher: "McAfee, LLC"
8. McAfee Safe Connect

### HP (8)
9. HP Wolf Security — riesgo medium (es un AV, avisar)
10. HP JumpStart
11. HP Audio Switch
12. HP Customer Experience Enhancements (telemetría)
13. HP Connection Optimizer
14. HP Documentation
15. HP Smart
16. HP System Event Utility

### Dell (6)
17. Dell SupportAssist — riesgo medium (gestor de soporte)
18. Dell Optimizer
19. Dell Mobile Connect
20. Dell Customer Connect
21. Dell Power Manager
22. Dell Display Manager

### Asus (5)
23. Armoury Crate — riesgo medium (gestiona periféricos Asus; quitar solo si no se usan)
24. MyAsus
25. ASUS GiftBox
26. ASUS Tutorial
27. ASUS Splendid

### Acer (4)
28. Care Center
29. Quick Access
30. Configuration Manager
31. Acer Collection

### Samsung (3)
32. Samsung Notes
33. Samsung Settings
34. Samsung Update

### Razer (3)
35. Razer Synapse — riesgo high (sin esto, periféricos Razer pierden funcionalidad)
36. Razer Cortex
37. Razer Central

### MSI / Otros (3)
38. MSI Center
39. Norton Security (trial preinstalado en muchos)
40. Bitdefender Antivirus Free (en algunos OEM EU)

## Plantilla OEM

```json
{
  "id": "hp-wolf-security",
  "displayName": "HP Wolf Security",
  "description": "Antivirus + EDR de HP preinstalado en portátiles HP empresariales.",
  "category": "oem-bloatware",
  "risk": "medium",
  "consequences": [
    "Tras desinstalar, Windows Defender se activará automáticamente.",
    "Pierdes protección EDR específica de HP (en empresas con políticas internas, NO desinstalar).",
    "Algunos servicios HP de gestión podrían dejar de funcionar correctamente."
  ],
  "removalMethod": "uninstaller-string",
  "displayNamePattern": "HP Wolf Security*",
  "publisher": "HP Inc.",
  "preservesDataByDefault": false,
  "requiresAdmin": true,
  "reversible": false,
  "reverseRecipe": {
    "kind": "noop",
    "reason": "Para reinstalar: descargar desde HP Support → buscar 'HP Wolf Security Console' → seleccionar modelo."
  },
  "presets": ["recommended", "total"],
  "tags": ["oem", "hp", "antivirus", "security"],
  "disclaimerLevel": "warning",
  "disclaimerText": "HP Wolf es un AV. Al desinstalar Windows Defender toma el control. Si tu PC pertenece a una empresa, consulta con IT antes.",
  "learnMoreUrl": "https://www.hp.com/us-en/security/endpoint-security-solutions.html",
  "verified": "2026-05-25",
  "verifiedBy": "@joelsanchez"
}
```

## Investigación necesaria por cada OEM

Para cada fabricante:

1. Descargar ISO OEM oficial o instalar desde recovery partition en VM.
2. Listar todos los uninstallers post-install:
   ```powershell
   Get-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\* |
     Where-Object Publisher -like "*Lenovo*" |
     Select-Object DisplayName, Publisher, UninstallString
   ```
3. Probar la uninstall string en VM (verifica que el método funciona).
4. Documentar consequences observadas (¿qué se rompe?).

## Tips para riesgos

- **Servicios OEM que afectan hardware** (Synapse para Razer, Vantage para Lenovo si gestiona la pantalla): `risk: "high"` o `"medium"`. Disclaimer explícito.
- **Trial antivirus** (McAfee, Norton): `risk: "low"`. Quitarlo activa Defender.
- **Telemetría OEM** (HP Customer Experience, Dell Customer Connect): `risk: "low"`, sin disclaimer.
- **Gestores opcionales** (Lenovo Vantage, Armoury Crate): `risk: "medium"`. Disclaimer: "puedes perder funciones de hardware específicas".

## Detección sin VM física

Si no tienes acceso a todas las marcas, usa fuentes:
- [Tron Script's Stage 1 lists](https://github.com/bmrf/tron) — comunidad técnica.
- [Sycnex's Win10/11 debloater](https://github.com/Sycnex/Windows10Debloater) — referencias de uninstall strings.
- [Reddit r/sysadmin OEM bloat threads](https://reddit.com/r/sysadmin/search?q=OEM+bloatware).

## Criterio de done

- [ ] 40 entradas OEM añadidas.
- [ ] Cada una con `displayNamePattern` para uninstaller-string.
- [ ] Disclaimers redactados para los AV trials y gestores hardware.
- [ ] `reverseRecipe.kind: "noop"` con instrucciones de re-descarga cuando aplique.
- [ ] Schema validation pasa.
- [ ] Smoke test en VM OEM (al menos 1 marca, idealmente 2): detect funciona para esa marca.
