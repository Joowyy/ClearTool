/*
 * disk-page — Disk Analyzer con treemap canvas 2D (d3-hierarchy).
 *
 * Canvas en lugar de SVG: con 5000+ nodos SVG sería inaceptable.
 * Click para zoom, hover para tooltip, colores por extensión.
 */
import { useState, useRef, useEffect, useMemo, useCallback } from "react";
import * as d3h from "d3-hierarchy";
import { HardDrive, FolderOpen, ArrowLeft } from "lucide-react";
import { EmptyState } from "../../components/empty-state";
import { Button } from "../../components/ui/button";
import { buildTreemapData } from "../../api/client";
import type { TreemapNode } from "../../api/types";
import { formatBytes } from "../../lib/utils";

// ── colores por extensión ──────────────────────────────────────────────

const EXT_CATEGORIES: Record<string, string> = {
  // video/audio
  mp4: "media", mkv: "media", avi: "media", mov: "media", mp3: "media",
  wav: "media", flac: "media", m4a: "media", wmv: "media",
  // imágenes
  jpg: "image", jpeg: "image", png: "image", gif: "image", webp: "image",
  bmp: "image", svg: "image", ico: "image",
  // código
  ts: "code", tsx: "code", js: "code", jsx: "code", rs: "code",
  py: "code", go: "code", java: "code", cpp: "code", c: "code",
  h: "code", css: "code", html: "code", json: "code", toml: "code",
  // docs
  pdf: "docs", docx: "docs", xlsx: "docs", pptx: "docs", txt: "docs",
  md: "docs",
  // archivos comprimidos
  zip: "archive", rar: "archive", "7z": "archive", tar: "archive",
  gz: "archive",
  // ejecutables
  exe: "executable", dll: "executable", msi: "executable", bat: "executable",
  ps1: "executable",
};

const CATEGORY_COLORS: Record<string, string> = {
  media:      "#8b5cf6",
  image:      "#06b6d4",
  code:       "#22c55e",
  docs:       "#3b82f6",
  archive:    "#f59e0b",
  executable: "#ef4444",
  other:      "#6b7280",
  dir:        "#374151",
};

function extColor(node: TreemapNode): string {
  if (node.kind === "Dir") return CATEGORY_COLORS.dir;
  const ext = (node.extension ?? "").toLowerCase();
  const cat = EXT_CATEGORIES[ext] ?? "other";
  return CATEGORY_COLORS[cat];
}

// ── layout helpers ─────────────────────────────────────────────────────

interface LayoutLeaf {
  x0: number; y0: number; x1: number; y1: number;
  data: TreemapNode;
  value: number;
}

function buildLayout(root: TreemapNode, width: number, height: number): LayoutLeaf[] {
  const hier = d3h
    .hierarchy<TreemapNode>(root, (d: TreemapNode) => d.children ?? null)
    .sum((d: TreemapNode) => (!d.children || d.children.length === 0 ? d.sizeBytes : 0))
    .sort((a: d3h.HierarchyNode<TreemapNode>, b: d3h.HierarchyNode<TreemapNode>) => (b.value ?? 0) - (a.value ?? 0));

  const tm = d3h
    .treemap<TreemapNode>()
    .size([width, height])
    .paddingInner(1)
    .round(true);

  tm(hier);

  const rootValue = hier.value ?? 1;
  const MIN_FRACTION = 0.001; // omitir nodos < 0.1% del root

  const leaves: LayoutLeaf[] = [];
  hier.each((node: d3h.HierarchyNode<TreemapNode>) => {
    const n = node as d3h.HierarchyRectangularNode<TreemapNode>;
    if (n.x0 === undefined) return;
    if ((n.value ?? 0) / rootValue < MIN_FRACTION) return;
    leaves.push({ x0: n.x0, y0: n.y0, x1: n.x1, y1: n.y1, data: n.data, value: n.value ?? 0 });
  });
  return leaves;
}

// ── canvas drawing ─────────────────────────────────────────────────────

const FONT = "11px Inter, system-ui, sans-serif";

