/*
 * titlebar — barra superior fina del shell.
 *
 * Tras el refactor M4, las tabs viven en el sidebar.
 * La titlebar solo muestra: logo + nombre + status chips + window controls.
 *
 * Drag region: contenedor con `data-tauri-drag-region`.
 */
import { Cpu } from "lucide-react";
import { StatusChips } from "./status-chips";
import { WindowControls } from "./window-controls";

export function Titlebar() {
  return (
    <header
      data-tauri-drag-region
      className="pillar relative flex items-center h-10 pl-3 pr-2 gap-3 select-none flex-shrink-0"
    >
      {/* Logo + nombre */}
      <div className="no-drag flex items-center gap-2 flex-shrink-0">
        <span className="inline-flex items-center justify-center h-6 w-6 rounded-md bg-signal-cyan/10 border border-signal-cyan/30">
          <Cpu className="h-3.5 w-3.5 text-signal-cyan" strokeWidth={2} />
        </span>
        <span className="text-xs font-semibold tracking-tight text-ink-primary">
          ClearTool
        </span>
        <span className="text-2xs text-ink-tertiary font-mono">v0.5</span>
      </div>

      <div className="flex-1" data-tauri-drag-region />

      <StatusChips />

      <WindowControls />
    </header>
  );
}
