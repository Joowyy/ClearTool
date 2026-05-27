# Tareas difíciles — Sonnet 4.6

> Esta carpeta concentra las tareas del refinamiento de caché que
> requieren **razonamiento sobre código Rust de backend, internals del
> SO, sincronización entre procesos o decisiones arquitectónicas con
> riesgo de regresión**. Son las que más se benefician de Sonnet 4.6.

## Criterios para asignar aquí

- Toca código **destructivo o crítico** (procesos, FS, IPC) y necesita
  defensa en profundidad + tests.
- Implica **nuevos módulos backend** con lifecycle, locks y caché en
  memoria.
- Exige **expertise específica** (WebGL/Three.js internals, R3F frame
  loop, etc.) que un modelo más pequeño puede aplicar mal con
  consecuencias visibles.
- Requiere **medir → razonar → optimizar** (no sólo aplicar un patch).

## Sets en esta carpeta

### Set A — Refinamiento base (estado: ✅ implementado)

| Doc | Por qué es difícil |
|---|---|
| [`01-shell-safe-close.md`](01-shell-safe-close.md) | Bug de seguridad real (mató Explorer del usuario). Toca whitelist de PIDs protegidos, `suggest_action` y `execute_plan`. Defensa en profundidad en 3 capas + tests Rust. Pasa por `security-auditor` antes de mergear. |
| [`02-auto-analyze-on-boot.md`](02-auto-analyze-on-boot.md) | Nuevo módulo `cache_background.rs` con `once_cell::Lazy<RwLock<...>>`, lifecycle de Tauri (`setup`), 2 comandos IPC nuevos, invalidación post-clean, sincronización con frontend (React Query + `useEffect`). Hay que pensar el TTL y los races. |
| [`05-three-context-lost.md`](05-three-context-lost.md) | Three.js/R3F internals, `frameloop="demand"`, `webglcontextlost` handler, `powerPreference`, fallback 2D, detección de WebGL. Requiere conocer el ciclo de render de R3F a fondo y saber cuándo `invalidate()` es necesario. |
| [`08-redundancias-y-perf.md`](08-redundancias-y-perf.md) | Optimización multi-capa: throttle de eventos IPC, caché de `who_locks_path`, memoización selectiva, defaults globales de React Query, eliminar scan duplicado. Requiere razonar sobre el impacto de cada cambio en CPU/GPU/IPC. |

### Set B — Consola de limpieza (estado: ✅ implementado)

Cuatro bloques que sustituyeron el spinner mudo de "Limpiar" por una
experiencia visual rica con ETA, fase actual, animación 3D y resumen
final.

| Doc | Qué resuelve | Depende de |
|---|---|---|
| [`09-progress-pipeline-backend.md`](09-progress-pipeline-backend.md) | Eventos tipados `cache:line / cache:progress-v2 / cache:phase / cache:summary` con ETA, throughput, fase. Throttle 150 ms. `ProgressTracker` con ventana móvil. Persistencia de muestras de throughput en JSON (anillo 20). | — |
| [`10-clean-console-frontend.md`](10-clean-console-frontend.md) | Componente `CleanConsole` modal con header (fase + ETA + barra + throughput), log scrolleable con auto-scroll, animaciones framer-motion por línea, integración en `cache-page.tsx`. **El toast se mantiene.** | 09 |
| [`11-clean-visualizer-r3f.md`](11-clean-visualizer-r3f.md) | Visualización 3D 120×120 px en el header de la consola: anillo de progreso + halo + partículas orbitando. Color cambia por fase. Respeta frameloop="demand", powerPreference low-power, context-lost handlers. Fallback SVG. | 10 |
| [`12-eta-estimate-and-summary.md`](12-eta-estimate-and-summary.md) | Tiempo estimado en el PlanView **antes** de pulsar Limpiar (basado en throughput histórico). `CleanSummaryHero` al terminar con métricas (espacio liberado, duración, archivos, restore point, errores). Cerrar consola invalida queries. | 09, 10 |

### Set C — Quality of life de la consola (estado: ⏳ pendiente)

Cuatro bloques que resuelven bugs reportados por el usuario tras
probar la consola y proponen una **reescritura del motor** para
pasar de ~880 KB/s a ≥10 MB/s.

| Doc | Qué resuelve | Sev |
|---|---|---|
| [`13-cancellation-token-and-button.md`](13-cancellation-token-and-button.md) | `tokio_util::CancellationToken` E2E, registro global por `runId`, propagación en `execute_plan` y `walk_and_delete`, IPC `cancel_clean_plan`, botón Cancelar en `CleanConsole` con estado `cancelling`, caso `cancelled` en `CleanSummaryHero`. | 🟡 P1 |
| [`14-eta-calculating-forever-fix.md`](14-eta-calculating-forever-fix.md) | El ETA siempre "calculando…" porque el threshold de throughput estaba en 1 MB/s y el caso real son ~880 KB/s. Bajar a 50 KB/s, fallback al throughput acumulado, flag `etaIsPrecise` para mostrar rango si no es preciso. | 🟡 P1 |
| [`15-residual-bytes-investigation.md`](15-residual-bytes-investigation.md) | Los ~400 MB residuales clasificados por motivo (`LockedBySystem`, `PendingReboot`, `AccessDenied`, `FilteredOut`, `ReparsePoint`), UI "Quedó pendiente" en el summary, acción **Ignorar** persistida en `cache-ignore.json`, CTA "Reiniciar ahora". | 🟡 P1 |
| [`16-performance-overhaul.md`](16-performance-overhaul.md) | **Crítico.** 7 min 36 s para 400 MB es inaceptable. `tokio::JoinSet` con `PARALLEL_LOCATIONS=4`, `tokio::fs` async, `stream::for_each_concurrent(8)` dentro de cada ubicación, eliminar retry con sleeps (try-once-or-schedule), lazy `who_locks_path`, `TrackerHandle` thread-safe. Speedup objetivo ≥10×. **Requiere security-auditor.** | 🔴 P0 |

## Recomendación de orden global

**Set A → Set B → Set C**. Dentro de cada set, orden estricto.

Concretamente:

1. **01** — proteger el shell (P0). ✅
2. **02** — auto-analyze al boot (P1). ✅
3. **08** — redundancias (P2). ✅
4. **05** — Three.js Context Lost (P1). ✅
5. **09** — pipeline de progreso enriquecido (P1). ✅
6. **10** — componente `CleanConsole` (P1). ✅
7. **11** — visualización 3D (P2). ✅
8. **12** — estimación previa + resumen final (P1). ✅
9. **13** — cancelación E2E + botón (P1).
10. **14** — ETA fix (P1, independiente — puede ir en paralelo).
11. **15** — residuales con desglose y acciones (P1, depende de 12).
12. **16** — performance overhaul (P0, **el más invasivo del Set C**,
    requiere `security-auditor`).

## Convenciones para esta carpeta

- Antes de mergear cualquier doc, ejecutar `cargo test` con todos los
  tests nuevos pasando.
- Si el doc tiene un test marcado como obligatorio en su sección
  "Criterio de done", **no es opcional**.
- Cambios en backend Rust requieren revisión de `security-auditor` (ver
  CLAUDE.md → mapa de subagentes).
- Set B (09-12) **no** toca código destructivo; pero al cambiar el
  protocolo de eventos (doc 09) conviene mantener los emisores legacy
  (`cache:progress`, `cache:debug`) durante una versión completa para
  no romper código viejo.
