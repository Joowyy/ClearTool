# 10 — Componente `CleanConsole` (frontend, sin 3D)

> **Severidad:** 🟡 P1 — sustituye el spinner mudo por una consola
> visual y legible.
> **Modelo:** Sonnet 4.6.
> **Bloque:** 2 de 4 del set "consola de limpieza".
> **Depende de:** [`09-progress-pipeline-backend.md`](09-progress-pipeline-backend.md).

## 1. Problema

Cuando el usuario pulsa **Limpiar**, lo único que ve durante segundos
(o minutos) es un botón con un spinner pequeño. No sabe:

- En qué paso está (¿restore point? ¿cerrando procesos? ¿limpiando?).
- Cuánto queda.
- Qué está haciendo ahora mismo.
- Si se rompió algo en silencio.

El backend ya emite eventos ricos (doc 09). El frontend tiene que
hacerlos visibles.

## 2. Objetivo

Mostrar, al pulsar Limpiar, una **consola modal en overlay** con:

```
┌─────────────────────────────────────────────────────────┐
│  ╭─╮  Limpiando caché                                   │
│  │%│  Fase: Limpiando archivos (3 de 12)                │
│  ╰─╯  Tiempo restante: ~2 min 30 s · 320 MB/s          │
│                                                         │
│  ████████████████░░░░░░░░░░░░░░░░░░░░░░░ 42 %         │
│  1.7 GB / 4.0 GB · 8 234 archivos                       │
│                                                         │
│  ┌────────────────────────────────────────────────┐    │
│  │ 17:42:13  INFO   ✓  Microsoft Edge cache       │    │
│  │ 17:42:14  INFO   ✓  Chrome user data           │    │
│  │ 17:42:15  WARN   ⚠  Spotify cache (1 file ...) │    │
│  │ 17:42:16  INFO   ✓  Windows temp               │    │
│  │ 17:42:17  INFO   ✓  Prefetch                   │    │
│  │ 17:42:18  INFO   ▸  Limpiando: WinSxS backup  │    │
│  │ 17:42:24  INFO   ✓  WinSxS backup              │    │
│  │ ▸ scroll automático                            │    │
│  └────────────────────────────────────────────────┘    │
│                                                         │
│                              [ Cerrar (disabled) ]      │
└─────────────────────────────────────────────────────────┘
```

(El círculo `%` de la esquina superior izquierda lo dibuja el doc 11
— por ahora un placeholder o anillo CSS.)

## 3. Archivos nuevos / tocados

| Path | Tipo |
|---|---|
| `src/features/cache-cleaner/clean-console.tsx` | NUEVO — componente principal |
| `src/features/cache-cleaner/clean-console-log.tsx` | NUEVO — sub-componente del log scrolleable |
| `src/features/cache-cleaner/clean-console-header.tsx` | NUEVO — fase + ETA + barra |
| `src/features/cache-cleaner/use-clean-stream.ts` | NUEVO — hook que escucha los 4 eventos |
| `src/features/cache-cleaner/cache-page.tsx` | MODIFICADO — integra el modal |
| `src/api/events.ts` | YA tocado en doc 09 |

## 4. Hook `useCleanStream`

`src/features/cache-cleaner/use-clean-stream.ts`:

