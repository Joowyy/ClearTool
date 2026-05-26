# Paso 02 — Frontend canvas treemap con d3-hierarchy

**Área**: 07-disk-analyzer
**Tiempo estimado**: 6-8 horas
**Dependencias**: Paso 01

## Qué hacemos

Componente React que renderiza el treemap en `<canvas>` usando `d3-hierarchy` para calcular layouts y Canvas 2D para dibujar.

## Archivos

- `package.json` (deps: `d3-hierarchy`)
- `src/features/disk-analyzer/disk-analyzer-page.tsx` (nuevo)
- `src/features/disk-analyzer/components/treemap-canvas.tsx` (nuevo)
- `src/features/disk-analyzer/lib/treemap-layout.ts` (nuevo)

## Cómo

### 1. Deps

```bash
npm i d3-hierarchy
npm i -D @types/d3-hierarchy
```

### 2. Componente treemap

```tsx
// src/features/disk-analyzer/components/treemap-canvas.tsx
import { useRef, useEffect, useState } from "react";
import { hierarchy, treemap, HierarchyRectangularNode } from "d3-hierarchy";
import type { TreemapNode } from "../../../api";

interface TreemapCanvasProps {
  data: TreemapNode;
  width: number;
  height: number;
  onHover?: (node: HierarchyRectangularNode<TreemapNode> | null) => void;
  onClick?: (node: HierarchyRectangularNode<TreemapNode>) => void;
  onDoubleClick?: (node: HierarchyRectangularNode<TreemapNode>) => void;
}

const EXT_COLORS: Record<string, string> = {
  // Media
  mp4: "#dc2626", mkv: "#dc2626", avi: "#dc2626", mov: "#dc2626",
  // Audio
  mp3: "#f97316", flac: "#f97316", wav: "#f97316",
  // Images
  jpg: "#10b981", jpeg: "#10b981", png: "#10b981", gif: "#10b981",
  // Archives
  zip: "#a855f7", rar: "#a855f7", "7z": "#a855f7", iso: "#a855f7",
  // Executables
  exe: "#0891b2", msi: "#0891b2", dll: "#0891b2",
  // Code
  ts: "#2563eb", js: "#eab308", rs: "#ea580c", py: "#3b82f6",
  // Docs
  pdf: "#dc2626", docx: "#2563eb", xlsx: "#16a34a",
  // Default
  __default: "#6b7280",
};

const DIR_COLOR = "#1e293b";

export function TreemapCanvas({
  data, width, height, onHover, onClick, onDoubleClick,
}: TreemapCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hovered, setHovered] = useState<HierarchyRectangularNode<TreemapNode> | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // DPI scaling para nitidez
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.scale(dpr, dpr);

    // Construir hierarchy
    const root = hierarchy<TreemapNode>(data, (d) => d.children)
      .sum((d) => (d.isDir ? 0 : d.sizeBytes))   // peso = solo hojas
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

    const layout = treemap<TreemapNode>()
      .size([width, height])
      .padding(1)
      .round(true);
    layout(root);

    // Dibujar
    ctx.clearRect(0, 0, width, height);
    const rectNodes = root.descendants() as HierarchyRectangularNode<TreemapNode>[];

    // Solo hojas
    rectNodes
      .filter((n) => !n.children || n.children.length === 0)
      .forEach((n) => {
        const w = (n.x1 ?? 0) - (n.x0 ?? 0);
        const h = (n.y1 ?? 0) - (n.y0 ?? 0);
        if (w < 2 || h < 2) return;

        const ext = n.data.extension ?? "__default";
        const color = EXT_COLORS[ext] ?? EXT_COLORS.__default;

        ctx.fillStyle = color;
        ctx.fillRect(n.x0!, n.y0!, w, h);
        ctx.strokeStyle = "rgba(0,0,0,0.3)";
        ctx.lineWidth = 0.5;
        ctx.strokeRect(n.x0!, n.y0!, w, h);

        // Label si cabe
        if (w > 60 && h > 20) {
          ctx.fillStyle = "white";
          ctx.font = "10px monospace";
          const label = truncate(n.data.name, Math.floor(w / 6));
          ctx.fillText(label, n.x0! + 4, n.y0! + 12);
        }
      });

    // Highlight hovered
    if (hovered) {
      ctx.strokeStyle = "#06b6d4";
      ctx.lineWidth = 2;
      ctx.strokeRect(hovered.x0!, hovered.y0!, hovered.x1! - hovered.x0!, hovered.y1! - hovered.y0!);
    }
  }, [data, width, height, hovered]);

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const root = hierarchy<TreemapNode>(data, (d) => d.children)
      .sum((d) => (d.isDir ? 0 : d.sizeBytes));
    const layout = treemap<TreemapNode>().size([width, height]).padding(1).round(true);
    layout(root);

    const hit = (root.descendants() as HierarchyRectangularNode<TreemapNode>[])
      .filter((n) => !n.children || n.children.length === 0)
      .find((n) => x >= n.x0! && x <= n.x1! && y >= n.y0! && y <= n.y1!);

    setHovered(hit ?? null);
    onHover?.(hit ?? null);
  };

  return (
    <canvas
      ref={canvasRef}
      onMouseMove={handleMouseMove}
      onMouseLeave={() => { setHovered(null); onHover?.(null); }}
      onClick={() => hovered && onClick?.(hovered)}
      onDoubleClick={() => hovered && onDoubleClick?.(hovered)}
      className="cursor-pointer"
    />
  );
}

function truncate(s: string, max: number) {
  return s.length > max ? s.slice(0, max - 1) + "…" : s;
}
```

