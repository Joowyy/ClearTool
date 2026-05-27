# 12 — Estimación previa + resumen final + integración

> **Severidad:** 🟡 P1 — cierra el ciclo: el usuario sabe cuánto va a
> tardar **antes** de pulsar Limpiar y cuánto liberó **después**.
> **Modelo:** Sonnet 4.6.
> **Bloque:** 4 de 4 del set "consola de limpieza".
> **Depende de:**
> - [`09-progress-pipeline-backend.md`](09-progress-pipeline-backend.md)
>   (provee `get_throughput_stats` + `cache:summary`).
> - [`10-clean-console-frontend.md`](10-clean-console-frontend.md)
>   (provee `CleanConsole` con estados `complete | failed`).

## 1. Problema

Después de los docs 09-11 todavía faltan dos piezas:

1. **Antes** de pulsar Limpiar, el usuario no sabe cuánto va a tardar.
   El PlanView muestra "Limpiar 4.03 GB" pero no "~30 s estimados".
2. **Cuando termina**, la consola sólo dice "¡Listo!" sin métricas
   concretas. El usuario pidió explícitamente:
   > _"mostrar igual un resumen con el espacio total limpiado y el
   > tiempo demorado en limpiar todo"_.
3. Tras cerrar la consola, el flujo no vuelve limpio al PlanView con
   los nuevos totales (el `cache_background` ya lo refresca, falta
   atar cabos en el UI).

## 2. Objetivo

### 2.1 Estimación previa en el PlanView

```
Plan de limpieza
12 se pueden limpiar ahora · 9 al reiniciar · 35 ya vacías

[ Se libera ahora: 4.03 GB ]  [ Se libera al reiniciar: 1.06 GB ]
                  ⌛ Tiempo estimado: ~30 s                       ← NUEVO
```

### 2.2 Resumen al terminar (dentro de `CleanConsole`)

```
        ✓
   ¡Hecho!
   Has recuperado 4.03 GB en 28 s

   ┌─────────────────────────┐
   │ Archivos limpiados   8 234│
   │ Programados al reboot  127│
   │ Errores                 0│
   │ Punto de restauración  #42│
   └─────────────────────────┘

   [Ver log completo]  [Listo]
```

## 3. Estimación previa — frontend

### 3.1 Hook `useThroughputStats`

`src/features/cache-cleaner/use-throughput-stats.ts`:

```ts
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export interface ThroughputStats {
  samples: number[];
  meanBytesPerSec: number;
  p95BytesPerSec: number;
}

export function useThroughputStats() {
  return useQuery({
    queryKey: ["throughput-stats"],
    queryFn: () => invoke<ThroughputStats>("get_throughput_stats"),
    // Cambia poco; refrescar al volver a la pestaña es overkill.
    staleTime: 60_000,
  });
}
```

(Añadir el wrapper en `src/api/client.ts`: `getThroughputStats` que
invoca el comando.)

### 3.2 Mostrar la estimación en `PlanView`

Justo debajo de las dos pills "Se libera ahora" / "Se libera al
reiniciar", añadir:

```tsx
import { useThroughputStats } from "./use-throughput-stats";
import { formatDuration } from "../../lib/format";

function EstimatedTime({ bytes }: { bytes: number }) {
  const { data } = useThroughputStats();
  if (bytes === 0) return null;
  // Usamos p95 como "tiempo plausible" en lugar de la media,
  // para no quedar corto cuando el SSD está saturado.
  const bps = data?.p95BytesPerSec ?? 100 * 1024 * 1024;
  const secs = bytes / bps;

  // Cap inferior a 3 s — borrar 50 ubicaciones nunca es <3 s real.
  const estimate = Math.max(3, secs);

  return (
    <div className="flex items-center gap-1.5 text-xs text-ink-tertiary">
      <Clock className="h-3.5 w-3.5" />
      <span>
        Tiempo estimado:{" "}
        <span className="text-ink-secondary font-medium">
          ~{formatDuration(estimate)}
        </span>
        {data && data.samples.length === 0 && (
          <span className="text-ink-muted"> (primera limpieza, estimación aproximada)</span>
        )}
      </span>
    </div>
  );
}

// En el JSX del PlanView, después de las dos pills:
<div className="flex gap-4 text-sm items-center flex-wrap">
  <div className="px-3 py-1.5 rounded-lg bg-green-900/20 border border-green-800/40">
    <span className="text-green-400 font-medium">Se libera ahora: </span>
    <span className="text-muted-foreground">{formatBytes(plan.totalEstimatedBytes)}</span>
  </div>
  <div className="px-3 py-1.5 rounded-lg bg-yellow-900/20 border border-yellow-800/40">
    <span className="text-yellow-400 font-medium">Se libera al reiniciar: </span>
    <span className="text-muted-foreground">{formatBytes(plan.totalBlockedBytes)}</span>
  </div>
  <EstimatedTime bytes={plan.totalEstimatedBytes} />
</div>
```