```ts
import { useCallback, useEffect, useReducer, useRef } from "react";
import { useTauriEvent } from "../../hooks/use-tauri-event";
import type {
  CleanLogLinePayload,
  CleanProgressV2Payload,
  CleanPhasePayload,
  CleanSummaryPayload,
} from "../../api/events";

export type CleanState =
  | { kind: "idle" }
  | { kind: "running"; progress: CleanProgressV2Payload; phase: CleanPhasePayload["phase"] }
  | { kind: "complete"; summary: CleanSummaryPayload }
  | { kind: "failed"; summary: CleanSummaryPayload };

interface State {
  status: CleanState;
  log: CleanLogLinePayload[];
  /** runId actual; ignoramos eventos de otros runs (defensa anti zombies). */
  runId: string | null;
}

type Action =
  | { type: "start"; runId: string }
  | { type: "line"; payload: CleanLogLinePayload }
  | { type: "progress"; payload: CleanProgressV2Payload }
  | { type: "phase"; payload: CleanPhasePayload }
  | { type: "summary"; payload: CleanSummaryPayload }
  | { type: "reset" };

const MAX_LOG = 500;

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "start":
      return { status: { kind: "idle" }, log: [], runId: action.runId };
    case "line":
      // sólo aceptamos líneas del run actual (si lo conocemos)
      return {
        ...state,
        log:
          state.log.length >= MAX_LOG
            ? [...state.log.slice(-MAX_LOG + 1), action.payload]
            : [...state.log, action.payload],
      };
    case "progress":
      if (state.runId && action.payload.runId !== state.runId) return state;
      return {
        ...state,
        status: {
          kind: "running",
          progress: action.payload,
          phase: action.payload.phase,
        },
      };
    case "phase":
      if (state.runId && action.payload.runId !== state.runId) return state;
      if (state.status.kind !== "running") return state;
      return {
        ...state,
        status: { ...state.status, phase: action.payload.phase },
      };
    case "summary":
      if (state.runId && action.payload.runId !== state.runId) return state;
      return {
        ...state,
        status: {
          kind: action.payload.success ? "complete" : "failed",
          summary: action.payload,
        },
      };
    case "reset":
      return { status: { kind: "idle" }, log: [], runId: null };
  }
}

export function useCleanStream() {
  const [state, dispatch] = useReducer(reducer, {
    status: { kind: "idle" },
    log: [],
    runId: null,
  });

  useTauriEvent("cache:line", (p) => dispatch({ type: "line", payload: p as CleanLogLinePayload }));
  useTauriEvent("cache:progress-v2", (p) => dispatch({ type: "progress", payload: p as CleanProgressV2Payload }));
  useTauriEvent("cache:phase", (p) => dispatch({ type: "phase", payload: p as CleanPhasePayload }));
  useTauriEvent("cache:summary", (p) => dispatch({ type: "summary", payload: p as CleanSummaryPayload }));

  const start = useCallback((runId: string) => dispatch({ type: "start", runId }), []);
  const reset = useCallback(() => dispatch({ type: "reset" }), []);

  return { ...state, start, reset };
}
```

**Nota sobre el `runId`**: el backend lo incluye en cada payload (doc 09)
y el frontend lo conoce porque al pulsar Limpiar genera uno y se lo
manda al backend, o lo recibe en el primer `cache:progress-v2`. Si
se prefiere, puede dejarse como `null` y aceptar el primer runId visto.

## 5. Componente `CleanConsoleHeader`

`src/features/cache-cleaner/clean-console-header.tsx`:

