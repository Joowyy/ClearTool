/*
 * privacy-page — Privacy Hardening preset bundle.
 *
 * M4: 3 niveles (Balanced, Strict, Paranoid) que reutilizan
 * Registry Tweaks + Services + Debloat. Dry-run obligatorio.
 */
import { useState } from "react";
import { Shield, AlertTriangle, Check } from "lucide-react";
import { Button } from "../../components/ui/button";
import { cn } from "../../lib/utils";

type PrivacyLevel = "balanced" | "strict" | "paranoid";

const LEVELS: {
  id: PrivacyLevel;
  label: string;
  description: string;
  warning?: string;
  changes: string[];
}[] = [
  {
    id: "balanced",
    label: "Equilibrado",
    description: "Quita publicidad y telemetría no esencial. Recomendado.",
    changes: [
      "Advertising ID desactivado",
      "Cortana desactivada",
      "Búsqueda web en Start desactivada",
      "Historial de actividad desactivado",
      "Telemetría reducida (solo seguridad)",
    ],
  },
  {
    id: "strict",
    label: "Estricto",
    description: "Privacidad fuerte. Algunas búsquedas web menos integradas.",
    changes: [
      "Todo lo de Equilibrado",
      "DiagTrack (telemetry service) → Disabled",
      "dmwappushsvc → Disabled",
      "WerSvc → Disabled",
      "RetailDemo → Disabled",
      "Ubicación desactivada",
      "Sugerencias de apps desactivadas",
    ],
  },
  {
    id: "paranoid",
    label: "Paranoid",
    description: "Privacidad máxima. Lee los disclaimers, esto rompe cosas.",
    warning: "Esto puede afectar la funcionalidad de Windows. No recomendado para uso diario.",
    changes: [
      "Todo lo de Estricto",
      "Defender SmartScreen reducido",
      "ConnectedUserExperiencesAndTelemetry eliminado",
      "Copilot eliminado",
      "Experiencias personalizadas desactivadas",
    ],
  },
];

export function PrivacyPage() {
  const [selected, setSelected] = useState<PrivacyLevel>("balanced");
  const [applying, setApplying] = useState(false);

  const handleApply = async () => {
    setApplying(true);
    // TODO: invoke apply_privacy_preset
    setTimeout(() => setApplying(false), 2000);
  };

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
            {LEVELS.map((level) => (
              <button
                key={level.id}
                onClick={() => setSelected(level.id)}
                className={cn(
                  "w-full text-left panel p-4 rounded-lg transition-all duration-120",
                  selected === level.id
                    ? "border-signal-cyan/30 bg-signal-cyan/5"
                    : "hover:bg-surface-2",
                )}
              >
                <div className="flex items-center gap-3">
                  <div
                    className={cn(
                      "h-4 w-4 rounded-full border-2 flex items-center justify-center flex-shrink-0",
                      selected === level.id
                        ? "border-signal-cyan bg-signal-cyan"
                        : "border-edge-default/30",
                    )}
                  >
                    {selected === level.id && (
                      <Check className="h-2.5 w-2.5 text-surface-canvas" />
                    )}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-ink-primary">
                        {level.label}
                      </span>
                      {level.warning && (
                        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-2xs font-medium text-signal-amber bg-signal-amber/10">
                          <AlertTriangle className="h-3 w-3" />
                          Precaución
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-ink-tertiary mt-0.5">
                      {level.description}
                    </p>
                  </div>
                </div>
              </button>
            ))}
          </div>

          {/* Changes preview */}
          <div className="panel p-4">
            <h3 className="text-xs font-semibold text-ink-secondary mb-3">
              Cambios que se aplicarán:
            </h3>
            <ul className="space-y-1.5">
              {LEVELS.find((l) => l.id === selected)?.changes.map((c, i) => (
                <li key={i} className="text-xs text-ink-tertiary flex items-start gap-2">
                  <span className="text-signal-cyan mt-px">•</span>
                  {c}
                </li>
              ))}
            </ul>
          </div>

          {/* Warning */}
          {LEVELS.find((l) => l.id === selected)?.warning && (
            <div className="panel p-4 border-signal-amber/20 bg-signal-amber/5">
              <div className="flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 text-signal-amber flex-shrink-0 mt-0.5" />
                <p className="text-xs text-ink-secondary">
                  {LEVELS.find((l) => l.id === selected)?.warning}
                </p>
              </div>
            </div>
          )}

          {/* Apply button */}
          <div className="flex justify-end gap-2">
            <Button variant="outline">Ver cambios detallados</Button>
            <Button onClick={handleApply} disabled={applying}>
              <Shield className="h-3.5 w-3.5 mr-1.5" />
              {applying ? "Aplicando..." : "Aplicar (con restore point)"}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