## 4. Resumen final — frontend

### 4.1 Componente `CleanSummaryHero`

`src/features/cache-cleaner/clean-summary-hero.tsx`:

```tsx
import { motion } from "framer-motion";
import { CheckCircle2, AlertTriangle, Clock, Files, RotateCw, Shield, FileWarning } from "lucide-react";
import type { CleanSummaryPayload } from "../../api/events";
import { formatBytes, formatDuration } from "../../lib/format";

interface Props {
  summary: CleanSummaryPayload;
}

export function CleanSummaryHero({ summary }: Props) {
  const success = summary.success && summary.totalFilesFailed === 0;
  const durationSecs = summary.durationMs / 1000;

  return (
    <div className="space-y-5">
      <motion.div
        initial={{ scale: 0.7, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 220, damping: 18 }}
        className="flex flex-col items-center text-center gap-2"
      >
        {success ? (
          <CheckCircle2 className="h-14 w-14 text-emerald-400" />
        ) : (
          <AlertTriangle className="h-14 w-14 text-amber-400" />
        )}
        <h2 className="text-2xl font-semibold text-ink-primary">
          {success ? "¡Hecho!" : "Terminó con avisos"}
        </h2>
        <p className="text-ink-secondary">
          Has recuperado{" "}
          <span className="text-ink-primary font-semibold">
            {formatBytes(summary.totalBytesFreed)}
          </span>{" "}
          en{" "}
          <span className="text-ink-primary font-semibold">
            {formatDuration(durationSecs)}
          </span>
          .
        </p>
      </motion.div>

      <div className="grid grid-cols-2 gap-2 text-sm">
        <SummaryRow
          icon={Files}
          label="Archivos limpiados"
          value={summary.totalFilesDeleted.toLocaleString()}
        />
        {summary.totalFilesScheduledReboot > 0 && (
          <SummaryRow
            icon={RotateCw}
            label="Al reiniciar"
            value={summary.totalFilesScheduledReboot.toLocaleString()}
            hint={formatBytes(summary.totalBytesScheduledReboot)}
          />
        )}
        {summary.totalFilesFailed > 0 && (
          <SummaryRow
            icon={FileWarning}
            label="Con error"
            value={summary.totalFilesFailed.toLocaleString()}
            tone="warn"
          />
        )}
        {summary.restorePointSeq != null && (
          <SummaryRow
            icon={Shield}
            label="Punto de restauración"
            value={`#${summary.restorePointSeq}`}
          />
        )}
        <SummaryRow
          icon={Clock}
          label="Velocidad media"
          value={`${formatBytes(summary.meanThroughputBytesPerSec)}/s`}
        />
      </div>

      {summary.locationsWithErrors.length > 0 && (
        <div className="rounded-lg bg-amber-900/15 border border-amber-700/30 p-3 text-xs text-amber-300">
          <p className="font-medium mb-1">
            {summary.locationsWithErrors.length} ubicaci
            {summary.locationsWithErrors.length === 1 ? "ón" : "ones"} con errores
          </p>
          <ul className="space-y-0.5 text-amber-200/80">
            {summary.locationsWithErrors.slice(0, 4).map((id) => (
              <li key={id} className="font-mono">{id}</li>
            ))}
            {summary.locationsWithErrors.length > 4 && (
              <li className="text-amber-300/60">
                … y {summary.locationsWithErrors.length - 4} más (ver log)
              </li>
            )}
          </ul>
        </div>
      )}
    </div>
  );
}

function SummaryRow({
  icon: Icon,
  label,
  value,
  hint,
  tone,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
  hint?: string;
  tone?: "warn";
}) {
  const valueColor =
    tone === "warn" ? "text-amber-300" : "text-ink-primary";
  return (
    <div className="flex items-start gap-2 rounded-lg bg-bg-canvas/40 border border-edge-default/20 p-2.5">
      <Icon className="h-4 w-4 mt-0.5 text-ink-tertiary" />
      <div className="flex-1 min-w-0">
        <div className="text-ink-tertiary text-xs">{label}</div>
        <div className={`font-semibold tabular-nums ${valueColor}`}>{value}</div>
        {hint && <div className="text-ink-muted text-[11px]">{hint}</div>}
      </div>
    </div>
  );
}
```

### 4.2 Sustituir el placeholder del doc 10 dentro de `CleanConsole`

En `clean-console.tsx`, donde ahora dice "Hero del resumen — DETALLE
EN DOC 12":

```tsx
{(status.kind === "complete" || status.kind === "failed") && (
  <CleanSummaryHero summary={status.summary} />
)}
```

Y abajo, en lugar del único botón "Cerrar":

```tsx
<div className="flex justify-end gap-2">
  {(status.kind === "complete" || status.kind === "failed") && (
    <Button variant="ghost" onClick={() => setShowFullLog((s) => !s)}>
      {showFullLog ? "Ocultar log" : "Ver log completo"}
    </Button>
  )}
  <Button
    onClick={onClose}
    disabled={isRunning}
    variant={status.kind === "complete" ? "default" : "outline"}
  >
    {isRunning ? "Limpiando…" : "Listo"}
  </Button>
