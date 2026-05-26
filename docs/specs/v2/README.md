# Specs v2 — Plan para release público (v1.0)

Esta carpeta documenta el **camino entre v0.1 y v1.0**. No es una wishlist: es un plan accionable con prioridades, dependencias y criterios de "done".

## Por qué v2 existe

ClearTool v0.1 funciona como prueba de concepto en máquina local. Tiene bugs visibles (`[object Object]` en errores, archivos UWP que no se borran, restore points que no aparecen) y carencias estructurales (sin signing, sin tests, catálogo pequeño, sin process manager). v2 documenta **exactamente** qué hay que arreglar y qué hay que añadir para que la app pueda publicarse.

## Cómo leer esta carpeta

```
v2/
├── 00-vision-public-release.md   ← empezar aquí (5 min)
├── 01-error-model-fix.md         ← bug crítico (sangra UX en toda la app)
├── 02-cache-engine-rewrite.md    ← bug crítico (cleaner deja residuos)
├── 03-process-manager.md         ← módulo nuevo (desbloquea cache cleaner)
├── 04-debloat-catalog-expansion.md ← contenido (catálogo de 11 → 120+)
├── 05-new-modules.md             ← módulos nuevos compactos (startup, boot, disk, net, privacy)
├── 06-ui-ux-refactor.md          ← layout, palette, theme
├── 07-distribution.md            ← signing, installer, Store
├── 08-known-issues.md            ← catálogo COMPLETO de bugs y fixes
└── 09-roadmap.md                 ← orden de ejecución (M1→M5)
```

## Resumen de prioridades

| Severidad | Categoría | Specs | Por qué |
|-----------|-----------|-------|---------|
| 🔴 P0 | Bug crítico de UX | `01`, `08` | Sin error model decente, el usuario ve `[object Object]` por todas partes. Esto bloquea release. |
| 🔴 P0 | Bug crítico de funcionalidad | `02`, `03` | El cache cleaner **no completa su trabajo principal**. Sin process manager no se pueden desbloquear los archivos en uso. |
| 🟠 P1 | Contenido | `04` | El catálogo actual cubre 11 apps. Un Windows 11 con OEM tiene fácilmente 60+ candidatos a debloat. |
| 🟠 P1 | Confianza | `07` | Sin signing y sin auto-update, ningún usuario va a instalar este `.exe` desde GitHub. |
| 🟡 P2 | Diferenciación | `05`, `06` | Disk analyzer, command palette y theme son features que separan a ClearTool de "otro cleaner más". |
| 🟢 P3 | Calidad | `v1/07-testing.md` | Tests bloquean refactors masivos, pero no bloquean release. |

## Principios v2

1. **Error first**. Cada operación destructiva tiene un caso de fallo bien tipado, un mensaje accionable y una reversa documentada. Sin `String(err)` ni `[object Object]`.
2. **Process-aware**. Antes de tocar un archivo, comprobar quién lo tiene abierto. Ofrecer al usuario cerrar el proceso o saltar.
3. **Reversibilidad doble** (heredada de v1): restore point + audit log siguen siendo obligatorios.
4. **Cero telemetría de red por defecto**. Toda diagnostic data se guarda local y se exporta sólo si el usuario lo pide.
5. **Plug-in friendly**. Catálogos (bloatware, registry tweaks, cache locations) cargables desde JSON externo además del embebido, para que la comunidad pueda contribuir sin recompilar.
6. **Sin features a medias**. Si un módulo no se puede completar con calidad en v1.0, se queda fuera y se pospone a v1.1.

## Estado del trabajo v2

| Spec | Estado | Tracking |
|------|--------|----------|
| 00-vision | Escrito | — |
| 01-error-model | Escrito | Pendiente código |
| 02-cache-engine | Escrito | Pendiente código |
| 03-process-manager | Escrito | Pendiente código |
| 04-debloat-catalog | Escrito | Pendiente curación |
| 05-new-modules | Escrito | Pendiente código |
| 06-ui-ux | Escrito | Pendiente diseño + código |
| 07-distribution | Escrito | Pendiente cert |
| 08-known-issues | Escrito | — |
| 09-roadmap | Escrito | — |

## Cómo proponer cambios a v2

1. Edita el spec correspondiente con un PR.
2. Si tu cambio es estructural (nuevo módulo, refactor masivo), añade entrada en `09-roadmap.md`.
3. Si tu cambio resuelve un bug del v1, márcalo en `08-known-issues.md` con el commit que lo cerró.
