/*
 * titlebar — barra superior personalizada del shell.
 *
 * Estructura:
 *   [logo + ClearTool] · [separador] · [tabs] · [spacer drag] · [status chips] · [min/max/close HTML]
 *
 * Los botones de ventana son **siempre** HTML propios (no dependemos del
 * overlay nativo de decorum, que en algunas versiones no se inyecta y deja
 * la titlebar sin X). decorum sigue cargado para acceder a `show_snap_overlay`
 * programáticamente más adelante (Win+Z) si lo añadimos como atajo.
 *
 * Drag region: contenedor con `data-tauri-drag-region`; hijos interactivos
 * con `.no-drag` o `data-tauri-drag-region="false"`.
 */
import { Cpu } from "lucide-react";
import { TabNav } from "./tab-nav";
import { StatusChips } from "./status-chips";
import { WindowControls } from "./window-controls";

export function Titlebar() {
  return (
    <header
      data-tauri-drag-region
      className="pillar relative flex items-center h-10 pl-3 gap-4 select-none flex-shrink-0"
    >
      {/* Logo + nombre. */}
      <div className="no-drag flex items-center gap-2 flex-shrink-0">
        <span className="inline-flex items-center justify-center h-6 w-6 rounded-md bg-signal-cyan/10 border border-signal-cyan/30">
          <Cpu className="h-3.5 w-3.5 text-signal-cyan" strokeWidth={2} />
        </span>
        <span className="text-xs font-semibold tracking-tight text-ink-primary">
          ClearTool
        </span>
        <span className="text-2xs text-ink-tertiary font-mono">v0.1</span>
      </div>

      <span className="h-4 w-px bg-edge-default/30 flex-shrink-0" aria-hidden />

      <TabNav />

      <div className="flex-1" data-tauri-drag-region />

      <StatusChips />

      {/* Botones min/max/close — siempre nuestros, asegura que la X exista. */}
      <WindowControls />
    </header>
  );
}