</div>
```

El log scrolleable (`CleanConsoleLog`) ya no se muestra siempre — sólo
durante `running` y bajo demanda en `complete | failed` cuando el
usuario pulsa "Ver log completo". Eso despeja la vista del resumen.

## 5. Cerrar la consola correctamente

```ts
function handleClose() {
  setConsoleOpen(false);
  // Reset del stream para que la próxima limpieza empiece limpia.
  cleanStream.reset();
  // Forzar refresh del plan caliente con los nuevos totales.
  void qc.invalidateQueries({ queryKey: ["cache-plan-warm"] });
  // Y, por consistencia, refrescar throughput-stats (acaba de haber
  // una nueva muestra persistida en backend).
  void qc.invalidateQueries({ queryKey: ["throughput-stats"] });
}
```

## 6. Toast — se mantiene

> _"El toast de la esquina inferior derecha me gusta, mantenle"_.

El `toast.success` en `onSuccess` de `executeMutation` (definido en
doc 10) **no se toca**. Convive con el resumen del modal:

- Modal con resumen rico = el usuario está mirando la app.
- Toast = se queda 4 s y desaparece, sirve para confirmación.

Si en el futuro se cierra la consola con un atajo rápido y el usuario
no llegó a ver el resumen, el toast lo cubre.

## 7. Resumen accesible (a11y)

```tsx
<motion.div role="dialog" aria-modal="true" aria-labelledby="clean-console-title" ...>
  {/* dentro del header del estado complete/failed */}
  <h2 id="clean-console-title" className="text-2xl font-semibold">
    {success ? "Limpieza completada" : "Limpieza con avisos"}
  </h2>
```

(El título visible "¡Hecho!" puede quedarse como visual; el `aria-label`
del dialog usa una versión menos exclamativa para screen readers.)

## 8. Persistencia adicional (opcional, sin bloquear el merge)

Guardar el resumen de las últimas 10 limpiezas en
`%APPDATA%\ClearTool\clean-history.json` para una futura pestaña
"Historial" en Auditoría. **No** se implementa aquí; sólo dejar
preparado:

- En `domain/throughput_stats.rs` ya hay precedente — puede crearse un
  `domain/clean_history.rs` con el mismo patrón.
- Bloqueado para futuro doc 13 si el usuario lo pide.

## 9. Criterio de done

- [ ] `EstimatedTime` se muestra en el PlanView con `formatDuration` y
      la nota "(primera limpieza, estimación aproximada)" si no hay
      muestras todavía.
- [ ] Al terminar `executeMutation` con éxito, la consola muestra
      `CleanSummaryHero` con la animación de scale-in del icono.
- [ ] Las métricas mostradas coinciden con `summary.*` 1:1.
- [ ] `Errores con N ubicaciones` se muestra sólo si la hay.
- [ ] `Punto de restauración` se muestra sólo si `restorePointSeq != null`.
- [ ] `Velocidad media` se muestra siempre (es el dato que alimenta
      futuras estimaciones).
- [ ] El log se oculta por defecto al pasar a `complete | failed`,
      desplegable con "Ver log completo".
- [ ] El toast inferior derecho se sigue mostrando (no se quita).
- [ ] Cerrar la consola invalida `cache-plan-warm` y
      `throughput-stats` (refresca el PlanView).
- [ ] La próxima limpieza arranca con `cleanStream.reset()` ya
      ejecutado (log vacío, estado idle).

## 10. Riesgos / efectos secundarios

- **Estimación poco precisa la primera vez** (no hay muestras): el copy
  lo dice explícitamente. Se irá refinando solo.
- **`p95` sobre 2 muestras es ruidoso**: con <5 muestras conviene caer a
  `mean`. El backend ya devuelve ambos; el frontend puede elegir según
  `samples.length`:
  ```ts
  const bps = (data?.samples?.length ?? 0) >= 5
    ? data!.p95BytesPerSec
    : data?.meanBytesPerSec ?? 100 * 1024 * 1024;
  ```
- **Toast + modal**: si el usuario cierra el modal antes de que aparezca
  el toast (latencia entre `summary` y `onSuccess` de TanStack Query),
  ambos pueden solaparse. No es problema — son redundantes
  intencionadamente.
- **Race**: si el backend tarda en emitir `cache:summary` después de
  resolver la promesa de `execute_clean_plan`, el `CleanConsole` puede
  quedarse en `running` aunque la mutación ya terminó. Mitigación:
  forzar `dispatch({ type: "summary", payload: { ... } })` cuando
  `onSuccess` de la mutación corra y no hayamos visto evento aún —
  hidratar desde `r` (CleanReportV2) + `v` (VerifyReport).
