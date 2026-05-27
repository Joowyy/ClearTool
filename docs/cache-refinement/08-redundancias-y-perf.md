# 08 — Redundancias y rendimiento del módulo Caché

> **Severidad:** 🟢 P2 — pulido. El usuario pide que la app "no
> moleste". Esta nota recopila ineficiencias y ruido a eliminar.

## 1. Hallazgos

### 1.1 `analyze_locations` repite cálculo de tamaños

Cuando se llama `scan(ids)` antes del analyze, y luego `analyze_locations(ids)`,
ambos recorren el árbol de cada caché — `scan_path_stats` se ejecuta dos
veces. **Fusionar**: el `CleanPlan` ya tiene bytes y file_count en
`ReadyLocation`; no necesitamos `scan(ids)` separado en el flujo del
PlanView.

**Fix**: borrar la llamada a `scan()` en el frontend si ya tenemos plan;
el `SelectionView` puede mostrar estimaciones del catálogo (size
hint) sin escanear hasta que el usuario seleccione y analice.

### 1.2 `who_locks_path` con muestreo de 200 archivos por carpeta

`src-tauri/src/platform/processes.rs:446-563` registra hasta 200
archivos por path en Restart Manager. Para caches como `inetcache`
con miles de archivos pequeños, eso son **200 syscalls extra
(`stat`)** por análisis, sólo para detectar lockers que igual no son
relevantes.

**Fix**:
- Bajar a 50 archivos cuando hay >5000 entries.
- Cachear el resultado de `who_locks_path` por path durante 30 segundos
  (los lockers no cambian instantáneamente).

### 1.3 `useQueryClient.invalidateQueries(["cache-locations"])` tras limpieza

`src/features/cache-cleaner/cache-page.tsx:613`:

```ts
void qc.invalidateQueries({ queryKey: ["cache-locations"] });
```

Eso refresca el catálogo entero (lista de ubicaciones) cuando lo que
queremos invalidar es el **plan**, no el catálogo. El catálogo es un
JSON estático que nunca cambia en runtime. Cambiar por
`invalidateQueries({ queryKey: ["cache-plan-warm"] })` (cuando
implementemos el doc 02).

### 1.4 `refetchOnWindowFocus` aleatorio

La query `["pending-renames"]` en `PendingRenamesPanel` tiene
`refetchOnWindowFocus: false`, perfecto. Pero `["cache-locations"]`
no lo desactiva → cada vez que el usuario vuelve a la ventana, se
re-fetcha. Aunque sea cheap, es ruido. **Fix**: desactivar
`refetchOnWindowFocus` globalmente en el `QueryClient` y activarlo
manualmente donde sí lo queramos.

`src/main.tsx` (o donde se cree el `QueryClient`):

```ts
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 30_000,
    },
  },
});
```

### 1.5 Emisores de progreso intensivos

`emit_progress` en `execute_plan` emite eventos por cada locación
("Limpiando: ...", "Completado: ..."). Si tenemos 50 ubicaciones, son
~150 events en pocos segundos → React Query / Tauri IPC se saturan y
puede causar el Context Lost del 3D (doc 05).

**Fix**: throttle a 1 evento cada 200ms en el frontend. El backend
puede seguir emitiendo, pero el `listen` del frontend agrega y aplica
en bursts.

```ts
import { listen } from "@tauri-apps/api/event";
import throttle from "lodash.throttle";

const apply = throttle((batch: Event[]) => setLogs(prev => [...prev, ...batch]), 200);
let queue: Event[] = [];

listen("cache:progress", (e) => {
  queue.push(e.payload as Event);
  apply([...queue]);
  queue = [];
});
```

(O bajar a `useTransition` para el render si no queremos meter
lodash.)

### 1.6 Renders innecesarios en `PlanView`

`PlanView` se re-renderiza cuando cambia `plan`, `report` o `verify`.
Eso es correcto. **Pero** los hijos (`SectionAccordion`, badges,
`Button`) no están memoizados. Cuando llegan eventos de progreso, todo
el PlanView se redibuja.

**Fix mínimo**: envolver los accordions en `React.memo` y pasar sólo
las props necesarias.

### 1.7 `analyzeCacheLocations` no expone progreso

El usuario ve un spinner sin pista de qué está pasando. Para cachés
grandes (WinSxS, prefetch, INetCache), el analyze tarda. Sugerencia:

- Backend: `analyze_locations` emite `cache:analyze-progress` con
  `{ current_id, current_index, total }`.
- Frontend: barra de progreso fina abajo durante el análisis.

(Lo dejamos como nice-to-have, sólo si tras los demás fixes el primer
análisis sigue sintiéndose lento.)

### 1.8 `Cargo.toml` — `tokio` con `features = ["full"]`

Ya está corregido en este branch (`Cargo.toml:60`):

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync", "fs", "process"] }
```

Bien. No bajar más, ya está mínimo.

### 1.9 React DevTools warning

```
Download the React DevTools for a better development experience
```

Es un mensaje informativo. No hay nada que arreglar — sólo aparece en
dev mode. Cuando se construya el bundle de release, no aparece.

## 2. Plan de acción

| Fix | Esfuerzo | Impacto |
|---|---|---|
| 1.4 — `refetchOnWindowFocus: false` global | XS | Quita ruido de IPC |
| 1.3 — Invalidar `cache-plan-warm`, no `cache-locations` | XS | Coherencia |
| 1.5 — Throttle de progreso | S | Reduce presión GPU/CPU |
| 1.1 — Borrar scan() previo al analyze en frontend | S | -200ms por análisis |
| 1.2 — Cachear `who_locks_path` | M | Análisis ~2x más rápido |
| 1.6 — Memoizar acordeones | S | UI más fluida durante limpieza |
| 1.7 — Progreso del analyze | M | UX (no urgente) |

## 3. Criterio de done

- [ ] No hay re-fetch de cache-locations al volver a la ventana.
- [ ] Los eventos de progreso no saturan la consola/IPC.
- [ ] El análisis de 50 ubicaciones tarda < 2s en SSD.
- [ ] Tras limpiar, el plan se recalcula automáticamente sin que el
      usuario pulse nada (combinado con doc 02).

## 4. Riesgos

- Throttle de progreso: el usuario puede sentir que la UI "salta" si el
  intervalo es muy alto. 200ms es el sweet spot por la regla de
  Doherty.
- Memoización: cuidado con cambios por referencia (e.g. `plan.ready`)
  que invalidan memo si no se compara por contenido. Usar
  `React.memo(Component, customCompare)` cuando aplique.
