# v2 · 08 — Catálogo de bugs conocidos (v0.1 → v1.0)

Listado completo de bugs y carencias detectadas en v0.1 a fecha 2026-05-25. Cada uno con: severidad, repro, causa raíz hipotética, spec donde se resuelve.

Convención:
- 🔴 **P0** — bloquea release, sangra UX visible
- 🟠 **P1** — degrada confianza, fácilmente observable
- 🟡 **P2** — molesto pero no rompe nada
- 🟢 **P3** — cosmético / nice-to-have

---

## CACHE CLEANER

### #001 🔴 P0 — Archivos UWP en uso no se eliminan

**Repro**:
1. Tener Spotify, Claude Desktop o Discord abierto.
2. Caché → Escanear → seleccionar "UWP Apps".
3. Limpiar.
4. Re-escanear.

**Resultado**: 14 GB siguen reportándose como ocupados, aunque la operación se marca como "exitosa".

**Causa raíz**:
- Archivos memory-mapped por procesos activos (LevelDB, IndexedDB, SQLite WAL).
- `fs::remove_file` falla con "file in use".
- `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` también falla para muchos UWP files.
- No hay retry ni detección de proceso bloqueador.

**Fix**: `v2/02-cache-engine-rewrite.md` + `v2/03-process-manager.md`.

**Cierre verificable**: tras la implementación, ofrecer "Cerrar Spotify y limpiar" desde la UI; el espacio se libera correctamente.

---

### #002 🔴 P0 — Errores `[object Object]` en cache cleaner

**Repro**: Caché → Limpiar con un error backend → ver mensaje "Error: [object Object]".

**Causa raíz**: `String(removeMutation.error)` aplicado sobre un `AppError` JSON serializado.

**Fix**: `v2/01-error-model-fix.md`.

---

### #003 🟠 P1 — Tras limpiar, el escaneo reporta el mismo tamaño

**Repro**: Caché → escanear (3.5 GB) → limpiar → escanear de nuevo (3.4 GB).

**Causa raíz**: Sólo se eliminaron archivos pequeños no bloqueados (~100 MB). La UI no diferencia "fallo" de "éxito parcial" — el botón se quedó verde.

**Fix**: `v2/02-cache-engine-rewrite.md` (sección "Verificación post-clean"). La UI mostrará:
```
Liberado: 3.2 GB · Bloqueado: 200 MB · Programado para reboot: 0 B
```

---

### #004 🟡 P2 — Logs en consola gigantescos por cada archivo

**Repro**: Limpiar caché grande → consola muestra cientos de líneas WARN/ERROR.

**Causa raíz**: El backend emite un evento por cada archivo, el frontend los renderiza todos.

**Fix**: Agrupar logs en backend (`N archivos bloqueados por proceso X`). Limitar live logs a 100 entradas; el resto en archivo `audit.jsonl` consultable después.

---

## DEBLOAT

### #005 🔴 P0 — "Detectar instalados" tira `[object Object]`

**Repro**: Debloat → Detectar instalados.

**Resultado**: banner rojo `Error al detectar paquetes: [object Object]`.

**Causa raíz**:
- Hasta `feat: implement all critical bug fixes`, el TS type `DetectedPackage` no coincidía con Rust → `p.entryId` undefined → bug silente.
- Adicionalmente, si la PowerShell para Appx falla (Get-AppxPackage sin admin completo o con bloqueo de antivirus), el `[object Object]` aparece.

**Fix**:
- TS type ya corregido (parcheado en `docs/fixes/03-debloat-detect.md`).
- Error model: `v2/01-error-model-fix.md`.
- Validación previa: si no hay admin, deshabilitar el botón con tooltip explicativo en lugar de fallar a posteriori.

---

### #006 🟠 P1 — Catálogo demasiado pequeño (11 entradas)

**Repro**: Debloat → ver lista.

**Resultado**: 11 entradas. Un Win 11 OEM típico tiene 60+ candidatos a debloat.

**Fix**: `v2/04-debloat-catalog-expansion.md` (expansión a ~120 curadas).

---

### #007 🟡 P2 — `removalStrategy` y `consequences[0]` no se ven bien

**Repro**: Debloat → mirar columna "Estrategia" → guion `—` en mayoría de filas.

**Causa raíz**: El catálogo no rellena `removal_strategy` para todas las entradas. La columna "Reversa" idem.

