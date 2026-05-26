/*
 * tab-nav — pestañas horizontales del shell.
 *
 * Vive en la titlebar custom. Sigue el patrón Linear/Vercel: pill discreta,
 * subrayado cyan tenue en el activo, sin iconos grandes. Los iconos quedan
 * a 14px solo cuando ayudan a la lectura (Inicio, Caché, Debloat son
 * conceptos abstractos — el icono acelera el escaneo visual).
 */
import { NavLink, useLocation } from "react-router-dom";
import { motion } from "framer-motion";
import {
  Activity,
  FolderTree,
  Trash2,
  Package,
  Settings2,
  Database,
  RotateCcw,
  Sliders,
  FileText,
  Cpu,
  Power,
} from "lucide-react";
import { ROUTES } from "../../lib/routes";
import { cn } from "../../lib/utils";

type TabItem = {
  to: string;
  label: string;
  icon: typeof Activity;
};

const TABS: TabItem[] = [
  { to: ROUTES.HOME, label: "Inicio", icon: Activity },
  { to: ROUTES.EXPLORER, label: "Explorador", icon: FolderTree },
  { to: ROUTES.CACHE, label: "Caché", icon: Trash2 },
  { to: ROUTES.DEBLOAT, label: "Debloat", icon: Package },
  { to: ROUTES.SERVICES, label: "Servicios", icon: Settings2 },
  { to: ROUTES.REGISTRY, label: "Registro", icon: Database },
  { to: ROUTES.RESTORE, label: "Restauración", icon: RotateCcw },
  { to: ROUTES.AUDIT, label: "Auditoría", icon: FileText },
  { to: ROUTES.PROCESSES, label: "Procesos", icon: Cpu },
  { to: ROUTES.STARTUP, label: "Arranque", icon: Power },
  { to: ROUTES.SETTINGS, label: "Ajustes", icon: Sliders },
];

export function TabNav() {
  const location = useLocation();

  return (
    <nav
      className="no-drag flex items-center gap-0.5 h-full"
      aria-label="Secciones principales"
    >
      {TABS.map(({ to, label, icon: Icon }) => {
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
              "relative inline-flex items-center gap-1.5 px-3 h-7",
              "text-2xs font-medium tracking-tight",
              "rounded-md transition-colors duration-120 ease-soft",
              isActive
                ? "text-signal-cyan"
                : "text-ink-secondary hover:text-ink-primary hover:bg-white/[0.03]",
            )}
          >
            <Icon
              className="h-3.5 w-3.5 flex-shrink-0"
              strokeWidth={isActive ? 2 : 1.75}
            />
            <span>{label}</span>
            {isActive && (
              <motion.span
                layoutId="tab-underline"
                className="absolute inset-x-2 -bottom-[1px] h-[2px] rounded-full bg-signal-cyan"
                transition={{ type: "spring", stiffness: 380, damping: 32 }}
              />
            )}
          </NavLink>
        );
      })}
    </nav>
  );
}
