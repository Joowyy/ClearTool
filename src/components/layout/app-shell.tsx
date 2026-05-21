import { Outlet } from "react-router-dom";
import { AlertTriangle, Shield, ShieldOff, User, ArrowUpCircle } from "lucide-react";
import { SidebarNav } from "./sidebar-nav";
import { useAppStore } from "../../lib/store";
import { useElevation } from "../../hooks/use-elevation";
import { useSystemSummary } from "../../hooks/use-system-summary";
import { DebugConsole } from "../debug/debug-console";
import { relaunchAsAdmin } from "../../api";

export function AppShell() {
  const isElevated = useAppStore((s) => s.isElevated);
  useElevation();
  const { data: summary } = useSystemSummary();

  const handleRelaunch = async () => {
    const relaunch = await relaunchAsAdmin();
    if (relaunch) {
      // El proceso se relanzó; este proceso debería morir.
      window.close();
    }
  };

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      <SidebarNav />
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Topbar compacta: username + estado de elevación */}
        <header className="flex items-center justify-between gap-3 border-b border-border/60 bg-card/60 backdrop-blur px-4 py-2 flex-shrink-0">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            {!isElevated && (
              <span className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-yellow-900/30 border border-yellow-700/40 text-yellow-300">
                <AlertTriangle className="h-3.5 w-3.5" />
                Modo limitado: solo lectura
                <button
                  onClick={handleRelaunch}
                  className="flex items-center gap-1 ml-2 px-2 py-0.5 rounded bg-yellow-800/50 hover:bg-yellow-700/60 text-yellow-200 text-[11px] transition-colors"
                >
                  <ArrowUpCircle className="h-3 w-3" />
                  Reabrir como administrador
                </button>
              </span>
            )}
          </div>
          <div className="flex items-center gap-3 text-xs">
            <span className="flex items-center gap-1.5 text-muted-foreground">
              <User className="h-3.5 w-3.5" />
              <span className="font-medium text-foreground">
                {summary?.username ?? "—"}
              </span>
            </span>
            <span
              className={`flex items-center gap-1 px-2 py-1 rounded-md border text-[11px] ${
                isElevated
                  ? "bg-emerald-900/30 border-emerald-700/40 text-emerald-300"
                  : "bg-zinc-900/50 border-zinc-700/40 text-zinc-300"
              }`}
            >
              {isElevated ? (
                <>
                  <Shield className="h-3 w-3" /> Administrador
                </>
              ) : (
                <>
                  <ShieldOff className="h-3 w-3" /> Sin elevación
                </>
              )}
            </span>
          </div>
        </header>
        <main className="flex-1 overflow-auto">
          <Outlet />
        </main>
        <DebugConsole />
      </div>
    </div>
  );
}
