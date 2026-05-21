// Fila individual del árbol del explorador.
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

interface TreeRowProps {
  node: TreeNode;
  depth: number;
  onExpand: (path: string) => void;
  children?: React.ReactNode;
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

export function TreeRow({ node, depth, onExpand, children }: TreeRowProps) {
  const [expanded, setExpanded] = useState(false);

  const handleExpand = useCallback(() => {
    if (node.kind === "Dir") {
      if (!expanded) {
        onExpand(node.path);
      }
      setExpanded((prev) => !prev);
    }
  }, [node.kind, node.path, expanded, onExpand]);

  const isDir = node.kind === "Dir";
  const Icon = isDir ? (expanded ? FolderOpen : Folder) : fileIcon(node.name);
  const sizeClass = sizeColor(node.sizeBytes);

  return (
    <>
      <tr
        className="border-b border-border/30 hover:bg-accent/20 cursor-pointer group"
        onClick={handleExpand}
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
            <div
              className="absolute left-0 top-0 bottom-0 w-px bg-border/30"
              style={{ left: `${depth * 20 + 12}px` }}
            />
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
      {expanded && isDir && children}
    </>
  );
}
