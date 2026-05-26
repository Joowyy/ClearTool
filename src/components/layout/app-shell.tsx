/*
 * app-shell — contenedor raíz del frontend.
 *
 * Layout vertical: [titlebar custom] · [outlet de la ruta activa] · [debug console].
 * El sidebar fue retirado en el refactor "Quirófano Cyan" — la navegación
 * vive en la titlebar como tabs horizontales. Esto libera la totalidad del
 * viewport para datos, lo cual es lo que importa en una herramienta.
 */
import { Outlet } from "react-router-dom";
import { Toaster } from "sonner";
import { Titlebar } from "./titlebar";
import { useElevation } from "../../hooks/use-elevation";
import { DebugConsole } from "../debug/debug-console";

export function AppShell() {
  useElevation();

  return (
    <div className="flex h-screen w-screen flex-col bg-surface-canvas text-ink-primary overflow-hidden">
      <Titlebar />
      <main className="relative flex-1 min-h-0 overflow-auto">
        <Outlet />
      </main>
      <DebugConsole />
      <Toaster
        theme="dark"
        position="bottom-right"
        richColors
        closeButton
        toastOptions={{
          classNames: {
            toast: "!bg-card !border-border",
          },
        }}
      />
    </div>
  );
}
