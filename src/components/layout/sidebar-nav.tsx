import { NavLink } from "react-router-dom";
import {
  FolderOpen,
  Trash2,
  Package,
  Settings2,
  Database,
  RotateCcw,
  Cpu,
  Settings,
} from "lucide-react";
import { ROUTES } from "../../lib/routes";
import { cn } from "../../lib/utils";

const navItems = [
  { to: ROUTES.HOME, label: "Inicio", icon: Cpu },
  { to: ROUTES.EXPLORER, label: "Explorador", icon: FolderOpen },
  { to: ROUTES.CACHE, label: "Caché", icon: Trash2 },
  { to: ROUTES.DEBLOAT, label: "Debloat", icon: Package },
  { to: ROUTES.SERVICES, label: "Servicios", icon: Settings2 },
  { to: ROUTES.REGISTRY, label: "Registro", icon: Database },
  { to: ROUTES.RESTORE, label: "Restauración", icon: RotateCcw },
  { to: ROUTES.SETTINGS, label: "Ajustes", icon: Settings },
];

export function SidebarNav() {
  return (
    <aside className="w-56 flex-shrink-0 border-r border-border bg-card flex flex-col">
      <div className="p-4 border-b border-border">
        <h1 className="text-lg font-bold text-foreground">ClearTool</h1>
        <p className="text-xs text-muted-foreground">Limpieza y optimización</p>
      </div>
      <nav className="flex-1 p-2 space-y-1">
        {navItems.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === ROUTES.HOME}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors",
                isActive
                  ? "bg-accent text-accent-foreground font-medium"
                  : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
              )
            }
          >
            <Icon className="h-4 w-4 flex-shrink-0" />
            {label}
          </NavLink>
        ))}
      </nav>
      <div className="p-3 border-t border-border text-xs text-muted-foreground">
        v0.1.0
      </div>
    </aside>
  );
}
