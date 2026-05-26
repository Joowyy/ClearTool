# Paso 01 — Sidebar colapsable

**Área**: 11-ui-refactor
**Tiempo estimado**: 5-6 horas
**Dependencias**: ninguna

## Qué hacemos

Reemplazar las tabs en titlebar por un sidebar a la izquierda con iconos + labels, colapsable a 56px.

## Archivos

- `src/components/layout/sidebar.tsx` (nuevo)
- `src/components/layout/app-shell.tsx` (modificar layout)
- `src/components/layout/titlebar.tsx` (quitar TabNav, simplificar)
- `src/components/layout/tab-nav.tsx` (eliminar, ya no se usa)

## Cómo

### 1. Estructura nueva del AppShell

```
┌──────────────────────────────────────────────────────────────┐
│ Titlebar (delgada, solo logo + status chips + window ctrls)  │
├────┬─────────────────────────────────────────────────────────┤
│ S  │                                                         │
│ I  │                 Main content (Outlet)                   │
│ D  │                                                         │
│ E  │                                                         │
│ B  │                                                         │
│ A  │                                                         │
│ R  │                                                         │
└────┴─────────────────────────────────────────────────────────┘
```

### 2. AppShell refactor

```tsx
// src/components/layout/app-shell.tsx
export function AppShell() {
  useElevation();

  return (
    <div className="flex h-screen w-screen flex-col bg-surface-canvas text-ink-primary overflow-hidden">
      <Titlebar />
      <div className="flex flex-1 min-h-0">
        <Sidebar />
        <main className="relative flex-1 min-h-0 overflow-auto">
          <Outlet />
        </main>
      </div>
      <DebugConsole />
      <Toaster theme="dark" position="bottom-right" richColors closeButton />
    </div>
  );
}
```

### 3. Componente Sidebar

```tsx
// src/components/layout/sidebar.tsx
import { Link, useLocation } from "react-router-dom";
import {
  Activity, FolderTree, Trash2, Package, Cog,
  FileText, RotateCcw, FileSearch, Cpu, Rocket,
  HardDrive, Shield, Settings, ChevronLeft, ChevronRight,
} from "lucide-react";
import { useAppStore } from "../../lib/store";
import { useTranslation } from "react-i18next";  // opcional, vendrá en paso 08

interface NavItem {
  to: string;
  icon: React.ElementType;
  labelKey: string;            // i18n key
  badge?: number | string;
}

const NAV_ITEMS: NavItem[] = [
  { to: "/", icon: Activity, labelKey: "nav.home" },
  { to: "/explorer", icon: FolderTree, labelKey: "nav.explorer" },
  { to: "/cache", icon: Trash2, labelKey: "nav.cache" },
  { to: "/debloat", icon: Package, labelKey: "nav.debloat" },
  { to: "/services", icon: Cog, labelKey: "nav.services" },
  { to: "/registry", icon: FileText, labelKey: "nav.registry" },
  { to: "/restore", icon: RotateCcw, labelKey: "nav.restore" },
  { to: "/audit", icon: FileSearch, labelKey: "nav.audit" },
  { to: "/processes", icon: Cpu, labelKey: "nav.processes" },
  { to: "/startup", icon: Rocket, labelKey: "nav.startup" },
  { to: "/disk", icon: HardDrive, labelKey: "nav.disk" },
  { to: "/privacy", icon: Shield, labelKey: "nav.privacy" },
  { to: "/settings", icon: Settings, labelKey: "nav.settings" },
];

export function Sidebar() {
  const collapsed = useAppStore(s => s.sidebarCollapsed);
  const setCollapsed = useAppStore(s => s.setSidebarCollapsed);
  const location = useLocation();
  // const { t } = useTranslation();  // activar tras paso 08

  return (
    <aside
      className={`
        flex flex-col border-r border-border bg-surface-1
        transition-[width] duration-200 ease-out
        ${collapsed ? "w-14" : "w-56"}
      `}
    >
      <nav className="flex-1 overflow-y-auto py-2 space-y-0.5">
        {NAV_ITEMS.map(item => (
          <SidebarLink
            key={item.to}
            item={item}
            active={location.pathname === item.to}
            collapsed={collapsed}
          />
        ))}
      </nav>

      <button
        onClick={() => setCollapsed(!collapsed)}
        className="m-2 p-2 rounded-md hover:bg-accent/30 flex items-center justify-center text-muted-foreground hover:text-foreground"
        title={collapsed ? "Expandir sidebar" : "Colapsar sidebar"}
      >
        {collapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
      </button>
    </aside>
  );
}

function SidebarLink({ item, active, collapsed }: { item: NavItem; active: boolean; collapsed: boolean }) {
  const Icon = item.icon;
  // Por ahora hard-code labels en ES. En paso 08 → t(item.labelKey)
  const label = LABEL_FALLBACK[item.labelKey] ?? item.labelKey;

  return (
    <Link
      to={item.to}
      className={`
        mx-2 flex items-center gap-3 rounded-md px-3 py-2 text-sm transition
        ${active
          ? "bg-signal-cyan/15 text-signal-cyan"
          : "text-ink-secondary hover:bg-accent/20 hover:text-foreground"}
      `}
      title={collapsed ? label : undefined}
    >
      <Icon className="h-4 w-4 flex-shrink-0" />
      {!collapsed && (
        <>
          <span className="flex-1 truncate">{label}</span>
          {item.badge !== undefined && (
            <span className="text-xs bg-signal-cyan/20 text-signal-cyan rounded-full px-1.5">
              {item.badge}
            </span>
          )}
        </>
      )}
    </Link>
  );
}

const LABEL_FALLBACK: Record<string, string> = {
  "nav.home": "Inicio",
  "nav.explorer": "Explorador",
  "nav.cache": "Caché",
  "nav.debloat": "Debloat",
  "nav.services": "Servicios",
  "nav.registry": "Registro",
  "nav.restore": "Restauración",
  "nav.audit": "Auditoría",
  "nav.processes": "Procesos",
  "nav.startup": "Arranque",
  "nav.disk": "Disco",
  "nav.privacy": "Privacidad",
  "nav.settings": "Ajustes",
};
```

### 4. Titlebar simplificada

```tsx
// src/components/layout/titlebar.tsx
export function Titlebar() {
  return (
    <header
      data-tauri-drag-region
      className="pillar relative flex items-center h-9 px-3 gap-3 select-none flex-shrink-0"
    >
      <div className="no-drag flex items-center gap-2 flex-shrink-0">
        <span className="inline-flex items-center justify-center h-5 w-5 rounded-md bg-signal-cyan/10 border border-signal-cyan/30">
          <Cpu className="h-3 w-3 text-signal-cyan" strokeWidth={2} />
        </span>
        <span className="text-xs font-semibold tracking-tight">ClearTool</span>
        <span className="text-2xs text-ink-tertiary font-mono">v0.5</span>
      </div>

      <div className="flex-1" data-tauri-drag-region />

      <StatusChips />
      <WindowControls />
    </header>
  );
}
```

### 5. Eliminar `tab-nav.tsx`

`git rm src/components/layout/tab-nav.tsx` y borrar imports orfanos.

## Criterio de done

- [ ] Sidebar visible a la izquierda con 13 entries.
- [ ] Click navega correctamente, active state highlight.
- [ ] Botón inferior colapsa/expande con animación.
- [ ] Colapsado muestra solo iconos + tooltip al hover.
- [ ] Titlebar ya no contiene TabNav.
- [ ] No quedan referencias a `tab-nav.tsx` en el código.
