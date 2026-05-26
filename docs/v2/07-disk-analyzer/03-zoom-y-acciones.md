# Paso 03 — Zoom + acciones contextuales

**Área**: 07-disk-analyzer
**Tiempo estimado**: 4-5 horas
**Dependencias**: Paso 02

## Qué hacemos

1. Click en un rectángulo → zoom in (esa carpeta llena el viewport).
2. Breadcrumb para volver atrás.
3. Doble click → menú contextual con acciones (Abrir en Explorer, Eliminar carpeta).
4. Acción "Eliminar carpeta" con restore point si >1GB.

## Archivos

- `src/features/disk-analyzer/disk-analyzer-page.tsx` (modificar — añadir state de zoom)
- `src/features/disk-analyzer/components/breadcrumb.tsx` (nuevo)
- `src/features/disk-analyzer/components/context-menu.tsx` (nuevo)

## Cómo

### 1. State de zoom

```tsx
// disk-analyzer-page.tsx
const [zoomStack, setZoomStack] = useState<TreemapNode[]>([]);

const visibleData = zoomStack.length > 0 ? zoomStack[zoomStack.length - 1] : data;

const handleClick = (node: HierarchyRectangularNode<TreemapNode>) => {
  if (node.data.isDir && node.data.children.length > 0) {
    setZoomStack(prev => [...prev, node.data]);
  }
};

const goBack = () => setZoomStack(prev => prev.slice(0, -1));
const goRoot = () => setZoomStack([]);
```

### 2. Breadcrumb

```tsx
// src/features/disk-analyzer/components/breadcrumb.tsx
import { ChevronRight, Home } from "lucide-react";

interface BreadcrumbProps {
  stack: TreemapNode[];
  onGoRoot: () => void;
  onGoTo: (depth: number) => void;
}

export function Breadcrumb({ stack, onGoRoot, onGoTo }: BreadcrumbProps) {
  return (
    <div className="flex items-center gap-1 text-sm text-muted-foreground">
      <button onClick={onGoRoot} className="hover:text-foreground">
        <Home className="h-3 w-3" />
      </button>
      {stack.map((node, i) => (
        <span key={i} className="flex items-center gap-1">
          <ChevronRight className="h-3 w-3" />
          <button
            onClick={() => onGoTo(i + 1)}
            className="hover:text-foreground"
          >
            {node.name}
          </button>
        </span>
      ))}
    </div>
  );
}
```

Integrar en page:
```tsx
<Breadcrumb
  stack={zoomStack}
  onGoRoot={goRoot}
  onGoTo={(d) => setZoomStack(prev => prev.slice(0, d))}
/>
```

### 3. Menú contextual

```tsx
// src/features/disk-analyzer/components/context-menu.tsx
import { useEffect, useRef } from "react";
import { FolderOpen, Trash2, Info } from "lucide-react";
import { formatBytes } from "../../../lib/utils";

interface ContextMenuProps {
  x: number;
  y: number;
  node: TreemapNode;
  onClose: () => void;
  onOpenInExplorer: () => void;
  onDelete: () => void;
}

export function ContextMenu({ x, y, node, onClose, onOpenInExplorer, onDelete }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    window.addEventListener("mousedown", handler);
    return () => window.removeEventListener("mousedown", handler);
  }, [onClose]);

  return (
    <div
      ref={ref}
      style={{ position: "fixed", left: x, top: y, zIndex: 50 }}
      className="bg-card border border-border rounded-lg shadow-lg py-1 min-w-[220px]"
    >
      <div className="px-3 py-2 border-b border-border">
        <div className="text-xs font-mono truncate">{node.path}</div>
        <div className="text-xs text-muted-foreground">{formatBytes(node.sizeBytes)}</div>
      </div>
      <MenuItem icon={FolderOpen} onClick={onOpenInExplorer}>Abrir en Explorer</MenuItem>
      <MenuItem icon={Info} onClick={() => navigator.clipboard.writeText(node.path)}>
        Copiar path
      </MenuItem>
      <div className="h-px bg-border my-1" />
      <MenuItem icon={Trash2} onClick={onDelete} danger>Eliminar carpeta</MenuItem>
    </div>
  );
}

function MenuItem({ icon: Icon, children, onClick, danger }: {
  icon: React.ElementType; children: React.ReactNode;
  onClick: () => void; danger?: boolean;
}) {
  return (
    <button
      className={`w-full px-3 py-1.5 text-left text-sm flex items-center gap-2 hover:bg-accent/30 ${
        danger ? "text-signal-red" : ""
      }`}
      onClick={onClick}
    >
      <Icon className="h-3 w-3" />
      {children}
    </button>
  );
}
```

### 4. Integración acciones

```tsx
// disk-analyzer-page.tsx
const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; node: TreemapNode } | null>(null);

const handleDoubleClick = (node: HierarchyRectangularNode<TreemapNode>) => {
  const evt = window.event as MouseEvent;
  setCtxMenu({ x: evt.clientX, y: evt.clientY, node: node.data });
};

const handleDelete = async (node: TreemapNode) => {
  if (node.sizeBytes > 1024 ** 3) {
    if (!confirm(`Eliminar carpeta de ${formatBytes(node.sizeBytes)}? Esto crea un restore point.`)) return;
  }
  try {
    // Usar Tauri fs plugin para eliminar
    const { remove } = await import("@tauri-apps/plugin-fs");
    await remove(node.path, { recursive: true });
    toast.success(`Eliminada ${node.name}`);
    refetch();
  } catch (err) {
    toast.error("No se pudo eliminar", err);
  }
};

const handleOpenInExplorer = (path: string) => {
  // Usar shell.open con el path
  import("@tauri-apps/plugin-opener").then(m => m.openPath(path));
};
```

### 5. Backend para delete grande con restore point

Mejor que usar el plugin fs sin restore point, añadir comando IPC dedicado:

```rust
#[tauri::command]
pub async fn delete_directory(path: String, create_restore_point: bool) -> AppResult<u64> {
    let p = std::path::Path::new(&path);
    if !p.exists() { return Err(AppError::Io("path no existe".into())); }

    // Calcular bytes a liberar (para report)
    let bytes = crate::platform::filesystem::directory_stats(p, false)
        .map(|(b, _, _)| b).unwrap_or(0);

    if create_restore_point && bytes > 1024 * 1024 * 1024 {
        crate::domain::restore::ensure_or_create(&format!("ClearTool — eliminar {}", path)).ok();
    }

    std::fs::remove_dir_all(p)
        .map_err(|e| AppError::Io(format!("remove {}: {}", path, e)))?;

    Ok(bytes)
}
```

## Criterio de done

- [ ] Click en carpeta hace zoom in.
- [ ] Breadcrumb permite navegar atrás.
- [ ] Doble click muestra context menu.
- [ ] "Abrir en Explorer" funciona.
- [ ] "Copiar path" funciona.
- [ ] "Eliminar carpeta" >1GB pide confirmación + crea restore point.
- [ ] Tras eliminar, refetch del treemap.
- [ ] Click fuera del menú lo cierra.
