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

### Set B — Consola de limpieza (estado: ⏳ pendiente)

Cuatro bloques que se implementan **en orden estricto** porque cada uno
depende del anterior. El objetivo es sustituir el spinner mudo de
"Limpiar" por una experiencia visual rica con ETA, fase actual,
animación 3D y resumen final.

| Doc | Qué resuelve | Depende de |
|---|---|---|
| [`09-progress-pipeline-backend.md`](09-progress-pipeline-backend.md) | Eventos tipados `cache:line / cache:progress-v2 / cache:phase / cache:summary` con ETA, throughput, fase. Throttle 150 ms. `ProgressTracker` con ventana móvil. Persistencia de muestras de throughput en JSON (anillo 20). | — |
| [`10-clean-console-frontend.md`](10-clean-console-frontend.md) | Componente `CleanConsole` modal con header (fase + ETA + barra + throughput), log scrolleable con auto-scroll, animaciones framer-motion por línea, integración en `cache-page.tsx`. **El toast se mantiene.** | 09 |
| [`11-clean-visualizer-r3f.md`](11-clean-visualizer-r3f.md) | Visualización 3D 120×120 px en el header de la consola: anillo de progreso + halo + partículas orbitando. Color cambia por fase. Respeta frameloop="demand", powerPreference low-power, context-lost handlers. Fallback SVG. | 10 |
| [`12-eta-estimate-and-summary.md`](12-eta-estimate-and-summary.md) | Tiempo estimado en el PlanView **antes** de pulsar Limpiar (basado en throughput histórico). `CleanSummaryHero` al terminar con métricas (espacio liberado, duración, archivos, restore point, errores). Cerrar consola invalida queries. | 09, 10 |

## Recomendación de orden global

**Set A → Set B**. Dentro de Set B: 09 → 10 → 11 → 12 (estricto).

Concretamente:

1. **01** — proteger el shell (P0). ✅
2. **02** — auto-analyze al boot (P1, depende de 01). ✅
3. **08** — redundancias (P2, depende de 02). ✅
4. **05** — Three.js Context Lost (P1, independiente). ✅
5. **09** — pipeline de progreso enriquecido (P1).
6. **10** — componente `CleanConsole` (P1, depende de 09).
7. **11** — visualización 3D (P2, depende de 10 — sólo añade un slot
   visual; el resto del flujo funciona sin él).
8. **12** — estimación previa + resumen final (P1, depende de 09 y 10).

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
