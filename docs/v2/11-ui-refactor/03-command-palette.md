# Paso 03 — Command Palette con cmdk

**Área**: 11-ui-refactor
**Tiempo estimado**: 5-6 horas
**Dependencias**: Paso 01

## Qué hacemos

Atajo Ctrl+K abre overlay con búsqueda fuzzy contra: páginas, acciones rápidas, entradas de catálogos (debloat, registry tweaks), settings.

## Archivos

- `package.json` (añadir `cmdk`)
- `src/components/command-palette.tsx` (nuevo)
- `src/hooks/use-command-palette.ts` (nuevo)
- `src/components/layout/app-shell.tsx` (montar)
- `src/components/layout/titlebar.tsx` (botón opcional)

## Cómo

### 1. Install

```bash
npm i cmdk
```

### 2. Hook abrir/cerrar

```ts
// src/hooks/use-command-palette.ts
import { create } from "zustand";

interface PaletteState {
  open: boolean;
  setOpen: (v: boolean) => void;
  toggle: () => void;
}

export const usePaletteStore = create<PaletteState>(set => ({
  open: false,
  setOpen: (v) => set({ open: v }),
  toggle: () => set(s => ({ open: !s.open })),
}));
```

### 3. Componente palette

```tsx
// src/components/command-palette.tsx
import { Command } from "cmdk";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Activity, FolderTree, Trash2, Package, Settings as SettingsIcon } from "lucide-react";
import { usePaletteStore } from "../hooks/use-command-palette";
import { listBloatwareCatalog, listRegistryTweaks } from "../api";

export function CommandPalette() {
  const { open, setOpen } = usePaletteStore();
  const navigate = useNavigate();

  // Atajo Ctrl+K
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setOpen(true);
      }
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [setOpen]);

  // Cargar catálogos (cached)
  const { data: bloat = [] } = useQuery({
    queryKey: ["bloatware-catalog"],
    queryFn: listBloatwareCatalog,
    enabled: open,
  });
  const { data: tweaks = [] } = useQuery({
    queryKey: ["registry-tweaks"],
    queryFn: listRegistryTweaks,
    enabled: open,
  });

  const goto = (path: string) => {
    navigate(path);
    setOpen(false);
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/40" onClick={() => setOpen(false)}>
      <Command
        loop
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-xl bg-card border border-border rounded-lg shadow-2xl overflow-hidden"
      >
        <Command.Input
          autoFocus
          placeholder="Busca acciones, apps, tweaks..."
          className="w-full px-4 py-3 bg-transparent text-sm border-b border-border outline-none"
        />
        <Command.List className="max-h-96 overflow-y-auto p-2">
          <Command.Empty className="px-3 py-6 text-center text-sm text-muted-foreground">
            Sin resultados.
          </Command.Empty>

          <Command.Group heading="Navegar">
            <Item icon={Activity} onSelect={() => goto("/")}>Inicio</Item>
            <Item icon={FolderTree} onSelect={() => goto("/explorer")}>Explorador</Item>
            <Item icon={Trash2} onSelect={() => goto("/cache")}>Caché</Item>
            <Item icon={Package} onSelect={() => goto("/debloat")}>Debloat</Item>
            <Item icon={SettingsIcon} onSelect={() => goto("/settings")}>Ajustes</Item>
            {/* ... resto */}
          </Command.Group>

          {bloat.length > 0 && (
            <Command.Group heading="Apps a desinstalar">
              {bloat.slice(0, 20).map(e => (
                <Item key={e.id} onSelect={() => goto(`/debloat?selected=${e.id}`)}>
                  {e.displayName}
                  <span className="ml-auto text-xs text-muted-foreground">{e.category}</span>
                </Item>
              ))}
            </Command.Group>
          )}

          {tweaks.length > 0 && (
            <Command.Group heading="Tweaks de registro">
              {tweaks.slice(0, 20).map(t => (
                <Item key={t.id} onSelect={() => goto(`/registry?selected=${t.id}`)}>
                  {t.displayName}
                </Item>
              ))}
            </Command.Group>
          )}

          <Command.Group heading="Acciones rápidas">
            <Item onSelect={() => alert("TODO")}>Escanear caché ahora</Item>
            <Item onSelect={() => alert("TODO")}>Crear restore point</Item>
            <Item onSelect={() => alert("TODO")}>Listar procesos</Item>
          </Command.Group>
        </Command.List>

        <div className="border-t border-border p-2 flex items-center justify-between text-xs text-muted-foreground">
          <span>↑↓ navegar · ⏎ seleccionar · esc cerrar</span>
          <kbd className="px-1.5 py-0.5 bg-muted rounded">Ctrl+K</kbd>
        </div>
      </Command>
    </div>
  );
}

function Item({ icon: Icon, children, onSelect }: {
  icon?: React.ElementType; children: React.ReactNode; onSelect: () => void;
}) {
  return (
    <Command.Item
      onSelect={onSelect}
      className="flex items-center gap-2 px-3 py-2 rounded text-sm cursor-pointer aria-selected:bg-accent/40"
    >
      {Icon && <Icon className="h-4 w-4 text-muted-foreground" />}
      {children}
    </Command.Item>
  );
}
```

### 4. Montar en AppShell

```tsx
import { CommandPalette } from "../command-palette";
// ...
<AppShell>
  {/* ... */}
  <CommandPalette />
</AppShell>
```

### 5. Botón opcional en titlebar

```tsx
// titlebar.tsx
import { usePaletteStore } from "../../hooks/use-command-palette";

const openPalette = usePaletteStore(s => s.setOpen);
// ...
<button onClick={() => openPalette(true)} className="no-drag text-xs text-muted-foreground">
  <Search className="h-3 w-3 inline" /> Buscar (Ctrl+K)
</button>
```

## Criterio de done

- [ ] Ctrl+K abre/cierra palette.
- [ ] Escape cierra.
- [ ] Click fuera cierra.
- [ ] Búsqueda fuzzy funciona.
- [ ] Grupos: Navegar, Apps, Tweaks, Acciones.
- [ ] Hasta 20 entries por grupo de catálogos.
- [ ] Foco en input auto al abrir.
- [ ] Botón de "Buscar (Ctrl+K)" opcional en titlebar abre lo mismo.
