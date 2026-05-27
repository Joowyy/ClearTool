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

## Índice

| Doc | Por qué es difícil |
|---|---|
| [`01-shell-safe-close.md`](01-shell-safe-close.md) | Bug de seguridad real (mató Explorer del usuario). Toca whitelist de PIDs protegidos, `suggest_action` y `execute_plan`. Defensa en profundidad en 3 capas + tests Rust. Pasa por `security-auditor` antes de mergear. |
| [`02-auto-analyze-on-boot.md`](02-auto-analyze-on-boot.md) | Nuevo módulo `cache_background.rs` con `once_cell::Lazy<RwLock<...>>`, lifecycle de Tauri (`setup`), 2 comandos IPC nuevos, invalidación post-clean, sincronización con frontend (React Query + `useEffect`). Hay que pensar el TTL y los races. |
| [`05-three-context-lost.md`](05-three-context-lost.md) | Three.js/R3F internals, `frameloop="demand"`, `webglcontextlost` handler, `powerPreference`, fallback 2D, detección de WebGL. Requiere conocer el ciclo de render de R3F a fondo y saber cuándo `invalidate()` es necesario. |
| [`08-redundancias-y-perf.md`](08-redundancias-y-perf.md) | Optimización multi-capa: throttle de eventos IPC, caché de `who_locks_path`, memoización selectiva, defaults globales de React Query, eliminar scan duplicado. Requiere razonar sobre el impacto de cada cambio en CPU/GPU/IPC. |

## Recomendación de orden

1. **01** — proteger el shell antes que nada (P0).
2. **02** — auto-analyze al boot (P1, depende parcialmente de 01 porque
   el primer plan no debe sugerir cerrar Explorer).
3. **08** — redundancias (P2, depende de 02 porque optimiza el flujo
   nuevo).
4. **05** — Three.js Context Lost (P1, independiente; puede ir en
   paralelo).

## Convenciones para esta carpeta

- Antes de mergear cualquier doc 0X, ejecutar `cargo test` con todos los
  tests nuevos pasando.
- Si el doc tiene un test marcado como obligatorio en su sección "Criterio
  de done", **no es opcional**.
- Cambios en backend Rust requieren revisión de `security-auditor` (ver
  CLAUDE.md → mapa de subagentes).
