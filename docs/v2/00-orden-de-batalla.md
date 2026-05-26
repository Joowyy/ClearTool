# Orden de batalla — qué hacer y en qué orden

Mapea los milestones del roadmap (`.claude/specs/v2/09-roadmap.md`) a las carpetas de esta guía.

## Reglas de oro

1. **P0 antes que P1, P1 antes que P2.**
2. **No mezclar refactor con feature en el mismo PR.**
3. **Cada milestone es releasable como `0.X` en GitHub Releases (canal beta).**
4. **Si un paso bloquea otro, hacer el bloqueador primero aunque rompa el orden numérico.**

## M1 — UX no sangra (semanas 1-3)

Versión target: **v0.2.0**.

Trabajo:
- `01-error-handling/` completo (5 pasos)

Por qué primero: sin un modelo de errores decente, todos los módulos siguientes heredan `[object Object]`. Cualquier feature nueva que añadas durante M2+ tendrá que volver atrás a aplicar el patrón.

**No empezar M2 hasta que M1 esté hecho.**

## M2 — Cache funcional (semanas 4-8)

Versión target: **v0.3.0**.

Trabajo (en este orden):
1. `03-process-manager/` (6 pasos) — primero, porque cache lo necesita
2. `02-cache-engine/` (8 pasos) — usa el process manager

Por qué en este orden: cache cleaner v2 depende de `who_locks_path` y `close_gracefully` del process manager. Sin esos, no puede analizar bloqueadores.

**No empezar M3 hasta que estas dos carpetas estén hechas.**

## M3 — Contenido y módulos sólidos (semanas 9-14)

Versión target: **v0.4.0**.

Trabajo (paralelo posible entre estas tres):
- `04-debloat-catalog/` (6 pasos) — empezar curación humana en paralelo
- `05-startup-manager/` (4 pasos)
- `06-boot-cleanup/` (3 pasos)
- `09-privacy-hardening/` (3 pasos)
- `10-app-reset/` (2 pasos) — pequeño, encaja al final

Algunos de estos son curación humana (catálogos) que pueden correr en paralelo a coding de otros.

## M4 — Brilla (semanas 15-18)

Versión target: **v0.5.0 (RC1)**.

Trabajo:
- `11-ui-refactor/` (8 pasos) — el grande
- `07-disk-analyzer/` (3 pasos) — feature wow
- `08-network-utilities/` (2 pasos) — quick win, hacer al final

Hacer UI refactor PRIMERO, porque los módulos nuevos (disk analyzer, network) se construirán encima del nuevo shell con sidebar.

## M5 — Distribución (semanas 19-21)

Versión target: **v1.0.0** 🎉.

Trabajo:
- `12-distribution/` (6 pasos)
- `13-testing-ci/` (4 pasos) — empezar antes en realidad, pero formalizar aquí

Empezar `12-distribution/01-azure-trusted-signing.md` **AL INICIO de M3** porque la aprobación de identity tarda 1-3 días.

## Pasos transversales (correr en paralelo a todo)

- `13-testing-ci/` debería arrancar en M2 (después de error handling) con tests del nuevo error model.
- `13-testing-ci/04-github-actions-build.md` (CI básico) puede ir en M1 ya.

## Diagrama Gantt simplificado

```
Semana:    1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21
           ────────────────────────────────────────────────────────────
M1 errors  ▓▓▓▓▓
M2 process       ▓▓▓▓▓▓
M2 cache               ▓▓▓▓▓▓▓
M3 debloat                     ▓▓▓▓▓▓▓▓▓▓
M3 startup                     ▓▓▓▓▓
M3 boot                        ▓▓▓
M3 privacy                            ▓▓▓
M4 UI                                       ▓▓▓▓▓▓▓
M4 disk                                            ▓▓▓
M4 network                                            ▓▓
M5 dist                                                   ▓▓▓▓
M5 cert (paralelo desde M3)         ████ ████ ████ ████ ████ ▓▓▓▓
CI tests       ▓▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓
```

## Cómo saber si vas adelantado / atrasado

Al final de cada milestone, evaluar:
- ¿El criterio de "done" del milestone está cumplido al 100%?
- Si no, ¿qué porcentaje? Si es <70% no pasar al siguiente.
- ¿Hay deuda técnica grave que pague entrando al siguiente milestone?

## Reglas para saltarse pasos

Algunos pasos pueden **omitirse** si:
- No aplican a tu hardware/uso (ej. `08-network-utilities` si nadie pide reset Winsock).
- Tienen alternativa de scope (ej. usar Tauri Updater built-in vs servidor propio).

Marcar el paso saltado en su checkbox con `[~]` y razón corta. No borrar — la información del paso sigue siendo útil de referencia.
