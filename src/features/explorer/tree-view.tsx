// Componente contenedor del árbol del explorador.
import { FolderOpen, RefreshCw, X } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { TreeRow } from "./tree-row";
import { EmptyState } from "../../components/empty-state";
import type { TreeNode } from "../../api";
import type { NodeState } from "./use-explorer-tree";

interface TreeViewProps {
  rootPath: string;
  nodes: TreeNode[];
  scanning: boolean;
  onScan: (path: string) => void;
  onExpand: (path: string) => void;
  onCancel: () => void;
  byPath: Map<string, NodeState>;
}

export function TreeView({ rootPath, nodes, scanning, onScan, onExpand, onCancel, byPath }: TreeViewProps) {
  return (
    <div className="flex flex-col gap-4 h-full">
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
          onChange={(e) => onScan(e.target.value)}
          placeholder="Ruta raíz (ej. C:\)"
          className="flex-1"
          onKeyDown={(e) => e.key === "Enter" && onScan(rootPath)}
        />
        <Button onClick={() => onScan(rootPath)} disabled={scanning}>
          {scanning ? (
            <RefreshCw className="h-4 w-4 animate-spin" />
          ) : (
            <FolderOpen className="h-4 w-4" />
          )}
          <span className="ml-2">{scanning ? "Escaneando..." : "Escanear"}</span>
        </Button>
        {scanning && (
          <Button variant="outline" onClick={onCancel}>
            <X className="h-4 w-4" />
            <span className="ml-2">Cancelar</span>
          </Button>
        )}
      </div>

      {nodes.length === 0 && !scanning ? (
        <EmptyState
          icon={FolderOpen}
          title="Sin resultados"
          description='Introduce una ruta y haz clic en "Escanear"'
        />
      ) : nodes.length === 0 && scanning ? (
        <div className="flex-1 overflow-auto">
          {Array.from({ length: 8 }).map((_, i) => (
            <div
              key={i}
              className="flex items-center gap-3 px-4 py-3 animate-pulse border-b border-border/30"
            >
              <div className="w-4 h-4 bg-muted rounded" />
              <div className="w-4 h-4 bg-muted rounded" />
              <div className="h-4 bg-muted rounded flex-1" />
              <div className="h-4 bg-muted rounded w-20" />
            </div>
          ))}
        </div>
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="text-left p-3 font-medium text-muted-foreground w-8" />
                <th className="text-left p-3 font-medium text-muted-foreground">Nombre</th>
                <th className="text-left p-3 font-medium text-muted-foreground w-24">Tipo</th>
                <th className="text-right p-3 font-medium text-muted-foreground w-32">Tamaño</th>
              </tr>
            </thead>
            <tbody>
              {nodes.map((node) => (
                <TreeRow
                  key={node.path}
                  node={node}
                  depth={0}
                  onExpand={onExpand}
                  byPath={byPath}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
