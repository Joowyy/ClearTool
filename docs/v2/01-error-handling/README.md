# 01 — Error Handling (M1, P0)

**Objetivo**: que ningún error llegue al usuario como `[object Object]`. Toda la cadena de errores backend → frontend → UI debe ser tipada, legible y accionable.

**Spec de referencia**: `.claude/specs/v2/01-error-model-fix.md`.

## Por qué este es el primer paso de TODO

Cada feature que añades hoy hereda el bug de errores. Si M2 introduce el cache engine v2 con `who_locks_path`, sus errores también van a salir como `[object Object]` a menos que primero arregles la base. Por eso esto va antes que nada.

## Pasos en orden

1. [01 — Crear `src/lib/errors.ts`](01-crear-errors-lib.md) — utilidad de normalización
2. [02 — Instalar sonner y montar Toaster](02-instalar-sonner.md)
3. [03 — Crear wrapper `src/lib/toast.ts`](03-toast-wrapper.md)
4. [04 — ErrorBoundary global en el router](04-error-boundary.md)
5. [05 — Sweep: reemplazar `String(err)` en todo el código](05-reemplazar-string-err.md)
6. [06 — Backend: `AppError::with_context` para enriquecer mensajes](06-rust-with-context.md)

## Prerequisitos

- v0.1 actual buildeando sin errores TS.
- React Router v6 instalado (ya está).
- `@tanstack/react-query` instalado (ya está).

## Criterio de done (área completa)

- [x] `src/lib/errors.ts` existe con `normalizeError`, `formatError`, `formatErrorWithKind`.
- [x] `sonner` instalado y `<Toaster />` en `<AppShell>`.
- [x] `src/lib/toast.ts` exporta `toast.error/success/info/warning` aceptando opcionalmente un `err: unknown`.
- [x] `ErrorBoundary` registrado en el router como `errorElement`.
- [x] Cero ocurrencias de `String(err)` o `${err}` para errores en JSX (grep verifica).
- [x] Los 3 errores conocidos (debloat detect, restore list, cache clean) muestran texto legible con acción cuando aplica.
- [x] `AppError::with_context` añadido en Rust y usado en mínimo 3 sitios.

## Tiempo total estimado

12-16 horas reparto:
- Paso 01: 2h
- Paso 02: 30min
- Paso 03: 1h
- Paso 04: 1.5h
- Paso 05: 4-6h (depende de cuántos `String(err)` aparezcan)
- Paso 06: 2h
