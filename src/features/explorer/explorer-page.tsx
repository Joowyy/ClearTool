import { useCallback, useState } from "react";
import { useTauriEvent } from "../../hooks/use-tauri-event";
import { useExplorerTree } from "./use-explorer-tree";
import { TreeView } from "./tree-view";
import type { TreeNode } from "../../api";

export function ExplorerPage() {
  const {
    state,
    scanRoot,
    addNode,
    scanDone,
    expandNode,
  } = useExplorerTree();

  const [search, setSearch] = useState("");

  const handleNode = useCallback(
    (node: TreeNode) => {
      addNode(node);
    },
    [addNode]
  );

  const handleDone = useCallback(() => {
    scanDone();
  }, [scanDone]);

  useTauriEvent("explorer:node", handleNode);
  useTauriEvent("explorer:done", handleDone);

  const handleScan = useCallback(
    (path: string) => {
      scanRoot(path);
    },
    [scanRoot]
  );

  const handleExpand = useCallback(
    (path: string) => {
      expandNode(path);
    },
    [expandNode]
  );

  const handleCancel = useCallback(() => {
    // Cancelación: el backend actual no soporta cancel real,
    // pero marcamos scanning = false en el frontend.
    scanDone();
  }, [scanDone]);

  // Filtrar nodos por búsqueda local
  const filteredNodes = search
    ? state.nodes.filter(
        (n) =>
          n.name.toLowerCase().includes(search.toLowerCase()) ||
          n.path.toLowerCase().includes(search.toLowerCase())
      )
    : state.nodes;

  return (
    <div className="p-6 h-full flex flex-col gap-4">
      <TreeView
        rootPath={state.rootPath}
        nodes={filteredNodes}
        scanning={state.scanning}
        onScan={handleScan}
        onExpand={handleExpand}
        onCancel={handleCancel}
        byPath={state.byPath}
      />

      {/* Barra de búsqueda local */}
      {state.nodes.length > 0 && (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Buscar en resultados..."
            className="flex-1 px-3 py-1.5 text-sm bg-muted/50 border border-border rounded-md focus:outline-none focus:ring-1 focus:ring-ring"
          />
          {search && (
            <span className="text-xs text-muted-foreground">
              {filteredNodes.length} de {state.nodes.length}
            </span>
          )}
        </div>
      )}
    </div>
  );
}
