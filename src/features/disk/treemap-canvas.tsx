// treemap-canvas — refactor del treemap canvas con look futurista:
// gradiente por celda, glow al hover, hit-test optimizado, sincronización
// del seleccionado con el tree.
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import * as d3h from "d3-hierarchy";
import { formatBytes } from "../../lib/utils";
import { categoryOf, CATEGORY_COLORS, colorForNode, darken, lighten, relativeTime } from "./disk-helpers";
import type { TreemapNode } from "../../api/types";

interface TreemapCanvasProps {
  root: TreemapNode;
  selectedPath: string | null;
  onHover: (path: string | null) => void;
  onSelect: (node: TreemapNode) => void;
  onNavigate: (node: TreemapNode) => void;
}

interface LayoutLeaf {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  data: TreemapNode;
  value: number;
  depth: number;
}

const FONT = "11px Inter, system-ui, sans-serif";
const MIN_FRACTION_VISIBLE = 0.0008;

function buildLayout(root: TreemapNode, width: number, height: number): LayoutLeaf[] {
  const hier = d3h
    .hierarchy<TreemapNode>(root, (d) => d.children?.length ? d.children : null)
    .sum((d) => (!d.children || d.children.length === 0 ? d.sizeBytes : 0))
    .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

  d3h
    .treemap<TreemapNode>()
    .size([width, height])
    .paddingInner(1)
    .round(true)(hier);

  const rootValue = hier.value ?? 1;
  const leaves: LayoutLeaf[] = [];
  hier.each((node) => {
    const n = node as d3h.HierarchyRectangularNode<TreemapNode>;
    if (n.x0 === undefined) return;
    if ((n.value ?? 0) / rootValue < MIN_FRACTION_VISIBLE) return;
    // Solo emitir las hojas para el render principal; el padding mostrará la jerarquía.
    if (n.children && n.children.length > 0) return;
    leaves.push({
      x0: n.x0,
      y0: n.y0,
      x1: n.x1,
      y1: n.y1,
      data: n.data,
      value: n.value ?? 0,
      depth: n.depth,
    });
  });
  return leaves;
}

function drawLabel(ctx: CanvasRenderingContext2D, leaf: LayoutLeaf) {
  const w = leaf.x1 - leaf.x0;
  const h = leaf.y1 - leaf.y0;
  if (w < 38 || h < 16) return;
  ctx.fillStyle = "rgba(255,255,255,0.92)";
  ctx.font = FONT;
  ctx.textBaseline = "top";
  const label = leaf.data.name || leaf.data.path;
  const maxW = w - 8;
  let text = label;
  if (ctx.measureText(text).width > maxW) {
    while (text.length > 1 && ctx.measureText(text + "…").width > maxW) {
      text = text.slice(0, -1);
    }
    text += "…";
  }
  ctx.fillText(text, leaf.x0 + 4, leaf.y0 + 4);
  if (h > 32) {
    ctx.fillStyle = "rgba(255,255,255,0.55)";
    ctx.font = "10px ui-monospace, SFMono-Regular, monospace";
    ctx.fillText(formatBytes(leaf.value), leaf.x0 + 4, leaf.y0 + 18);
  }
}

function drawTreemap(
  ctx: CanvasRenderingContext2D,
  leaves: LayoutLeaf[],
  dpr: number,
  hoveredPath: string | null,
  selectedPath: string | null,
) {
  ctx.save();
  ctx.scale(dpr, dpr);

  for (const leaf of leaves) {
    const w = leaf.x1 - leaf.x0;
    const h = leaf.y1 - leaf.y0;
    if (w < 1 || h < 1) continue;

    const baseColor = colorForNode(leaf.data);
    const grad = ctx.createLinearGradient(leaf.x0, leaf.y0, leaf.x0, leaf.y1);
    grad.addColorStop(0, lighten(baseColor, 0.12));
    grad.addColorStop(1, darken(baseColor, 0.22));
    ctx.fillStyle = grad;
    ctx.fillRect(leaf.x0, leaf.y0, w, h);

    ctx.strokeStyle = darken(baseColor, 0.35);
    ctx.lineWidth = 1;
    ctx.strokeRect(leaf.x0 + 0.5, leaf.y0 + 0.5, w - 1, h - 1);

    drawLabel(ctx, leaf);

    const isHovered = hoveredPath === leaf.data.path;
    const isSelected = selectedPath === leaf.data.path;
    if (isHovered || isSelected) {
      ctx.save();
      ctx.shadowColor = baseColor;
      ctx.shadowBlur = isSelected ? 22 : 14;
      ctx.strokeStyle = isSelected ? "rgba(255,255,255,0.95)" : "rgba(255,255,255,0.75)";
      ctx.lineWidth = isSelected ? 2 : 1.5;
      ctx.strokeRect(leaf.x0 + 1, leaf.y0 + 1, w - 2, h - 2);
      ctx.restore();
    }
  }
  ctx.restore();
}

interface TooltipState {
  x: number;
  y: number;
  node: TreemapNode;
  value: number;
}

