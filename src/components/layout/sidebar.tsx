/*
 * sidebar — navegación lateral colapsable del shell.
 *
 * Sustituye a las tabs horizontales de la titlebar.
 * Width: expanded 220px, collapsed 56px (solo iconos).
 * Estado persistido en Zustand store.
 */
import { NavLink, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import {
  Trash2,
  Package,
  Settings2,
  Database,
  RotateCcw,
  Sliders,
  FileText,
  Cpu,
  Power,
  ChevronLeft,
  ChevronRight,
  HardDrive,
  Shield,
  Home,
} from "lucide-react";
import { ROUTES } from "../../lib/routes";
import { cn } from "../../lib/utils";
import { useAppStore } from "../../lib/store";

type NavItem = {
  to: string;
  label: string;
  icon: typeof Home;
  badge?: string;
};

const NAV_ITEMS: NavItem[] = [
  { to: ROUTES.HOME, label: "Inicio", icon: Home },
  { to: ROUTES.CACHE, label: "Caché", icon: Trash2 },
  { to: ROUTES.DEBLOAT, label: "Debloat", icon: Package },
  { to: ROUTES.SERVICES, label: "Servicios", icon: Settings2 },
  { to: ROUTES.REGISTRY, label: "Registro", icon: Database },
  { to: ROUTES.RESTORE, label: "Restauración", icon: RotateCcw },
  { to: ROUTES.AUDIT, label: "Auditoría", icon: FileText },
  { to: ROUTES.PROCESSES, label: "Procesos", icon: Cpu },
  { to: ROUTES.STARTUP, label: "Arranque", icon: Power },
  { to: ROUTES.DISK, label: "Disco", icon: HardDrive },
  { to: ROUTES.PRIVACY, label: "Privacidad", icon: Shield },
  { to: ROUTES.SETTINGS, label: "Ajustes", icon: Sliders },
];

export function Sidebar() {
  const location = useLocation();
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);

  return (
    <aside
      className={cn(
        "flex flex-col border-r border-edge-default/5 bg-surface-inset transition-all duration-180 ease-decel flex-shrink-0",
        sidebarCollapsed ? "w-14" : "w-52",
      )}
    >
      {/* Toggle button */}
      <div className="flex items-center justify-end p-2 h-10 flex-shrink-0">
        <button
          onClick={toggleSidebar}
          className={cn(
            "inline-flex items-center justify-center h-6 w-6 rounded-md",
            "text-ink-tertiary hover:text-ink-primary hover:bg-white/[0.05]",
            "transition-colors duration-120",
            sidebarCollapsed && "w-full",
          )}
          aria-label={sidebarCollapsed ? "Expandir sidebar" : "Colapsar sidebar"}
        >
          {sidebarCollapsed ? (
            <ChevronRight className="h-3.5 w-3.5" />
          ) : (
            <ChevronLeft className="h-3.5 w-3.5" />
          )}
        </button>
      </div>

      {/* Nav items */}
      <nav className="flex-1 overflow-y-auto overflow-x-hidden py-1 px-1.5" aria-label="Navegación principal">
        <div className="flex flex-col gap-0.5">
          {NAV_ITEMS.map(({ to, label, icon: Icon }) => {
            const isActive =
              to === ROUTES.HOME
                ? location.pathname === ROUTES.HOME
                : location.pathname.startsWith(to);

            return (
              <NavLink
                key={to}
                to={to}
                end={to === ROUTES.HOME}
                className={cn(
                  "relative flex items-center gap-2.5 rounded-md transition-colors duration-120 ease-soft no-drag",
                  sidebarCollapsed ? "justify-center h-9 w-9 mx-auto" : "h-9 px-2.5",
                  isActive
                    ? "text-signal-cyan bg-signal-cyan/8"
                    : "text-ink-secondary hover:text-ink-primary hover:bg-white/[0.03]",
                )}
              >
                <Icon
                  className="h-4 w-4 flex-shrink-0"
                  strokeWidth={isActive ? 2 : 1.75}
                />
                <AnimatePresence>
                  {!sidebarCollapsed && (
                    <motion.span
                      initial={{ opacity: 0, width: 0 }}
                      animate={{ opacity: 1, width: "auto" }}
                      exit={{ opacity: 0, width: 0 }}
                      transition={{ duration: 0.12 }}
                      className="text-xs font-medium tracking-tight truncate"
                    >
                      {label}
                    </motion.span>
                  )}
                </AnimatePresence>
                {isActive && (
                  <motion.div
                    layoutId="sidebar-active"
                    className="absolute left-0 top-1 bottom-1 w-0.5 rounded-full bg-signal-cyan"
                    transition={{ type: "spring", stiffness: 380, damping: 32 }}
                  />
                )}
              </NavLink>
            );
          })}
        </div>
      </nav>
    </aside>
  );
}
