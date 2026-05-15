import { Outlet } from "react-router-dom";
import { SidebarNav } from "./sidebar-nav";
import { useAppStore } from "../../lib/store";
import { useElevation } from "../../hooks/use-elevation";
import { AlertTriangle } from "lucide-react";

export function AppShell() {
  const isElevated = useAppStore((s) => s.isElevated);
  useElevation();

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      <SidebarNav />
      <div className="flex-1 flex flex-col overflow-hidden">
        {!isElevated && (
          <div className="flex items-center gap-2 bg-yellow-900/30 border-b border-yellow-700/50 px-4 py-2 text-yellow-400 text-sm flex-shrink-0">
            <AlertTriangle className="h-4 w-4 flex-shrink-0" />
            <span>
              Modo limitado: algunas funciones requieren permisos de administrador.
            </span>
          </div>
        )}
        <main className="flex-1 overflow-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