**Fix**: como parte de la expansión (`v2/04`), cada entry tendrá `removalStrategy`, `consequences[]` y `reversalMethod` poblados.

---

## RESTORE POINTS

### #008 🔴 P0 — Lista vacía aunque hay restore points

**Repro**: Restauración → ver "Sin puntos de restauración" aunque haya creados.

**Causa raíz**:
- PowerShell script con `$ErrorActionPreference = 'SilentlyContinue'` → si falla, stdout vacío → Rust devuelve `Ok(Vec::new())`.
- Sin admin completo, `Get-ComputerRestorePoint` no devuelve resultados.

**Fix**:
1. Cambiar `SilentlyContinue` por `Stop` y propagar el error.
2. Si no hay admin → mensaje explícito.
3. Considerar fallback a WMI directo (`Get-WmiObject -Class SystemRestore`).
4. `v2/01-error-model-fix.md` para que el error llegue legible a la UI.

---

### #009 🟠 P1 — `creationTime` vs `createdAt` (campos TS desincronizados)

Ya parcheado en `docs/fixes/04-restore-points.md`. Documentado aquí como evidencia histórica.

---

### #010 🟡 P2 — Crear restore point con throttle bypass requiere HKLM write

**Repro**: Sin admin → "Crear punto" → error registry permission denied.

**Causa raíz**: `set_throttle_bypass` escribe `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore` antes de llamar a `SRSetRestorePointW`.

**Fix**: Detectar admin antes; si no, ofrecer ejecutar sin bypass throttle (mensaje: "Windows limita a 1 punto cada 24h sin admin").

---

## AUDIT LOG

### #011 🔴 P0 — React crash "Objects are not valid as a React child"

Ya parcheado en `docs/fixes/05-audit-crash.md`. Causa: `ReverseRecipe` se renderizaba como object en lugar de string.

---

### #012 🟡 P2 — Sin paginación / virtualización

**Repro**: Generar 200+ entries → la página se vuelve laggy.

**Fix**: Virtualizar la lista con react-virtuoso si entries > 50.

---

### #013 🟡 P2 — Sin filtros (módulo, fecha, status)

**Repro**: Audit → no se puede filtrar "sólo errores" ni "sólo debloat".

**Fix**: añadir filtros + búsqueda. Tarea de UX, parte de `v2/06-ui-ux-refactor.md`.

---

## REGISTRY TWEAKS

### #014 🟡 P2 — Glitch de renderizado: `\` al final de descripciones

**Repro**: Registro → ver entradas.

