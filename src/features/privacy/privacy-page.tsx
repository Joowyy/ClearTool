/*
 * privacy-page — Privacy Hardening preset bundle.
 *
 * 3 niveles (Balanced, Strict, Paranoid) que reutilizan registry tweaks + servicios.
 * Preview real desde el backend. Dry-run + apply con restore point automático.
 */
import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Shield, AlertTriangle, Check, Loader2, ChevronDown, ChevronUp } from "lucide-react";
import { Button } from "../../components/ui/button";
import { cn } from "../../lib/utils";
import { formatError } from "../../lib/errors";
import { toast } from "../../lib/toast";

// Lazy-importar los comandos IPC para no contaminar el tipo del módulo client
async function getPrivacyPresetPreview(level: string) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<{
    level: string;
    registryTweakIds: string[];
    serviceChanges: { serviceName: string; targetStartType: string }[];
    estimatedChanges: number;
  }>("get_privacy_preset_preview", { level });
}

async function applyPrivacyPreset(level: string, dryRun: boolean) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<{
    level: string;
    dryRun: boolean;
    registryApplied: number;
    registryFailed: number;
    servicesApplied: number;
    servicesFailed: number;
    restorePointSeq: number | null;
    auditRunId: string;
    errors: string[];
  }>("apply_privacy_preset", { level, dryRun });
}

type PrivacyLevel = "balanced" | "strict" | "paranoid";

const LEVEL_META: Record<PrivacyLevel, { label: string; description: string; warning?: string }> = {
  balanced: {
    label: "Equilibrado",
    description: "Quita publicidad y telemetría no esencial. Recomendado.",
  },
  strict: {
    label: "Estricto",
    description: "Privacidad fuerte. Algunas búsquedas web menos integradas.",
  },
  paranoid: {
    label: "Paranoid",
    description: "Privacidad máxima. Lee los disclaimers, esto puede romper cosas.",
    warning: "Esto puede afectar la funcionalidad de Windows. No recomendado para uso diario.",
  },
};

