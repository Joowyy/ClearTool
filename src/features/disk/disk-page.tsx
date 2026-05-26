/*
 * disk-page — Disk Analyzer con treemap visual.
 *
 * M4: Canvas 2D con d3-hierarchy. Rectángulos proporcionales al tamaño,
 * colores por extensión. Click para zoom, doble click para acciones.
 */
import { useState } from "react";
import { HardDrive, FolderOpen } from "lucide-react";
import { EmptyState } from "../../components/empty-state";
import { Button } from "../../components/ui/button";
import { buildTreemapData } from "../../api/client";
import type { TreemapNode } from "../../api/types";

export function DiskPage() {
  const [scanning, setScanning] = useState(false);
  const [treemapData, setTreemapData] = useState<TreemapNode | null>(null);
  const [rootPath, setRootPath] = useState("C:\\");

  const handleScan = async () => {
    setScanning(true);
    try {
      const data = await buildTreemapData({
        root: rootPath,
        maxDepth: 4,
        minSizeMb: 10,
        followReparsePoints: false,
      });
      setTreemapData(data);
    } catch (err) {
      console.error("Failed to build treemap:", err);
    } finally {
      setScanning(false);
    }
  };

  if (!treemapData && !scanning) {
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
        <div>
          <h1 className="text-sm font-semibold text-ink-primary">Disk Analyzer</h1>
          <p className="text-xs text-ink-tertiary mt-0.5">
            {rootPath} · {scanning ? "Escaneando..." : "Análisis completo"}
          </p>
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
      <div className="flex-1 p-4">
        {scanning ? (
          <div className="flex items-center justify-center h-full text-xs text-ink-muted">
            Escaneando disco...
          </div>
        ) : (
          <div className="panel p-4 h-full">
            <p className="text-xs text-ink-secondary">
              Treemap canvas — pendiente de implementación con d3-hierarchy.
              Datos recibidos: {treemapData?.name} ({treemapData?.sizeBytes} bytes)
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