function drawTreemap(ctx: CanvasRenderingContext2D, leaves: LayoutLeaf[], dpr: number) {
  ctx.save();
  ctx.scale(dpr, dpr);

  for (const leaf of leaves) {
    const w = leaf.x1 - leaf.x0;
    const h = leaf.y1 - leaf.y0;
    if (w < 1 || h < 1) continue;

    ctx.fillStyle = extColor(leaf.data);
    ctx.fillRect(leaf.x0, leaf.y0, w, h);

    if (w > 40 && h > 18) {
      ctx.fillStyle = "rgba(255,255,255,0.85)";
      ctx.font = FONT;
      ctx.textBaseline = "middle";
      const label = leaf.data.name;
      const maxW = w - 6;
      let text = label;
      if (ctx.measureText(text).width > maxW) {
        while (text.length > 1 && ctx.measureText(text + "…").width > maxW) {
          text = text.slice(0, -1);
        }
        text += "…";
      }
      ctx.fillText(text, leaf.x0 + 3, leaf.y0 + h / 2);
    }
  }
  ctx.restore();
}

// ── Tooltip ────────────────────────────────────────────────────────────

interface TooltipState { x: number; y: number; node: TreemapNode; value: number }

function Tooltip({ tip }: { tip: TooltipState }) {
  return (
    <div
      className="pointer-events-none fixed z-50 panel-raised rounded-md px-3 py-2 text-xs shadow-lg"
      style={{ left: tip.x + 12, top: tip.y - 10 }}
    >
      <div className="font-medium text-ink-primary truncate max-w-[260px]">{tip.node.name}</div>
      <div className="text-ink-tertiary mt-0.5">{formatBytes(tip.value)}</div>
      {tip.node.extension && (
        <div className="text-ink-muted">.{tip.node.extension}</div>
      )}
    </div>
  );
}

// ── TreemapCanvas ──────────────────────────────────────────────────────

interface TreemapCanvasProps {
  root: TreemapNode;
  onNavigate: (node: TreemapNode) => void;
}

function TreemapCanvas({ root, onNavigate }: TreemapCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [size, setSize] = useState({ w: 0, h: 0 });
  const [tip, setTip] = useState<TooltipState | null>(null);
  const leavesRef = useRef<LayoutLeaf[]>([]);

  // ResizeObserver con debounce 200ms
  useEffect(() => {
    if (!containerRef.current) return;
    let timer: ReturnType<typeof setTimeout>;
    const ro = new ResizeObserver((entries) => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const { width, height } = entries[0].contentRect;
        setSize({ w: Math.floor(width), h: Math.floor(height) });
      }, 200);
    });
    ro.observe(containerRef.current);
    return () => { clearTimeout(timer); ro.disconnect(); };
  }, []);

  const leaves = useMemo(() => {
    if (size.w === 0 || size.h === 0) return [];
    return buildLayout(root, size.w, size.h);
  }, [root, size.w, size.h]);

  leavesRef.current = leaves;

  // Dibujar en canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || leaves.length === 0) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = size.w * dpr;
    canvas.height = size.h * dpr;
    canvas.style.width = `${size.w}px`;
    canvas.style.height = `${size.h}px`;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawTreemap(ctx, leaves, dpr);
  }, [leaves, size]);

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

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const leaf = hitTest(e.clientX, e.clientY);
    if (leaf) setTip({ x: e.clientX, y: e.clientY, node: leaf.data, value: leaf.value });
    else setTip(null);
  }, [hitTest]);

  const handleClick = useCallback((e: React.MouseEvent) => {
    const leaf = hitTest(e.clientX, e.clientY);
    if (leaf && leaf.data.kind === "Dir" && leaf.data.children?.length) {
      onNavigate(leaf.data);
    }
  }, [hitTest, onNavigate]);

  const handleMouseLeave = useCallback(() => setTip(null), []);

  return (
    <div ref={containerRef} className="w-full h-full relative">
      <canvas
        ref={canvasRef}
        className="block cursor-pointer"
        style={{ width: "100%", height: "100%" }}
        onMouseMove={handleMouseMove}
        onClick={handleClick}
        onMouseLeave={handleMouseLeave}
      />
      {tip && <Tooltip tip={tip} />}
    </div>
  );
}

