import { CleanVisualizer } from "./clean-visualizer";
import { formatBytes, formatDuration } from "../../lib/format";
import type { CleanProgressV2Payload } from "../../api/events";

const PHASE_LABELS: Record<string, string> = {
  preparing: "Preparando",
  creatingRestorePoint: "Creando punto de restauración del sistema",
  closingProcesses: "Cerrando apps bloqueantes",
  cleaning: "Limpiando archivos",
  schedulingReboot: "Programando archivos para el próximo reinicio",
  verifying: "Verificando que se ha liberado el espacio",
  complete: "¡Listo!",
  failed: "Algo salió mal",
  cancelling: "Cancelando…",
};

function etaLabel(etaSecs: number | null, etaIsPrecise: boolean): string {
  if (etaSecs == null) return "calculando…";
  if (etaIsPrecise) return `~${formatDuration(etaSecs)}`;
  // Rango amplio: ×0.6 — ×1.6
  const lo = Math.max(0, etaSecs * 0.6);
  const hi = etaSecs * 1.6;
  return `${formatDuration(lo)} – ${formatDuration(hi)}`;
}

interface Props {
  progress: CleanProgressV2Payload;
}

export function CleanConsoleHeader({ progress }: Props) {
  const pct =
    progress.totalEstimatedBytes > 0
      ? Math.min(100, (progress.bytesFreed / progress.totalEstimatedBytes) * 100)
      : 0;

  const phaseLabel = PHASE_LABELS[progress.phase] ?? "Trabajando";

  return (
    <div className="flex gap-5">
      <div className="relative shrink-0" style={{ width: 120, height: 120 }}>
        <CleanVisualizer
          percent={pct}
          phase={progress.phase}
          throughputBytesPerSec={progress.throughputBytesPerSec}
          size={120}
        />
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <span className="text-lg font-semibold tabular-nums text-ink-primary">
            {pct.toFixed(0)}%
          </span>
        </div>
      </div>

      <div className="flex-1 space-y-3 min-w-0">
        <div>
          <h2 className="text-xl font-semibold text-ink-primary">Limpieza en curso</h2>
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

        <div className="flex items-baseline gap-3 text-sm flex-wrap">
          <span className="text-ink-secondary">
            Tiempo restante:{" "}
            <span className="text-ink-primary font-medium">
              {etaLabel(progress.etaSecs, progress.etaIsPrecise)}
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
            <span>{pct.toFixed(1)}%</span>
          </div>
        </div>
      </div>
    </div>
  );
}
