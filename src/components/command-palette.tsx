/*
 * command-palette — overlay global Ctrl+K.
 *
 * Búsqueda fuzzy contra páginas, acciones, entradas de catálogo y settings.
 * Usa cmdk (Vercel) — ~6KB, A11y impecable.
 */
import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Command } from "cmdk";
import {
  Search,
  Trash2,
  RotateCcw,
  Package,
  Settings2,
  Database,
  Cpu,
  Power,
  HardDrive,
  Shield,
  Sliders,
  Home,
  FileText,
  Zap,
} from "lucide-react";
import { ROUTES } from "../lib/routes";

type PaletteAction = {
  id: string;
  label: string;
  icon: typeof Search;
  group: string;
  run: () => void;
};

function usePaletteActions() {
  const navigate = useNavigate();

  return [
    {
      id: "nav-home",
      label: "Ir a Inicio",
      icon: Home,
      group: "Navegar",
      run: () => navigate(ROUTES.HOME),
    },
    {
      id: "nav-cache",
      label: "Ir a Caché",
      icon: Trash2,
      group: "Navegar",
      run: () => navigate(ROUTES.CACHE),
    },
    {
      id: "nav-debloat",
      label: "Ir a Debloat",
      icon: Package,
      group: "Navegar",
      run: () => navigate(ROUTES.DEBLOAT),
    },
    {
      id: "nav-services",
      label: "Ir a Servicios",
      icon: Settings2,
      group: "Navegar",
      run: () => navigate(ROUTES.SERVICES),
    },
    {
      id: "nav-registry",
      label: "Ir a Registro",
      icon: Database,
      group: "Navegar",
      run: () => navigate(ROUTES.REGISTRY),
    },
    {
      id: "nav-restore",
      label: "Ir a Restauración",
      icon: RotateCcw,
      group: "Navegar",
      run: () => navigate(ROUTES.RESTORE),
    },
    {
      id: "nav-audit",
      label: "Ir a Auditoría",
      icon: FileText,
      group: "Navegar",
      run: () => navigate(ROUTES.AUDIT),
    },
    {
      id: "nav-processes",
      label: "Ir a Procesos",
      icon: Cpu,
      group: "Navegar",
      run: () => navigate(ROUTES.PROCESSES),
    },
    {
      id: "nav-startup",
      label: "Ir a Arranque",
      icon: Power,
      group: "Navegar",
      run: () => navigate(ROUTES.STARTUP),
    },
    {
      id: "nav-disk",
      label: "Ir a Disco",
      icon: HardDrive,
      group: "Navegar",
      run: () => navigate(ROUTES.DISK),
    },
    {
      id: "nav-privacy",
      label: "Ir a Privacidad",
      icon: Shield,
      group: "Navegar",
      run: () => navigate(ROUTES.PRIVACY),
    },
    {
      id: "nav-settings",
      label: "Ir a Ajustes",
      icon: Sliders,
      group: "Navegar",
      run: () => navigate(ROUTES.SETTINGS),
    },
    {
      id: "action-scan-cache",
      label: "Escanear caché",
      icon: Zap,
      group: "Acciones rápidas",
      run: () => {
        navigate(ROUTES.CACHE);
      },
    },
    {
      id: "action-restore-point",
      label: "Crear restore point",
      icon: RotateCcw,
      group: "Acciones rápidas",
      run: () => {
        navigate(ROUTES.RESTORE);
      },
    },
  ] as PaletteAction[];
}

let globalToggle: (() => void) | null = null;

export function toggleCommandPalette() {
  globalToggle?.();
}

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const actions = usePaletteActions();

  useEffect(() => {
    globalToggle = () => setOpen((o) => !o);
    return () => {
      globalToggle = null;
    };
  }, []);

  return (
    <Command.Dialog
      open={open}
      onOpenChange={setOpen}
      label="Command Palette"
      className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
    >
      <div
        className="fixed inset-0 bg-black/60"
        onClick={() => setOpen(false)}
        aria-hidden
      />
      <div className="relative w-full max-w-lg panel-raised overflow-hidden animate-slide-up">
        <div className="flex items-center gap-2 px-3 border-b border-edge-default/10">
          <Search className="h-4 w-4 text-ink-tertiary flex-shrink-0" />
          <Command.Input
            placeholder="Busca acciones, módulos, apps..."
            className="w-full py-3 bg-transparent text-sm text-ink-primary placeholder:text-ink-muted outline-none"
          />
        </div>
        <Command.List className="max-h-80 overflow-y-auto p-1.5">
          <Command.Empty className="py-6 text-center text-xs text-ink-muted">
            Sin resultados.
          </Command.Empty>

          {["Navegar", "Acciones rápidas"].map((group) => (
            <Command.Group
              key={group}
              heading={group}
              className="px-1 py-1.5 text-2xs font-semibold text-ink-tertiary uppercase tracking-wider"
            >
              {actions
                .filter((a) => a.group === group)
                .map((action) => (
                  <Command.Item
                    key={action.id}
                    value={action.label}
                    onSelect={() => {
                      action.run();
                      setOpen(false);
                    }}
                    className="flex items-center gap-2.5 px-2.5 py-2 rounded-md text-sm text-ink-secondary cursor-pointer select-none data-[selected=true]:bg-signal-cyan/10 data-[selected=true]:text-signal-cyan transition-colors"
                  >
                    <action.icon className="h-4 w-4 flex-shrink-0" />
                    <span>{action.label}</span>
                  </Command.Item>
                ))}
            </Command.Group>
          ))}
        </Command.List>
        <div className="px-3 py-2 border-t border-edge-default/10 flex items-center gap-3 text-2xs text-ink-muted">
          <kbd className="px-1.5 py-0.5 rounded bg-surface-3 text-ink-tertiary font-mono">Ctrl+K</kbd>
          <span>abrir</span>
          <kbd className="px-1.5 py-0.5 rounded bg-surface-3 text-ink-tertiary font-mono">↑↓</kbd>
          <span>navegar</span>
          <kbd className="px-1.5 py-0.5 rounded bg-surface-3 text-ink-tertiary font-mono">Enter</kbd>
          <span>ejecutar</span>
          <kbd className="px-1.5 py-0.5 rounded bg-surface-3 text-ink-tertiary font-mono">Esc</kbd>
          <span>cerrar</span>
        </div>
      </div>
    </Command.Dialog>
  );
}