**Resultado**: bajo cada descripción aparece un `\` solo en una línea.

**Causa raíz hipótesis**: la JSON del catálogo tiene `\n` mal escapado o un `\\` que el JSX renderiza como `\`. Inspeccionar `registry-tweaks.json` y la `<RegistryRow>`.

**Fix**: revisar el catálogo + componente; añadir test para que la regresión no vuelva.

---

### #015 🟡 P2 — Sin agrupación por categoría

**Repro**: Registro → lista plana sin secciones.

**Fix**: agrupar por `category` (search, ui, privacy, ...). Parte de UX refactor.

---

### #016 🟡 P2 — "Estado actual" no se ve

**Repro**: Registro → columna "Estado actual" vacía en muchas filas.

**Causa raíz**: `readRegistryTweakState` no se llama automáticamente al cargar la lista. La UI necesita un batch read.

**Fix**: nuevo IPC `read_registry_tweak_states_batch(ids)` que devuelve states de todos.

---

## SERVICIOS

### #017 🟡 P2 — Sin presets visibles

**Repro**: Servicios → no hay botones de preset (similares a Debloat).

**Fix**: añadir presets ("Mínimo gaming", "Mínimo privacidad", "Recomendado").

---

### #018 🟡 P2 — Sin dependency graph

**Repro**: Servicios → no se ve qué depende de qué.

**Fix**: añadir vista de dependencias para evitar romper cascadas.

---

## EXPLORER

### #019 🟢 P3 — Sin treemap visual

Cubierto por `v2/05-new-modules.md` (Disk Analyzer).

### #020 🟢 P3 — Sin virtualización para > 5k nodos

**Repro**: Escanear `C:\` con `maxDepth: 5` → árbol con miles de nodos → lag.

**Fix**: virtualizar tabla con react-virtuoso cuando > 200 visibles.

### #021 🟢 P3 — Sin filtros (sólo carpetas, sólo > tamaño, etc.)

**Fix**: añadir filtros en el header del árbol.

---

## DASHBOARD / HOME

### #022 🟢 P3 — Métricas no se persisten

**Repro**: Cerrar app → reabrir → telemetría empieza desde 0.

**Fix**: Ring buffer en disco para CPU/RAM history (últimas 24h).

### #023 🟡 P2 — Top procesos no es clickable

**Repro**: Home → click en proceso → no hace nada.

**Fix**: click → navegar a `/processes?pid=X` con el proceso preseleccionado.

---

## SETTINGS

### #024 🟡 P2 — Settings escasos

**Repro**: Settings → faltan opciones evidentes (tema, idioma, restore behavior).

**Fix**: ya hay scaffold de settings. Llenarlo con las opciones de `v2/06-ui-ux-refactor.md`.

### #025 🟢 P3 — Sin export/import de settings

**Fix**: botones "Exportar settings" / "Importar settings" (JSON).

---

## SHELL / NAVIGATION

### #026 🟠 P1 — Titlebar nativa + custom (BUG DOBLE TITLEBAR)

Ya parcheado en `docs/fixes/01-double-titlebar.md`.

### #027 🟡 P2 — Tabs en titlebar saturadas

Fix: `v2/06-ui-ux-refactor.md` — migrar a sidebar.

### #028 🟢 P3 — Sin atajos de teclado

Fix: `v2/06-ui-ux-refactor.md` — Ctrl+K palette + atajos por módulo.

---

## ERRORES GENÉRICOS

### #029 🔴 P0 — `String(err)` produce `[object Object]` cross-cutting

Fix: `v2/01-error-model-fix.md`.

### #030 🟠 P1 — No hay error boundary

**Repro**: Cualquier crash de render → "Hey developer 👋" de React Router.

Fix: `v2/01-error-model-fix.md` parte C.

### #031 🟡 P2 — Sin sistema de toasts

Fix: `v2/01-error-model-fix.md` parte D + `v2/06-ui-ux-refactor.md`.

---

## INFRAESTRUCTURA

### #032 🟠 P1 — Sin code signing

Fix: `v2/07-distribution.md`.

### #033 🟠 P1 — Sin auto-updater

Fix: `v2/07-distribution.md`.

### #034 🟡 P2 — Sin tests automatizados

Cubierto por `v1/07-testing.md` (spec heredado). Pendiente implementación.

### #035 🟡 P2 — Sin CI configurado

Fix: `.github/workflows/build.yml` con build matrix + cargo test + tsc + lint.

### #036 🟢 P3 — Sin sentry/crash reporting

**Justificación**: cero telemetría por diseño (ver `v2/00-vision-public-release.md`). NO se implementa salvo opt-in explícito futuro.

---

## OBSERVABILIDAD

### #037 🟡 P2 — Logs no exportables

**Repro**: Bug en backend → usuario no puede dar a "Exportar logs" para enviar.

**Fix**: botón "Generar reporte de diagnóstico" → zip con `audit.jsonl` + logs + system summary.

### #038 🟡 P2 — Sin "About" con info técnica

**Fix**: Settings → About con versión, build date, OS detected, deps versions.

---

## DOC / META

### #039 🟢 P3 — README mínimo

**Fix**: README serio con screenshots, install, FAQ, screenshots, license, contributing.

### #040 🟢 P3 — Sin CONTRIBUTING.md

**Fix**: guía corta para PRs (especialmente para curación de catálogos).

---

## RESUMEN POR PRIORIDAD

| Severidad | Cantidad | Specs que los cubren |
|-----------|----------|----------------------|
| 🔴 P0 | 5 | `01`, `02`, `03`, `04` |
| 🟠 P1 | 10 | `01`, `02`, `04`, `06`, `07`, `08` |
| 🟡 P2 | 17 | `05`, `06`, `08` |
| 🟢 P3 | 8 | `05`, `06` post-1.0 |
| **TOTAL** | **40** | — |

Una vez resueltos los P0 + P1, el release v1.0 es defendible.

## Mantenimiento de este catálogo

Cuando un bug se cierre, **no se borra**: se marca `✅ resuelto en commit <sha> / spec <X>`. Esto mantiene historia útil.
