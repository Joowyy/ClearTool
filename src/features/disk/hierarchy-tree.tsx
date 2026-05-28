// hierarchy-tree — árbol jerárquico ordenado por peso, virtualizado.
//
// Aplanamos el árbol cada vez que cambia el set de expanded paths. Cada
// fila muestra: chevron, icono, nombre, mini-bar de % y métricas en mono.
import { useMemo, useRef, useState, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  ChevronRight,
  Folder,
  FolderOpen,
  File as FileIcon,
  Link2,
} from "lucide-react";
import { cn, formatBytes } from "../../lib/utils";
import { categoryOf, CATEGORY_COLORS, relativeTime } from "./disk-helpers";
import type { TreemapNode } from "../../api/types";

interface HierarchyTreeProps {
  root: TreemapNode | null;
  selectedPath: string | null;
  onSelect: (path: string) => void;
  onZoom: (node: TreemapNode) => void;
}

interface FlatRow {
  node: TreemapNode;
  depth: number;
  expanded: boolean;
}

function flatten(
  root: TreemapNode,
  expanded: Set<string>,
  depth = 0,
  out: FlatRow[] = [],
): FlatRow[] {
  const isExpanded = expanded.has(root.path);
  out.push({ node: root, depth, expanded: isExpanded });
  if (isExpanded && root.children.length) {
    for (const c of root.children) flatten(c, expanded, depth + 1, out);
  }
  return out;
}

export function HierarchyTree({
  root,
  selectedPath,
  onSelect,
  onZoom,
}: HierarchyTreeProps) {
  const [expanded, setExpanded] = useState<Set<string>>(() => {
    return new Set(root ? [root.path] : []);
  });

  // Resemilla cuando el root cambia (otro disco / re-escaneo).
  const initialPath = root?.path;
  const lastPathRef = useRef<string | null>(null);
  if (initialPath && lastPathRef.current !== initialPath) {
    lastPathRef.current = initialPath;
    if (!expanded.has(initialPath)) {
      setExpanded(new Set([initialPath]));
    }
  }

  const rows = useMemo(() => {
    if (!root) return [];
    return flatten(root, expanded);
  }, [root, expanded]);

  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 36,
    overscan: 12,
  });

  const toggle = useCallback((path: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }, []);

  if (!root) {
    return (
      <div className="h-full flex items-center justify-center text-xs text-ink-muted px-4 text-center">
        Selecciona un disco y pulsa Analizar para ver el árbol.
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="px-3 py-2 border-b border-edge-default/10 flex items-center gap-2">
        <span className="text-[10px] uppercase tracking-wider text-ink-muted">
          Jerarquía por peso
        </span>
        <span className="text-[10px] text-ink-muted font-mono ml-auto">
          {rows.length} filas
        </span>
      </div>

      <div ref={parentRef} className="flex-1 overflow-auto">
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: "100%",
            position: "relative",
          }}
        >
          {virtualizer.getVirtualItems().map((vi) => {
            const row = rows[vi.index];
            return (
              <TreeRow
                key={`${row.node.path}-${row.depth}`}
                row={row}
                selected={selectedPath === row.node.path}
                onToggle={toggle}
                onSelect={onSelect}
                onZoom={onZoom}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  height: `${vi.size}px`,
                  transform: `translateY(${vi.start}px)`,
                }}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
}

interface TreeRowProps {
  row: FlatRow;
  selected: boolean;
  onToggle: (path: string) => void;
  onSelect: (path: string) => void;
  onZoom: (node: TreemapNode) => void;
  style?: React.CSSProperties;
}

function TreeRow({ row, selected, onToggle, onSelect, onZoom, style }: TreeRowProps) {
  const { node, depth, expanded } = row;
  const hasChildren = node.children.length > 0 || node.truncated;
  const isDir = node.kind === "Dir";
  const isLink = node.kind === "Symlink" || node.kind === "Junction";

  let color = CATEGORY_COLORS.dir;
  if (node.kind === "File") {
    color = CATEGORY_COLORS[categoryOf(node.extension)];
  }

  const Icon = isLink ? Link2 : isDir ? (expanded ? FolderOpen : Folder) : FileIcon;

  return (
    <div
      style={style}
      onClick={() => onSelect(node.path)}
      onDoubleClick={() => isDir && onZoom(node)}
      className={cn(
        "group flex items-center gap-1.5 px-1.5 py-1 cursor-pointer transition-colors",
        "hover:bg-white/[0.03] border-l-2 border-transparent",
        selected && "bg-signal-cyan/[0.08] border-l-signal-cyan",
      )}
    >
      <div style={{ width: depth * 14 }} className="flex-shrink-0" />

      <button
        onClick={(e) => {
          e.stopPropagation();
          if (hasChildren) onToggle(node.path);
        }}
        className={cn(
          "flex items-center justify-center w-4 h-4 rounded text-ink-muted hover:text-ink-primary flex-shrink-0",
          !hasChildren && "invisible",
        )}
        aria-label={expanded ? "Colapsar" : "Expandir"}
      >
        <ChevronRight
          className={cn(
            "h-3 w-3 transition-transform duration-100",
            expanded && "rotate-90",
          )}
          strokeWidth={2}
        />
      </button>

      <Icon
        className="h-3.5 w-3.5 flex-shrink-0"
        strokeWidth={1.75}
        style={{ color }}
      />

      <span
        className={cn(
          "text-xs truncate flex-1 min-w-0",
          selected ? "text-ink-primary font-medium" : "text-ink-secondary",
        )}
        title={node.path}
      >
        {node.name || node.path}
        {node.truncated && (
          <span className="ml-1 text-ink-muted">…</span>
        )}
      </span>

      <div className="hidden md:flex items-center gap-1 flex-shrink-0">
        <div className="w-16 h-1 rounded-full bg-edge-default/15 overflow-hidden">
          <div
            className="h-full rounded-full"
            style={{
              width: `${Math.min(100, node.percentOfParent)}%`,
              backgroundColor: color,
              opacity: 0.85,
            }}
          />
        </div>
        <span className="w-9 text-right text-[10px] font-mono tabular-nums text-ink-tertiary">
          {node.percentOfParent.toFixed(1)}%
        </span>
      </div>

      <span className="w-20 text-right text-xs font-mono tabular-nums text-ink-primary flex-shrink-0">
        {formatBytes(node.sizeBytes)}
      </span>

      <span className="hidden xl:inline w-20 text-right text-[10px] font-mono tabular-nums text-ink-muted flex-shrink-0">
        {node.fileCount > 0 ? `${node.fileCount.toLocaleString("es-ES")} f` : "—"}
      </span>

      <span className="hidden xl:inline w-20 text-right text-[10px] text-ink-muted flex-shrink-0 truncate">
        {relativeTime(node.lastModified)}
      </span>
    </div>
  );
}
