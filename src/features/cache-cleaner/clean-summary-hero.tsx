import { motion } from "framer-motion";
import {
  CheckCircle2,
  AlertTriangle,
  CircleSlash,
  Clock,
  Files,
  RotateCw,
  Shield,
  FileWarning,
  Lock,
  Filter,
  Link,
  RefreshCw,
} from "lucide-react";
import type { CleanSummaryPayload, ResidualEntry, ResidualReason } from "../../api/events";
import { formatBytes, formatDuration } from "../../lib/format";
import { ignoreResidualPath } from "../../api/client";

interface Props {
  summary: CleanSummaryPayload;
  residuals?: ResidualEntry[];
}

export function CleanSummaryHero({ summary, residuals }: Props) {
  const cancelled = summary.cancelled ?? false;
  const success = !cancelled && summary.success && summary.totalFilesFailed === 0;
  const durationSecs = summary.durationMs / 1000;

  return (
    <div className="space-y-5">
      <motion.div
        initial={{ scale: 0.7, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 220, damping: 18 }}
        className="flex flex-col items-center text-center gap-2"
      >
        {cancelled ? (
          <CircleSlash className="h-14 w-14 text-zinc-400" />
        ) : success ? (
          <CheckCircle2 className="h-14 w-14 text-emerald-400" />
        ) : (
          <AlertTriangle className="h-14 w-14 text-amber-400" />
        )}
        <h2 className="text-2xl font-semibold text-ink-primary">
          {cancelled ? "Cancelado" : success ? "¡Hecho!" : "Terminó con avisos"}
        </h2>
        {!cancelled && (
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
        )}
        {cancelled && summary.totalBytesFreed > 0 && (
          <p className="text-ink-secondary">
            Se liberaron{" "}
            <span className="text-ink-primary font-semibold">
              {formatBytes(summary.totalBytesFreed)}
            </span>{" "}
            antes de cancelar.
          </p>
        )}
      </motion.div>

      {!cancelled && (
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
      )}

      {summary.locationsWithErrors.length > 0 && (
        <div className="rounded-lg bg-amber-900/15 border border-amber-700/30 p-3 text-xs text-amber-300">
          <p className="font-medium mb-1">
            {summary.locationsWithErrors.length} ubicaci
            {summary.locationsWithErrors.length === 1 ? "ón" : "ones"} con errores
          </p>
          <ul className="space-y-0.5 text-amber-200/80">
            {summary.locationsWithErrors.slice(0, 4).map((id) => (
              <li key={id} className="font-mono">
                {id}
              </li>
            ))}
            {summary.locationsWithErrors.length > 4 && (
              <li className="text-amber-300/60">
                … y {summary.locationsWithErrors.length - 4} más (ver log)
              </li>
            )}
          </ul>
        </div>
      )}

      {/* Reboot CTA */}
      {summary.totalBytesScheduledReboot > 0 && (
        <div className="rounded-lg bg-bg-canvas/40 border border-edge-default/30 p-3 text-sm text-ink-secondary flex items-center gap-3">
          <RefreshCw className="h-4 w-4 shrink-0 text-ink-tertiary" />
          <span>
            {formatBytes(summary.totalBytesScheduledReboot)} programados para borrarse al reiniciar.
          </span>
        </div>
      )}

      {/* Residuals section */}
      {residuals && residuals.length > 0 && (
        <details
          className="rounded-lg bg-bg-canvas/40 border border-edge-default/30"
          open
        >
          <summary className="cursor-pointer px-3 py-2 text-sm text-ink-secondary select-none">
            Quedó pendiente:{" "}
            {formatBytes(residuals.reduce((s, r) => s + r.bytes, 0))} (
            {residuals.length} ubicaci{residuals.length === 1 ? "ón" : "ones"})
          </summary>
          <div className="px-3 pb-3 pt-1 space-y-2">
            {residuals.map((r) => (
              <ResidualRow key={r.path} entry={r} />
            ))}
          </div>
        </details>
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
  const valueColor = tone === "warn" ? "text-amber-300" : "text-ink-primary";
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

function residualIcon(reason: ResidualReason) {
  switch (reason.kind) {
    case "lockedBySystem": return Lock;
    case "pendingReboot": return RotateCw;
    case "accessDenied": return Shield;
    case "filteredOut": return Filter;
    case "reparsePoint": return Link;
    default: return FileWarning;
  }
}

function residualLabel(reason: ResidualReason): string {
  switch (reason.kind) {
    case "lockedBySystem": return `Bloqueado por proceso del sistema`;
    case "pendingReboot": return "Se borrará al reiniciar";
    case "accessDenied": return "Acceso denegado (permisos)";
    case "filteredOut": return "Excluido por filtros de seguridad";
    case "reparsePoint": return "Enlace simbólico — no se siguió";
    default: return "Motivo desconocido";
  }
}

function residualTooltip(reason: ResidualReason): string | null {
  if (reason.kind === "lockedBySystem") return reason.suggestedAction;
  if (reason.kind === "filteredOut")
    return "Este archivo se modificó recientemente. ClearTool lo omite por seguridad.";
  return null;
}

function ResidualRow({ entry }: { entry: ResidualEntry }) {
  const Icon = residualIcon(entry.reason);
  const label = residualLabel(entry.reason);
  const tooltip = residualTooltip(entry.reason);
  const canIgnore =
    entry.reason.kind === "lockedBySystem" || entry.reason.kind === "accessDenied";

  const handleIgnore = async () => {
    try {
      await ignoreResidualPath(entry.path);
    } catch {
      // Best-effort
    }
  };

  return (
    <div className="flex items-start gap-2 text-xs" title={tooltip ?? undefined}>
      <Icon className="h-3.5 w-3.5 mt-0.5 text-ink-tertiary shrink-0" />
      <div className="flex-1 min-w-0">
        <span className="text-ink-secondary font-medium">
          {entry.path.split(/[/\\]/).pop() ?? entry.path}
        </span>
        <span className="text-ink-tertiary"> ({formatBytes(entry.bytes)}) — {label}</span>
      </div>
      {canIgnore && (
        <button
          className="text-ink-tertiary hover:text-ink-secondary underline text-[11px] shrink-0"
          onClick={handleIgnore}
        >
          Ignorar
        </button>
      )}
    </div>
  );
}
