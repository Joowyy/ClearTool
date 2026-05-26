/*
 * status-chips — chips persistentes a la derecha de los tabs.
 *
 * Comunican estado crítico que el técnico necesita ver siempre, sin
 * tener que volver a /Inicio: privilegio y usuario activo. Si la app
 * está en modo limitado, ofrecemos CTA inline de re-elevación.
 */
import { Shield, ShieldOff, User, ArrowUpRight } from "lucide-react";
import { useAppStore } from "../../lib/store";
import { useSystemSummary } from "../../hooks/use-system-summary";
import { relaunchAsAdmin } from "../../api";

export function StatusChips() {
  const isElevated = useAppStore((s) => s.isElevated);
  const { data: summary } = useSystemSummary();

  const handleRelaunch = async () => {
    const ok = await relaunchAsAdmin();
    if (ok) window.close();
  };

  return (
    <div className="no-drag flex items-center gap-2">
      {!isElevated && (
        <button
          type="button"
          onClick={handleRelaunch}
          className="chip group hover:bg-signal-amber/12 transition-colors"
          data-tone="amber"
          title="Re-abre la app como administrador. Se cerrará esta ventana."
        >
          <ShieldOff className="h-3 w-3" strokeWidth={1.75} />
          <span>Modo limitado</span>
          <ArrowUpRight
            className="h-3 w-3 opacity-60 group-hover:opacity-100 transition-opacity"
            strokeWidth={1.75}
          />
        </button>
      )}

      {isElevated && (
        <span className="chip" data-tone="emerald" aria-label="Privilegios elevados">
          <Shield className="h-3 w-3" strokeWidth={1.75} />
          <span>Admin</span>
        </span>
      )}

      <span className="chip" aria-label={`Usuario activo: ${summary?.username ?? "desconocido"}`}>
        <User className="h-3 w-3" strokeWidth={1.75} />
        <span className="font-mono">{summary?.username ?? "—"}</span>
      </span>
    </div>
  );
}