### 3. Page wrapper

```tsx
// src/features/disk-analyzer/disk-analyzer-page.tsx
import { useState, useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { buildTreemapData } from "../../api";
import { TreemapCanvas } from "./components/treemap-canvas";
import { Input } from "../../components/ui/input";
import { Button } from "../../components/ui/button";
import { formatBytes } from "../../lib/utils";

export function DiskAnalyzerPage() {
  const [root, setRoot] = useState("C:\\Users");
  const [maxDepth, setMaxDepth] = useState(4);
  const [hovered, setHovered] = useState<any>(null);

  const { data, isLoading, isFetching, refetch } = useQuery({
    queryKey: ["treemap", root, maxDepth],
    queryFn: () => buildTreemapData({
      root,
      maxDepth,
      minSizeBytes: 10 * 1024 * 1024,
      followReparsePoints: false,
    }),
    enabled: false,  // sólo on demand
  });

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <h2 className="text-2xl font-bold">Análisis de disco</h2>

      <div className="flex gap-2">
        <Input value={root} onChange={(e) => setRoot(e.target.value)} placeholder="C:\\" />
        <Button onClick={() => refetch()} disabled={isFetching}>
          {isFetching ? "Escaneando..." : "Analizar"}
        </Button>
      </div>

      {data && (
        <div className="flex-1 flex flex-col border border-border rounded-lg overflow-hidden">
          <div className="p-3 border-b border-border text-sm">
            {hovered ? (
              <div className="flex justify-between">
                <span className="font-mono">{hovered.data.path}</span>
                <span>{formatBytes(hovered.data.sizeBytes)}</span>
              </div>
            ) : (
              <span className="text-muted-foreground">
                Total: {formatBytes(data.sizeBytes)} · Pasa el cursor para info
              </span>
            )}
          </div>
          <div className="flex-1 p-2">
            <TreemapCanvas
              data={data}
              width={1200}
              height={700}
              onHover={setHovered}
            />
          </div>
        </div>
      )}
    </div>
  );
}
```

## Criterio de done

- [ ] Página `/disk` accesible y renderiza treemap.
- [ ] Color por extensión visible.
- [ ] Hover muestra path + tamaño.
- [ ] Render <500ms para 5k nodos.
- [ ] DPI scaling correcto (no se ve borroso en pantallas HiDPI).
- [ ] Labels solo en rectángulos grandes.