// ── DiskPage ───────────────────────────────────────────────────────────

export function DiskPage() {
  const [scanning, setScanning] = useState(false);
  const [rootData, setRootData] = useState<TreemapNode | null>(null);
  const [navStack, setNavStack] = useState<TreemapNode[]>([]);
  const [rootPath, setRootPath] = useState("C:\\");

  const currentNode = navStack.length > 0 ? navStack[navStack.length - 1] : rootData;

  const handleScan = async () => {
    setScanning(true);
    setNavStack([]);
    try {
      const data = await buildTreemapData({
        root: rootPath,
        maxDepth: 4,
        minSizeMb: 10,
        followReparsePoints: false,
      });
      setRootData(data);
    } catch (err) {
      console.error("Failed to build treemap:", err);
    } finally {
      setScanning(false);
    }
  };

  const handleNavigate = useCallback((node: TreemapNode) => {
    setNavStack((s) => [...s, node]);
  }, []);

  const handleBack = useCallback(() => {
    setNavStack((s) => s.slice(0, -1));
  }, []);

  if (!rootData && !scanning) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex items-center justify-between p-4 border-b border-edge-default/10">
          <div>
            <h1 className="text-sm font-semibold text-ink-primary">Disk Analyzer</h1>
            <p className="text-xs text-ink-tertiary mt-0.5">Visualiza el uso del disco con treemap</p>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={rootPath}
              onChange={(e) => setRootPath(e.target.value)}
              className="inset px-2.5 h-8 text-xs text-ink-primary w-40 font-mono"
              placeholder="C:\\"
            />
            <Button onClick={handleScan} disabled={scanning}>
              <FolderOpen className="h-3.5 w-3.5 mr-1.5" />
              Escanear
            </Button>
          </div>
        </div>
        <EmptyState
          icon={HardDrive}
          title="Sin análisis de disco"
          description="Selecciona una ruta y pulsa Escanear para generar el treemap."
          action={{ label: "Escanear C:\\", onClick: handleScan }}
        />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between p-4 border-b border-edge-default/10">
        <div className="flex items-center gap-2">
          {navStack.length > 0 && (
            <button
              onClick={handleBack}
              className="p-1 rounded hover:bg-white/10 text-ink-tertiary hover:text-ink-primary transition-colors"
              title="Subir"
            >
              <ArrowLeft className="h-4 w-4" />
            </button>
          )}
          <div>
            <h1 className="text-sm font-semibold text-ink-primary">Disk Analyzer</h1>
            <p className="text-xs text-ink-tertiary mt-0.5 font-mono">
              {currentNode?.path ?? rootPath}
              {scanning ? " · Escaneando..." : ""}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={rootPath}
            onChange={(e) => setRootPath(e.target.value)}
            className="inset px-2.5 h-8 text-xs text-ink-primary w-40 font-mono"
            placeholder="C:\\"
          />
          <Button onClick={handleScan} disabled={scanning}>
            <FolderOpen className="h-3.5 w-3.5 mr-1.5" />
            {scanning ? "Escaneando..." : "Re-escanear"}
          </Button>
        </div>
      </div>

      <div className="flex-1 p-2 min-h-0">
        {scanning ? (
          <div className="flex items-center justify-center h-full text-xs text-ink-muted">
            Escaneando disco...
          </div>
        ) : currentNode ? (
          <TreemapCanvas
            key={currentNode.path}
            root={currentNode}
            onNavigate={handleNavigate}
          />
        ) : null}
      </div>

      {/* Leyenda de colores */}
      <div className="px-4 py-2 border-t border-edge-default/10 flex items-center gap-4 flex-wrap">
        {Object.entries(CATEGORY_COLORS).filter(([k]) => k !== "dir").map(([cat, color]) => (
          <div key={cat} className="flex items-center gap-1.5">
            <div className="w-2.5 h-2.5 rounded-sm" style={{ backgroundColor: color }} />
            <span className="text-[10px] text-ink-tertiary capitalize">{cat}</span>
          </div>
        ))}
        <span className="text-[10px] text-ink-muted ml-auto">Click en carpeta para zoom</span>
      </div>
    </div>
  );
}
