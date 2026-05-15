import { useState, useCallback } from "react";
import { FolderOpen, RefreshCw } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { formatBytes } from "../../lib/utils";
import { EmptyState } from "../../components/empty-state";
import { scanTree } from "../../lib/tauri";
import { useTauriEvent } from "../../hooks/use-tauri-event";
import type { TreeNode } from "../../bindings";

export function ExplorerPage() {
  const [rootPath, setRootPath] = useState("C:\\");
  const [nodes, setNodes] = useState<TreeNode[]>([]);
  const [scanning, setScanning] = useState(false);

  const handleNode = useCallback((node: TreeNode) => {
    setNodes((prev) =>
      [...prev, node].sort((a, b) => b.size_bytes - a.size_bytes)
    );
  }, []);

  const handleDone = useCallback(() => {
    setScanning(false);
  }, []);

  useTauriEvent<TreeNode>("explorer:node", handleNode);
  useTauriEvent("explorer:done", handleDone);

  const handleScan = async () => {
    setNodes([]);
    setScanning(true);
    try {
      await scanTree({
        root: rootPath,
        max_depth: 2,
        follow_reparse_points: false,
        include_hidden: true,
        min_size_bytes: null,
        size_strategy: "Logical",
      });
    } catch {
      setScanning(false);
    }
  };

  return (
    <div className="p-6 h-full flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <h2 className="text-2xl font-bold">Explorador de archivos</h2>
        {scanning && (
          <span className="text-sm text-muted-foreground animate-pulse">
            Escaneando...
          </span>
        )}
      </div>

      <div className="flex gap-2">
        <Input
          value={rootPath}
          onChange={(e) => setRootPath(e.target.value)}
          placeholder="Ruta raíz (ej. C:\)"
          className="flex-1"
          onKeyDown={(e) => e.key === "Enter" && handleScan()}
        />
        <Button onClick={handleScan} disabled={scanning}>
          {scanning ? (
            <RefreshCw className="h-4 w-4 animate-spin" />
          ) : (
            <FolderOpen className="h-4 w-4" />
          )}
          <span className="ml-2">{scanning ? "Escaneando..." : "Escanear"}</span>
        </Button>
      </div>

      {nodes.length === 0 && !scanning ? (
        <EmptyState
          icon={FolderOpen}
          title="Sin resultados"
          description='Introduce una ruta y haz clic en "Escanear"'
        />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="text-left p-3 font-medium text-muted-foreground">Nombre</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Tipo</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Tamaño</th>
              </tr>
            </thead>
            <tbody>
              {nodes.map((node) => (
                <tr
                  key={node.path}
                  className="border-b border-border/50 hover:bg-accent/30"
                >
                  <td className="p-3">
                    <div className="font-medium truncate max-w-xs" title={node.path}>
                      {node.name}
                    </div>
                    <div className="text-xs text-muted-foreground truncate max-w-xs">
                      {node.path}
                    </div>
                  </td>
                  <td className="p-3 text-muted-foreground text-xs">{node.kind}</td>
                  <td className="p-3 text-right font-mono text-xs">
                    {formatBytes(node.size_bytes)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
