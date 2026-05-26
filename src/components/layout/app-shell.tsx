/*
 * app-shell — contenedor raíz del frontend.
 *
 * Layout M4: [titlebar fina] · [sidebar + outlet] · [debug console] · [toaster].
 * La navegación se movió de la titlebar al sidebar colapsable.
 * Incluye: ConfirmProvider, CommandPalette, keyboard shortcuts.
 */
import { Outlet } from "react-router-dom";
import { Toaster } from "sonner";
import { Titlebar } from "./titlebar";
import { Sidebar } from "./sidebar";
import { CommandPalette } from "../command-palette";
import { ConfirmProvider } from "../ui/confirm-dialog";
import { useElevation } from "../../hooks/use-elevation";
import { useKeyboardShortcuts } from "../../hooks/use-keyboard-shortcuts";
import { DebugConsole } from "../debug/debug-console";

function KeyboardShortcutsHandler() {
  useKeyboardShortcuts();
  return null;
}

export function AppShell() {
  useElevation();

  return (
    <ConfirmProvider>
      <KeyboardShortcutsHandler />
      <div className="flex h-screen w-screen flex-col bg-surface-canvas text-ink-primary overflow-hidden">
        <Titlebar />
        <div className="flex flex-1 min-h-0 overflow-hidden">
          <Sidebar />
          <main className="relative flex-1 min-h-0 overflow-auto">
            <Outlet />
          </main>
        </div>
        <CommandPalette />
        <DebugConsole />
        <Toaster
          theme="system"
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
    </ConfirmProvider>
  );
}
