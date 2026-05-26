// Fila individual del árbol del explorador — renderiza recursivamente.
import { useState, useCallback } from "react";
import {
  ChevronRight,
  ChevronDown,
  Folder,
  FolderOpen,
  File,
  FileText,
  FileImage,
  FileCode,
} from "lucide-react";
import { formatBytes } from "../../lib/utils";
import type { TreeNode } from "../../api";
import type { NodeState } from "./use-explorer-tree";

interface TreeRowProps {
  node: TreeNode;
  depth: number;
  onExpand: (path: string) => void;
  byPath: Map<string, NodeState>;
}

function fileIcon(name: string) {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  const imageExts = ["png", "jpg", "jpeg", "gif", "bmp", "svg", "ico", "webp"];
  const codeExts = ["ts", "tsx", "js", "jsx", "rs", "py", "css", "html", "json", "toml", "md"];
  const textExts = ["txt", "log", "cfg", "ini", "conf", "yaml", "yml"];

  if (imageExts.includes(ext)) return FileImage;
  if (codeExts.includes(ext)) return FileCode;
  if (textExts.includes(ext)) return FileText;
  return File;
}

function sizeColor(bytes: number): string {
  if (bytes === 0) return "text-muted-foreground";
  if (bytes < 1024 * 1024) return "text-green-400";
  if (bytes < 1024 * 1024 * 1024) return "text-yellow-400";
  return "text-red-400";
}

export function TreeRow({ node, depth, onExpand, byPath }: TreeRowProps) {
  const [expanded, setExpanded] = useState(false);

  const handleClick = useCallback(() => {
    if (node.kind !== "Dir") return;
    if (!expanded) onExpand(node.path);
    setExpanded((v) => !v);
  }, [node.kind, node.path, expanded, onExpand]);

  const isDir = node.kind === "Dir";
  const Icon = isDir ? (expanded ? FolderOpen : Folder) : fileIcon(node.name);
  const sizeClass = sizeColor(node.sizeBytes);

  const nodeState = byPath.get(node.path);
  const childNodes = nodeState?.children;
  const isLoading = childNodes === "loading";
  const hasError = childNodes === "error";

  return (
    <>
      <tr
        className="border-b border-border/30 hover:bg-accent/20 cursor-pointer group"
        onClick={handleClick}
      >
        <td className="p-2 pl-4 w-8">
          {isDir ? (
            expanded ? (
              <ChevronDown className="h-4 w-4 text-muted-foreground" />
            ) : (
              <ChevronRight className="h-4 w-4 text-muted-foreground" />
            )
          ) : (
            <div className="w-4" />
          )}
        </td>
        <td className="p-2">
          <div className="flex items-center gap-2" style={{ paddingLeft: depth * 20 }}>
            <Icon className="h-4 w-4 flex-shrink-0 text-muted-foreground" />
            <span
              className="font-medium truncate max-w-md group-hover:text-foreground"
              title={node.path}
            >
              {node.name}
            </span>
          </div>
        </td>
        <td className="p-2 text-muted-foreground text-xs">
          {isDir ? "Carpeta" : node.kind}
        </td>
        <td className={`p-2 text-right font-mono text-xs ${sizeClass}`}>
          {isDir ? (
            node.sizeBytes > 0 ? (
              formatBytes(node.sizeBytes)
            ) : (
              <span className="text-muted-foreground">—</span>
            )
          ) : (
            formatBytes(node.sizeBytes)
          )}
        </td>
      </tr>

      {expanded && isDir && isLoading && (
        <tr>
          <td colSpan={4} className="p-2 text-xs text-muted-foreground animate-pulse"
            style={{ paddingLeft: (depth + 1) * 20 + 16 }}>
            Cargando...
          </td>
        </tr>
      )}

      {expanded && isDir && hasError && (
        <tr>
          <td colSpan={4} className="p-2 text-xs text-red-400"
            style={{ paddingLeft: (depth + 1) * 20 + 16 }}>
            Error al leer directorio
          </td>
        </tr>
      )}

      {expanded && isDir && Array.isArray(childNodes) &&
        childNodes.map((child) => (
          <TreeRow
            key={child.path}
            node={child}
            depth={depth + 1}
            onExpand={onExpand}
            byPath={byPath}
          />
        ))
      }
    </>
  );
}