export function PrivacyPage() {
  const [selected, setSelected] = useState<PrivacyLevel>("balanced");
  const [showDetails, setShowDetails] = useState(false);

  const previewQuery = useQuery({
    queryKey: ["privacy-preview", selected],
    queryFn: () => getPrivacyPresetPreview(selected),
    staleTime: 60_000,
  });

  const applyMutation = useMutation({
    mutationFn: ({ level, dryRun }: { level: PrivacyLevel; dryRun: boolean }) =>
      applyPrivacyPreset(level, dryRun),
    onSuccess: (report) => {
      if (report.dryRun) {
        toast.success("Simulación completada", {
          description: `${report.registryApplied + report.servicesApplied} cambios simulados sin modificar el sistema.`,
        });
      } else {
        const total = report.registryApplied + report.servicesApplied;
        const failed = report.registryFailed + report.servicesFailed;
        toast.success(`Preset aplicado — ${total} cambio${total !== 1 ? "s" : ""}`, {
          description: failed > 0
            ? `${failed} falló. Restore point: ${report.restorePointSeq ?? "no creado"}.`
            : report.restorePointSeq
              ? `Restore point #${report.restorePointSeq} creado.`
              : undefined,
        });
      }
    },
    onError: (err) => toast.error("Error al aplicar preset", { description: formatError(err) }),
  });

  const meta = LEVEL_META[selected];
  const preview = previewQuery.data;

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between p-4 border-b border-edge-default/10">
        <div>
          <h1 className="text-sm font-semibold text-ink-primary">Privacidad</h1>
          <p className="text-xs text-ink-tertiary mt-0.5">
            Endurece la privacidad de Windows con presets coordinados
          </p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto space-y-6">
          {/* Level selector */}
          <div className="space-y-2">
            {(Object.keys(LEVEL_META) as PrivacyLevel[]).map((level) => {
              const m = LEVEL_META[level];
              return (
                <button
                  key={level}
                  onClick={() => setSelected(level)}
                  className={cn(
                    "w-full text-left panel p-4 rounded-lg transition-all duration-120",
                    selected === level
                      ? "border-signal-cyan/30 bg-signal-cyan/5"
                      : "hover:bg-surface-2",
                  )}
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={cn(
                        "h-4 w-4 rounded-full border-2 flex items-center justify-center flex-shrink-0",
                        selected === level
                          ? "border-signal-cyan bg-signal-cyan"
                          : "border-edge-default/30",
                      )}
                    >
                      {selected === level && (
                        <Check className="h-2.5 w-2.5 text-surface-canvas" />
                      )}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-ink-primary">{m.label}</span>
                        {m.warning && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-2xs font-medium text-signal-amber bg-signal-amber/10">
                            <AlertTriangle className="h-3 w-3" />
                            Precaución
                          </span>
                        )}
                      </div>
                      <p className="text-xs text-ink-tertiary mt-0.5">{m.description}</p>
                    </div>
                  </div>
                </button>
              );
            })}
          </div>

          {/* Preview real del backend */}
          <div className="panel p-4">
            <button
              className="w-full flex items-center justify-between"
              onClick={() => setShowDetails((v) => !v)}
            >
              <h3 className="text-xs font-semibold text-ink-secondary">
                {previewQuery.isLoading
                  ? "Cargando cambios..."
                  : `${preview?.estimatedChanges ?? 0} cambios que se aplicarán`}
              </h3>
              {showDetails ? (
                <ChevronUp className="h-3.5 w-3.5 text-ink-tertiary" />
              ) : (
                <ChevronDown className="h-3.5 w-3.5 text-ink-tertiary" />
              )}
            </button>

            {showDetails && preview && (
              <div className="mt-3 space-y-3">
                {preview.registryTweakIds.length > 0 && (
                  <div>
                    <p className="text-[10px] uppercase tracking-wider text-ink-muted mb-1.5">
                      Registry tweaks
                    </p>
                    <ul className="space-y-1">
                      {preview.registryTweakIds.map((id) => (
                        <li key={id} className="text-xs text-ink-tertiary flex items-center gap-2">
                          <span className="text-signal-cyan">•</span>
                          <code className="font-mono text-[11px]">{id}</code>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                {preview.serviceChanges.length > 0 && (
                  <div>
                    <p className="text-[10px] uppercase tracking-wider text-ink-muted mb-1.5">
                      Servicios
                    </p>
                    <ul className="space-y-1">
                      {preview.serviceChanges.map((sc) => (
                        <li
                          key={sc.serviceName}
                          className="text-xs text-ink-tertiary flex items-center gap-2"
                        >
                          <span className="text-signal-amber">•</span>
                          <span className="font-mono text-[11px]">{sc.serviceName}</span>
                          <span className="text-ink-muted">→</span>
                          <span className="text-ink-secondary">{sc.targetStartType}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Warning */}
          {meta.warning && (
            <div className="panel p-4 border-signal-amber/20 bg-signal-amber/5">
              <div className="flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 text-signal-amber flex-shrink-0 mt-0.5" />
                <p className="text-xs text-ink-secondary">{meta.warning}</p>
              </div>
            </div>
          )}

          {/* Botones */}
          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={() => applyMutation.mutate({ level: selected, dryRun: true })}
              disabled={applyMutation.isPending}
            >
              {applyMutation.isPending && applyMutation.variables?.dryRun ? (
                <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
              ) : null}
              Simular (dry-run)
            </Button>
            <Button
              onClick={() => applyMutation.mutate({ level: selected, dryRun: false })}
              disabled={applyMutation.isPending}
            >
              {applyMutation.isPending && !applyMutation.variables?.dryRun ? (
                <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
              ) : (
                <Shield className="h-3.5 w-3.5 mr-1.5" />
              )}
              Aplicar (con restore point)
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
