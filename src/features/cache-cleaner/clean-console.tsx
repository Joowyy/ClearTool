import { useCallback, useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { CleanConsoleHeader } from "./clean-console-header";
import { CleanConsoleLog } from "./clean-console-log";
import { CleanSummaryHero } from "./clean-summary-hero";
import { Button } from "../../components/ui/button";
import { useCleanStream } from "./use-clean-stream";
import { cancelCleanPlan } from "../../api/client";
import type { CleanProgressV2Payload, CleanSummaryPayload } from "../../api/events";

interface Props {
  open: boolean;
  onClose: () => void;
  isRunning: boolean;
}

export function CleanConsole({ open, onClose, isRunning }: Props) {
  const { status, log, runId, cancelRequested } = useCleanStream();
  const [showFullLog, setShowFullLog] = useState(false);
  const [cancelling, setCancelling] = useState(false);

  const isDone =
    status.kind === "complete" ||
    status.kind === "failed" ||
    status.kind === "cancelled";
  const summary: CleanSummaryPayload | null =
    isDone ? (status as { summary: CleanSummaryPayload }).summary : null;

  // Reset state when a new run starts.
  useEffect(() => {
    if (status.kind === "idle") {
      setShowFullLog(false);
      setCancelling(false);
    }
  }, [status.kind]);

  const handleCancel = useCallback(async () => {
    if (!runId || cancelling) return;
    setCancelling(true);
    cancelRequested();
    try {
      await cancelCleanPlan(runId);
    } catch {
      // Best-effort
    }
  }, [runId, cancelling, cancelRequested]);

  // Esc closes only when not running.
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isRunning) onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, isRunning, onClose]);

  const activePhase = status.kind === "running" || status.kind === "cancelling";

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          role="dialog"
          aria-modal="true"
          aria-labelledby="clean-console-title"
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
            {/* Running / cancelling state */}
            {activePhase && (
              <CleanConsoleHeader progress={(status as { progress: CleanProgressV2Payload }).progress} />
            )}

            {/* Idle / preparing */}
            {status.kind === "idle" && (
              <div className="space-y-2">
                <h2 id="clean-console-title" className="text-xl font-semibold">
                  Preparando…
                </h2>
                <p className="text-sm text-ink-secondary">
                  ClearTool está iniciando la limpieza. Aguarda un momento.
                </p>
              </div>
            )}

            {/* Done: summary hero */}
            {isDone && summary && (
              <>
                <h2 id="clean-console-title" className="sr-only">
                  {summary.cancelled
                    ? "Limpieza cancelada"
                    : summary.success
                      ? "Limpieza completada"
                      : "Limpieza con avisos"}
                </h2>
                <CleanSummaryHero summary={summary} />
              </>
            )}

            {/* Log */}
            {(activePhase || status.kind === "idle" || showFullLog) && (
              <CleanConsoleLog log={log} />
            )}

            <div className="flex justify-end gap-2">
              {isDone && (
                <Button variant="ghost" onClick={() => setShowFullLog((s) => !s)}>
                  {showFullLog ? "Ocultar log" : "Ver log completo"}
                </Button>
              )}
              {/* Cancel button — visible only while running */}
              {(status.kind === "running" || status.kind === "cancelling") && (
                <Button
                  variant="outline"
                  onClick={handleCancel}
                  disabled={cancelling || status.kind === "cancelling"}
                  className="text-red-400 border-red-400/40 hover:bg-red-400/10"
                >
                  {cancelling || status.kind === "cancelling" ? "Cancelando…" : "Cancelar"}
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
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

