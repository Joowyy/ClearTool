// Hook para el árbol del explorador con carga lazy de nodos y tamaños.
import { useState, useCallback, useRef } from "react";
import { scanTree, computeDirectorySize, type TreeNode } from "../../api";

export type NodeState = {
  children: TreeNode[] | "loading" | "error";
  size: number | "computing" | undefined;
  expanded: boolean;
};

type TreeState = {
  rootPath: string;
  byPath: Map<string, NodeState>;
  nodes: TreeNode[];
  scanning: boolean;
};

export function useExplorerTree() {
  const [state, setState] = useState<TreeState>({
    rootPath: "C:\\",
    byPath: new Map(),
    nodes: [],
    scanning: false,
  });

  const scanIdRef = useRef<string | null>(null);

  const scanRoot = useCallback(async (path: string) => {
    setState((prev) => ({
      ...prev,
      rootPath: path,
      nodes: [],
      byPath: new Map(),
      scanning: true,
    }));

    try {
      const handle = await scanTree({
        root: path,
        maxDepth: 1,
        followReparsePoints: false,
        includeHidden: true,
        minSizeBytes: null,
        sizeStrategy: "Logical",
      });
      scanIdRef.current = handle.scanId;
    } catch {
      setState((prev) => ({ ...prev, scanning: false }));
    }
  }, []);

  const addNode = useCallback((node: TreeNode) => {
    setState((prev) => ({
      ...prev,
      nodes: [...prev.nodes, node].sort((a, b) => {
        if (a.kind === "Dir" && b.kind !== "Dir") return -1;
        if (a.kind !== "Dir" && b.kind === "Dir") return 1;
        return b.sizeBytes - a.sizeBytes;
      }),
    }));
  }, []);

  const scanDone = useCallback(() => {
    setState((prev) => ({ ...prev, scanning: false }));
    scanIdRef.current = null;
  }, []);

  const expandNode = useCallback(async (path: string) => {
    setState((prev) => {
      const existing = prev.byPath.get(path);
      if (existing && existing.expanded) return prev;
      const next = new Map(prev.byPath);
      next.set(path, {
        children: existing?.children === "error" ? "loading" : (existing?.children ?? "loading"),
        size: existing?.size ?? undefined,
        expanded: true,
      });
      return { ...prev, byPath: next };
    });

    try {
      const [childrenResult, sizeResult] = await Promise.allSettled([
        scanTree({
          root: path,
          maxDepth: 1,
          followReparsePoints: false,
          includeHidden: true,
          minSizeBytes: null,
          sizeStrategy: "Logical",
        }),
        computeDirectorySize(path, false),
      ]);

      setState((prev) => {
        const next = new Map(prev.byPath);

        if (childrenResult.status === "fulfilled") {
          next.set(path, {
            children: [] as TreeNode[],
            size: sizeResult.status === "fulfilled" ? sizeResult.value.logicalBytes : undefined,
            expanded: true,
          });
        } else {
          next.set(path, {
            children: "error",
            size: undefined,
            expanded: true,
          });
        }

        return { ...prev, byPath: next };
      });
    } catch {
      setState((prev) => {
        const next = new Map(prev.byPath);
        next.set(path, { children: "error", size: undefined, expanded: true });
        return { ...prev, byPath: next };
      });
    }
  }, []);

  const collapseNode = useCallback((path: string) => {
    setState((prev) => {
      const existing = prev.byPath.get(path);
      if (!existing) return prev;
      const next = new Map(prev.byPath);
      next.set(path, { ...existing, expanded: false });
      return { ...prev, byPath: next };
    });
  }, []);

  const addChildNodes = useCallback((parentId: string, children: TreeNode[]) => {
    setState((prev) => {
      const existing = prev.byPath.get(parentId);
      if (!existing) return prev;
      const next = new Map(prev.byPath);
      next.set(parentId, {
        ...existing,
        children,
        size: existing.size === "computing" ? existing.size : existing.size,
      });
      return { ...prev, byPath: next };
    });
  }, []);

  return {
    state,
    scanRoot,
    addNode,
    scanDone,
    expandNode,
    collapseNode,
    addChildNodes,
  };
}