```tsx
import { CleanProgressV2Payload } from "../../api/events";
import { formatBytes, formatDuration } from "../../lib/format";

interface Props {
  progress: CleanProgressV2Payload;
  phaseLabel: string;
}

const PHASE_LABELS: Record<string, string> = {
  preparing: "Preparando",
  creatingRestorePoint: "Creando punto de restauración",
  closingProcesses: "Cerrando apps bloqueantes",
  cleaning: "Limpiando archivos",
  schedulingReboot: "Programando para reinicio",
  verifying: "Verificando resultado",
  complete: "Completado",
  failed: "Error",
};

export function CleanConsoleHeader({ progress, phaseLabel }: Props) {
  const pct =
    progress.totalEstimatedBytes > 0
      ? Math.min(100, (progress.bytesFreed / progress.totalEstimatedBytes) * 100)
      : 0;

  return (
    <div className="space-y-3">
      <div>
        <h2 className="text-xl font-semibold text-ink-primary">
          Limpieza en curso
        </h2>
        <p className="text-sm text-ink-secondary mt-1">
          {phaseLabel}
          {progress.currentLocationDisplayName ? (
            <span className="text-ink-tertiary"> · {progress.currentLocationDisplayName}</span>
          ) : null}
          {progress.totalLocations > 0 ? (
            <span className="text-ink-tertiary">
              {" "}({progress.currentLocationIndex} de {progress.totalLocations})
            </span>
          ) : null}
        </p>
      </div>

      <div className="flex items-baseline gap-3 text-sm">
        <span className="text-ink-secondary">
          Tiempo restante:{" "}
          <span className="text-ink-primary font-medium">
            {progress.etaSecs != null
              ? `~${formatDuration(progress.etaSecs)}`
              : "calculando…"}
          </span>
        </span>
        <span className="text-ink-tertiary">·</span>
        <span className="text-ink-secondary">
          {formatBytes(progress.throughputBytesPerSec)}/s
        </span>
      </div>

      <div className="space-y-1">
        <div className="h-2 bg-edge-default/30 rounded-full overflow-hidden">
          <div
            className="h-full bg-emerald-500 transition-[width] duration-200 ease-out"
            style={{ width: `${pct}%` }}
          />
        </div>
        <div className="flex justify-between text-xs text-ink-tertiary">
          <span>
            {formatBytes(progress.bytesFreed)} / {formatBytes(progress.totalEstimatedBytes)}
            {" · "}
            {progress.filesDeleted.toLocaleString()} archivos
          </span>
          <span>{pct.toFixed(1)} %</span>
        </div>
      </div>
    </div>
  );
}
```

**Helpers nuevos** en `src/lib/format.ts` (o donde ya viva `formatBytes`):

```ts
/** "2 min 30 s", "45 s", "1 h 12 min". Entrada: segundos. */
export function formatDuration(secs: number): string {
  if (!isFinite(secs) || secs <= 0) return "0 s";
  if (secs < 60) return `${Math.round(secs)} s`;
  if (secs < 3600) {
    const m = Math.floor(secs / 60);
    const s = Math.round(secs % 60);
    return s ? `${m} min ${s} s` : `${m} min`;
  }
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  return m ? `${h} h ${m} min` : `${h} h`;
}
```

## 6. Componente `CleanConsoleLog`

`src/features/cache-cleaner/clean-console-log.tsx`:

```tsx
import { useEffect, useRef, useState } from "react";
import { CheckCircle2, AlertTriangle, XCircle, Activity } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import type { CleanLogLinePayload } from "../../api/events";

interface Props {
  log: CleanLogLinePayload[];
}

const ICONS = {
  info: { Icon: Activity, className: "text-sky-400" },
  success: { Icon: CheckCircle2, className: "text-emerald-400" },
  warn: { Icon: AlertTriangle, className: "text-amber-400" },
  error: { Icon: XCircle, className: "text-rose-400" },
} as const;

export function CleanConsoleLog({ log }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  // Scroll al final cuando autoScroll está activo y llega línea nueva.
  useEffect(() => {
    if (!autoScroll || !containerRef.current) return;
    containerRef.current.scrollTop = containerRef.current.scrollHeight;
  }, [log.length, autoScroll]);

  // Detectar si el usuario hizo scroll manual para desactivar el follow.
  function onScroll() {
    if (!containerRef.current) return;
    const el = containerRef.current;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 24;
    setAutoScroll(nearBottom);
  }

  return (
    <div
      ref={containerRef}
      onScroll={onScroll}
      className="h-64 overflow-y-auto rounded-lg bg-bg-canvas/60 border border-edge-default/30 p-3 font-mono text-xs text-ink-secondary"
    >
      <AnimatePresence initial={false}>
        {log.map((line) => {
          const { Icon, className } = ICONS[line.level as keyof typeof ICONS] ?? ICONS.info;
          const time = new Date(line.timestampMs).toLocaleTimeString();
          return (
            <motion.div
              key={`${line.timestampMs}-${line.location}-${line.message.slice(0, 12)}`}
              initial={{ opacity: 0, x: -4 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.15 }}
              className="flex items-start gap-2 py-0.5"
            >
              <span className="text-ink-muted">{time}</span>
              <Icon className={`h-3.5 w-3.5 mt-0.5 shrink-0 ${className}`} />
              <span className="text-ink-tertiary truncate" title={line.location}>
                {line.location}
              </span>
              <span className="flex-1">{line.message}</span>
            </motion.div>
          );
        })}
      </AnimatePresence>

      {!autoScroll && (
        <button
          className="sticky bottom-1 left-1/2 -translate-x-1/2 text-[10px] px-2 py-0.5 rounded bg-ink-primary/10 text-ink-primary hover:bg-ink-primary/20"
          onClick={() => setAutoScroll(true)}
        >
          ↓ Ir al final
        </button>
      )}
    </div>
  );
}
```

