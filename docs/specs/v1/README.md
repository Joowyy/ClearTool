# Specs v1 — Estado actual (v0.1)

Esta carpeta contiene los specs **tal y como se diseñaron al arrancar el proyecto**. Reflejan la arquitectura, módulos y decisiones que están implementadas (parcial o totalmente) en el código actual.

## Estado por documento (2026-05-25)

| Spec | Implementado | Notas |
|------|--------------|-------|
| `00-overview.md` | ✅ | Visión y stack confirmados. |
| `01-architecture.md` | ✅ | Topología `core → platform → domain → models → ipc` aplicada. |
| `02-backend-rust.md` | 🟡 | Falta `audit_log` writer robusto, retry de PowerShell, cancelación real de scans. |
| `03-frontend-react.md` | 🟡 | Tabs en titlebar, React Query, Zustand. Faltan command palette y theme switching. |
| `04-modules/directory-explorer.md` | 🟡 | Sólo scan top-level + expand recursivo (fix mayo 2026). Falta treemap, virtualización. |
| `04-modules/cache-cleaner.md` | 🟡 | Limpieza básica OK. Falla con archivos en uso (UWP, browsers activos). Ver v2. |
| `04-modules/debloat-engine.md` | 🟡 | 11 entradas en el catálogo. Reverse recipe parcial. Ver v2 para expansión. |
| `04-modules/service-manager.md` | 🟡 | Lista + set start-type. Sin gráfico de dependencias. |
| `04-modules/registry-tweaks.md` | 🟡 | Apply + revert. UI tiene glitch de renderizado (backslash sobrante). |
| `04-modules/restore-point-system.md` | 🟡 | Crear + listar. Listar falla silenciosamente sin admin. |
| `05-security.md` | 🟡 | Allowlists implementadas. Falta signing del binario y verificación de catálogos firmados. |
| `06-ui-ux.md` | 🟡 | Refactor "Quirófano Cyan" en marcha. |
| `07-testing.md` | ❌ | No hay tests unitarios ni integración. Sólo el smoke test manual. |

## Qué hay roto / no funciona bien

Estos son los bugs conocidos del v0.1 actual. La solución está documentada en `v2/`:

1. **Cache cleaner**: archivos `in use` que no se pueden eliminar (Spotify, Claude, Discord activos). Solución: `v2/02-cache-engine-rewrite.md`.
2. **Errores `[object Object]`**: el frontend hace `String(err)` sobre objetos `AppError`. Solución: `v2/01-error-model-fix.md`.
3. **Debloat detect**: error al detectar paquetes instalados con `[object Object]`. Mismo origen que el bug anterior.
4. **Restore points**: lista vacía aunque haya admin. Mezcla de error silente + PowerShell `$ErrorActionPreference = SilentlyContinue`. Ver `v2/08-known-issues.md`.
5. **Registry tweaks UI**: carácter `\` extraño al final de la descripción. Probablemente line-break mal escapado en JSON.
6. **Cache cleaner**: deja almacenamiento residual incluso tras "limpieza completa". Las carpetas se reescanan con el mismo tamaño porque sólo se borraron *algunos* archivos.

## Cuándo dejar de usar este v1

Cuando un módulo del v2 esté implementado y validado, su spec correspondiente reemplaza al v1 (mover desde `v2/` a `v1/`). En ese momento esta carpeta deja de ser "la verdad histórica" y pasa a ser "la verdad actual".

Hasta entonces, `v1/` representa el estado **al que el código se aproxima**.
