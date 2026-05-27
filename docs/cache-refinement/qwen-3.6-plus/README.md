# Tareas fáciles — Qwen 3.6 Plus

> Esta carpeta concentra las tareas del refinamiento de caché que son
> **mecánicas, localizadas y de bajo riesgo**: edición de config,
> reemplazo de un elemento HTML, ajuste de strings, activación de flags.
> Cualquier modelo capaz puede aplicarlas en un paso siguiendo el doc.

## Criterios para asignar aquí

- Cambios en **un único archivo** o muy pocos.
- Sin lógica nueva — sólo **reemplazo de valor, atributo o markup**.
- **Sin riesgo de regresión sistémica** (no toca procesos, FS ni
  registry).
- El "criterio de done" se verifica visualmente o con devtools en menos
  de un minuto.

## Sets en esta carpeta

### Set A — Refinamiento base (estado: ✅ implementado)

| Doc | Por qué es fácil |
|---|---|
| [`03-button-in-button.md`](03-button-in-button.md) | Reemplazar un `<button>` por `<div role="button" tabIndex={0}>` con `onKeyDown`. Un sólo componente, un solo archivo. Verificable en devtools al instante. |
| [`04-react-router-future-flags.md`](04-react-router-future-flags.md) | Añadir un objeto `{ future: { v7_*: true } }` al segundo argumento de `createBrowserRouter`. Una edición en `src/router.tsx`. |
| [`06-window-size.md`](06-window-size.md) | Editar `width`, `height`, `minWidth`, `minHeight`, `center` en `src-tauri/tauri.conf.json`. Opcionalmente añadir el plugin `tauri-plugin-window-state`. |
| [`07-plan-view-copy.md`](07-plan-view-copy.md) | Cambiar strings hardcoded en `src/features/cache-cleaner/cache-page.tsx`. Tabla de mapping antes→después incluida en el doc. Volumen mayor pero sin lógica. |

### Set C — Quality of life de la consola (estado: ⏳ pendiente)

| Doc | Por qué es fácil |
|---|---|
| [`08-log-reset-between-runs.md`](08-log-reset-between-runs.md) | Añadir `cleanStream.reset()` en 3 puntos de `cache-page.tsx`. Bug funcional: hoy el log se acumula entre limpiezas en la misma sesión. |
| [`09-log-readability-no-emoji.md`](09-log-readability-no-emoji.md) | Cambiar iconos del log a sólo ✓/✗/▲/·, subir tipografía a `font-sans text-[13px]`, alinear columnas. Un único componente afectado. |

## Recomendación de orden

Cualquiera. Son independientes entre sí y no se pisan con las tareas
difíciles. Pueden hacerse en paralelo o todas seguidas.

Si hay que priorizar dentro del Set C:

1. **08** — bug funcional reportado por el usuario; arreglo trivial.
2. **09** — el usuario lo pidió explícitamente para tener la consola
   "más legible".

## Convenciones para esta carpeta

- Verificar visualmente en `npm run tauri dev` antes de marcar como
  hecho.
- Si un doc dice "snippet pegable", se pega tal cual; no reinventar la
  solución.
- Si surge una duda no resuelta por el doc, **escalar al doc
  equivalente en `sonnet-4.6/`** o preguntar — no improvisar en código
  destructivo.