**Rendimiento del log**: `motion.div` por línea sería costoso a 500
líneas. Si vemos jank, dos opciones:

1. Limitar las líneas animadas a las últimas 30 (resto sin animación).
2. Virtualizar con `react-window` (overkill para 500 líneas).

Empezar por la opción 1 si hace falta.

## 7. Componente principal `CleanConsole`

`src/features/cache-cleaner/clean-console.tsx`:

```tsx
import { useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { CleanConsoleHeader } from "./clean-console-header";
import { CleanConsoleLog } from "./clean-console-log";
import { Button } from "../../components/ui/button";
import { useCleanStream } from "./use-clean-stream";

interface Props {
  open: boolean;
  onClose: () => void;
  /** Si está corriendo la mutación, no permitir cerrar. */
  isRunning: boolean;
}

const PHASE_LABEL: Record<string, string> = {
  preparing: "Preparando",
  creatingRestorePoint: "Creando punto de restauración del sistema",
  closingProcesses: "Cerrando apps bloqueantes",
  cleaning: "Limpiando archivos",
  schedulingReboot: "Programando archivos para el próximo reinicio",
  verifying: "Verificando que se ha liberado el espacio",
  complete: "¡Listo!",
  failed: "Algo salió mal",
};

export function CleanConsole({ open, onClose, isRunning }: Props) {
  const { status, log } = useCleanStream();

  // Bloquear cierre mientras está corriendo + capturar Esc.
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isRunning) onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, isRunning, onClose]);

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.18 }}
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        >
          <motion.div
            initial={{ scale: 0.96, y: 8 }}
            animate={{ scale: 1, y: 0 }}
            exit={{ scale: 0.96, y: 8 }}
            transition={{ type: "spring", stiffness: 220, damping: 24 }}
            className="w-full max-w-2xl mx-4 p-6 rounded-2xl bg-bg-elevated border border-edge-default/40 shadow-2xl space-y-5"
          >
            {status.kind === "running" && (
              <CleanConsoleHeader
                progress={status.progress}
                phaseLabel={PHASE_LABEL[status.phase] ?? "Trabajando"}
              />
            )}

            {status.kind === "idle" && (
              <div className="space-y-2">
                <h2 className="text-xl font-semibold">Preparando…</h2>
                <p className="text-sm text-ink-secondary">
                  ClearTool está iniciando la limpieza. Aguarda un momento.
                </p>
              </div>
            )}

            {(status.kind === "complete" || status.kind === "failed") && (
              /* Hero del resumen — DETALLE EN DOC 12 */
              <div className="space-y-2">
                <h2 className="text-xl font-semibold">
                  {status.kind === "complete" ? "¡Listo!" : "Terminó con errores"}
                </h2>
                <p className="text-sm text-ink-secondary">
                  Ver el log completo abajo. El resumen detallado llega
                  en el doc 12.
                </p>
              </div>
            )}

            <CleanConsoleLog log={log} />

            <div className="flex justify-end">
              <Button
                onClick={onClose}
                disabled={isRunning}
                variant={status.kind === "complete" ? "default" : "outline"}
              >
                {isRunning ? "Limpiando…" : "Cerrar"}
              </Button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
```