function Tooltip({ tip }: { tip: TooltipState }) {
  const color = colorForNode(tip.node);
  const cat = categoryOf(tip.node.extension);
  return (
    <div
      className="pointer-events-none fixed z-50 panel-raised rounded-lg px-3 py-2 text-xs shadow-2xl border border-edge-default/15"
      style={{ left: tip.x + 14, top: tip.y - 12, maxWidth: 320 }}
    >
      <div className="flex items-center gap-1.5 mb-1">
        <span
          className="inline-block w-2 h-2 rounded-sm flex-shrink-0"
          style={{ backgroundColor: color }}
        />
        <span className="font-medium text-ink-primary truncate">
          {tip.node.name}
        </span>
      </div>
      <div className="font-mono text-[10px] text-ink-tertiary truncate" title={tip.node.path}>
        {tip.node.path}
      </div>
      <div className="grid grid-cols-2 gap-x-3 gap-y-0.5 mt-1.5">
        <span className="text-ink-muted">Tamaño</span>
        <span className="font-mono tabular-nums text-ink-primary">
          {formatBytes(tip.value)}
        </span>
        <span className="text-ink-muted">% del root</span>
        <span className="font-mono tabular-nums text-ink-secondary">
          {tip.node.percentOfRoot.toFixed(2)}%
        </span>
        {tip.node.kind === "File" && (
          <>
            <span className="text-ink-muted">Categoría</span>
            <span className="text-ink-secondary">{cat}</span>
          </>
        )}
        {tip.node.fileCount > 0 && (
          <>
            <span className="text-ink-muted">Archivos</span>
            <span className="font-mono tabular-nums text-ink-secondary">
              {tip.node.fileCount.toLocaleString("es-ES")}
            </span>
          </>
        )}
        {tip.node.lastModified && (
          <>
            <span className="text-ink-muted">Modif.</span>
            <span className="text-ink-secondary">
              {relativeTime(tip.node.lastModified)}
            </span>
          </>
        )}
      </div>
    </div>
  );
}

export function TreemapCanvas({
  root,
  selectedPath,
  onHover,
  onSelect,
  onNavigate,
}: TreemapCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [size, setSize] = useState({ w: 0, h: 0 });
  const [tip, setTip] = useState<TooltipState | null>(null);
  const [hoveredPath, setHoveredPath] = useState<string | null>(null);
  const leavesRef = useRef<LayoutLeaf[]>([]);

  useEffect(() => {
    if (!containerRef.current) return;
    let timer: ReturnType<typeof setTimeout>;
    const ro = new ResizeObserver((entries) => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const { width, height } = entries[0].contentRect;
        setSize({ w: Math.floor(width), h: Math.floor(height) });
      }, 150);
    });
    ro.observe(containerRef.current);
    return () => {
      clearTimeout(timer);
      ro.disconnect();
    };
  }, []);

  const leaves = useMemo(() => {
    if (size.w === 0 || size.h === 0) return [];
    return buildLayout(root, size.w, size.h);
  }, [root, size.w, size.h]);

  leavesRef.current = leaves;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    canvas.width = size.w * dpr;
    canvas.height = size.h * dpr;
    canvas.style.width = `${size.w}px`;
    canvas.style.height = `${size.h}px`;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawTreemap(ctx, leaves, dpr, hoveredPath, selectedPath);
  }, [leaves, size, hoveredPath, selectedPath]);

  const hitTest = useCallback((cx: number, cy: number): LayoutLeaf | null => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return null;
    const x = cx - rect.left;
    const y = cy - rect.top;
    for (let i = leavesRef.current.length - 1; i >= 0; i--) {
      const l = leavesRef.current[i];
      if (x >= l.x0 && x <= l.x1 && y >= l.y0 && y <= l.y1) return l;
    }
    return null;
  }, []);

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      const leaf = hitTest(e.clientX, e.clientY);
      if (leaf) {
        setTip({ x: e.clientX, y: e.clientY, node: leaf.data, value: leaf.value });
        if (hoveredPath !== leaf.data.path) {
          setHoveredPath(leaf.data.path);
          onHover(leaf.data.path);
        }
      } else {
        setTip(null);
        if (hoveredPath !== null) {
          setHoveredPath(null);
          onHover(null);
        }
      }
    },
    [hitTest, hoveredPath, onHover],
  );

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      const leaf = hitTest(e.clientX, e.clientY);
      if (!leaf) return;
      onSelect(leaf.data);
    },
    [hitTest, onSelect],
  );

  const handleDoubleClick = useCallback(
    (e: React.MouseEvent) => {
      const leaf = hitTest(e.clientX, e.clientY);
      if (leaf && leaf.data.kind === "Dir" && leaf.data.children?.length) {
        onNavigate(leaf.data);
      }
    },
    [hitTest, onNavigate],
  );

  const handleMouseLeave = useCallback(() => {
    setTip(null);
    setHoveredPath(null);
    onHover(null);
  }, [onHover]);

  return (
    <div ref={containerRef} className="w-full h-full relative overflow-hidden rounded-lg border border-edge-default/10">
      <div
        aria-hidden
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            "radial-gradient(ellipse at top left, rgba(34,211,238,0.05), transparent 70%)",
        }}
      />
      <canvas
        ref={canvasRef}
        className="block cursor-pointer relative"
        style={{ width: "100%", height: "100%" }}
        onMouseMove={handleMouseMove}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        onMouseLeave={handleMouseLeave}
      />
      {tip && <Tooltip tip={tip} />}
    </div>
  );
}

export { CATEGORY_COLORS };
