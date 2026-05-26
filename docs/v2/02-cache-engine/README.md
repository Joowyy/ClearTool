# 02 — Cache Engine Rewrite (M2, P0)

**Objetivo**: rediseñar el cache cleaner para que (a) sepa quién bloquea qué archivo, (b) ofrezca acción al usuario, (c) verifique resultado real tras limpiar.

**Spec de referencia**: `.claude/specs/v2/02-cache-engine-rewrite.md`.

**Dependencias hard**:
- `03-process-manager/` **debe estar terminado primero** (necesitamos `who_locks_path` y `close_gracefully`).
- `01-error-handling/` terminado (los nuevos errores son tipados y dependen del normalizer).

## Pasos en orden

1. [01 — Restart Manager API (`who_locks`)](01-restart-manager-api.md) ⚠ requiere `03-process-manager` solo si no se hace ahí
2. [02 — Helper `PendingFileRenameOperations`](02-pending-rename-helper.md)
3. [03 — Modelos: `CleanPlan`, `LocationStatus`, `BlockedLocation`](03-modelos-clean-plan.md)
4. [04 — Pre-flight: `analyze_locations`](04-analyze-locations.md)
5. [05 — Ejecución: `execute_plan` con retry + estrategias](05-execute-plan-retry.md)
6. [06 — Verify after clean](06-verify-after-clean.md)
7. [07 — Catálogo: añadir `strategy` y `lockedBy` por entrada](07-catalogo-strategies.md)
8. [08 — IPC commands + frontend rediseño](08-ipc-y-frontend.md)

## Criterio de done (agregado)

- [ ] `who_locks(path)` devuelve PID + nombre del proceso que tiene un handle abierto.
- [ ] `analyze_locations` clasifica cada path en 4 grupos: ready / blocked / permission-issue / non-existent.
- [ ] `execute_plan` aplica retry x3 con backoff exponencial.
- [ ] Si un archivo no se puede borrar, se programa via PendingFileRenameOperations con logging claro.
- [ ] Tras limpiar, `verify_after` re-escanea y reporta bytes reales liberados.
- [ ] El catálogo tiene `strategy` poblada para todas las entradas (`direct-delete`, `uwp-app-aware`, `browser-aware`, etc.).
- [ ] UI muestra las 4 secciones (ready / blocked / permission / skipped) con totales por sección.
- [ ] Botón "Cerrar Spotify y limpiar" funciona end-to-end.
- [ ] Smoke test VM: con Spotify+Discord+Chrome abiertos, "Limpiar" libera el 80%+ de lo escaneado.

## Tiempo total estimado

40-50 horas. Es el área más grande de M2.

## Notas críticas

- **NUNCA tocar `%LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_*`** o paquetes críticos. Mantener allowlist (ver Paso 07).
- **NO usar MoveFileEx + DELAY_UNTIL_REBOOT en archivos críticos del sistema** (Windows Terminal, Microsoft Store, AppInstaller). Mantener blocklist explícita.
- Restore point obligatorio antes de `execute_plan` (ya está en v0.1, mantener).