## 8. Integración en `cache-page.tsx`

Cambios en `CachePage`:

```tsx
const [consoleOpen, setConsoleOpen] = useState(false);

const executeMutation = useMutation({
  mutationFn: async (opts: { /* igual que antes */ }) => {
    if (!plan) throw new Error("No hay plan");
    setConsoleOpen(true);                // ABRIR consola
    const r = await executeCleanPlan(plan, { /* igual */ });
    const v = await verifyClean(plan, r);
    return { report: r, verify: v, dryRun: opts.dryRun };
  },
  onSuccess: ({ report: r, verify: v, dryRun }) => {
    setReport(r);
    setVerify(v);
    if (!dryRun) {
      void qc.invalidateQueries({ queryKey: ["cache-plan-warm"] });
    }
    // TOAST: SE MANTIENE — el usuario explícitamente lo pidió.
    toast.success(
      dryRun ? "Simulación completada" : `Liberados ${formatBytes(r.totalBytesFreed)}`,
      {
        description:
          v.totalStillPresent > 0
            ? `${formatBytes(v.totalStillPresent)} no se pudieron eliminar.`
            : undefined,
      }
    );
  },
  onError: (err) => {
    toast.error("Error ejecutando limpieza", { description: formatError(err) });
    // dejar la consola abierta para que el usuario vea el log con el error.
  },
});

return (
  <div className="p-6 flex flex-col gap-4 h-full">
    {/* ... resto idéntico ... */}

    <CleanConsole
      open={consoleOpen}
      isRunning={executeMutation.isPending}
      onClose={() => setConsoleOpen(false)}
    />
  </div>
);
```

## 9. Criterio de done

- [ ] Al pulsar **Limpiar**, aparece `CleanConsole` en overlay con
      fade+spring (~200 ms).
- [ ] El header muestra fase, ubicación actual (X de Y), ETA y
      throughput vivos. Actualiza ≥4 veces por segundo (límite del
      throttle del backend, doc 09).
- [ ] La barra de progreso se mueve suavemente (transición CSS 200 ms).
- [ ] El log aparece con fade-in por línea (~150 ms).
- [ ] Auto-scroll-to-bottom funciona; al hacer scroll manual hacia
      arriba, aparece el botón "↓ Ir al final".
- [ ] El botón **Cerrar** está deshabilitado mientras corre y se
      habilita en `complete | failed`.
- [ ] Esc cierra la consola sólo si `!isRunning`.
- [ ] **El toast se sigue mostrando** al terminar (no se quita).
- [ ] No re-renderiza más de 8 veces por segundo en el peor caso
      (verificable con React Profiler — `useReducer` ayuda).
- [ ] El log se trunca a 500 líneas (sin OOM en limpiezas largas).

## 10. Riesgos / efectos secundarios

- **Bloqueo de navegación**: con `consoleOpen=true`, el usuario no
  puede usar el sidebar. Es intencional — limpiar es una acción
  bloqueante por seguridad. Pero documentarlo en el copy.
- **Eventos zombies**: si el usuario cancela navegando hacia otra
  pestaña y vuelve, el `useReducer` mantiene el log. Si quiere reset,
  basta con `reset()` cuando se cierra la consola en estado `complete`.
- **Animaciones costosas**: framer-motion + 500 líneas puede crujir en
  hardware modesto. Mitigación incluida en §6.
- **El doc 11 añade un 3D en el header** — `CleanConsoleHeader` deja
  un slot a la izquierda con un anillo CSS placeholder; el doc 11 lo
  sustituye sin tocar este componente.
