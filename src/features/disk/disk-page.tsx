// disk-page — Disk Analyzer: selector de discos + escaneo + 3 paneles
// (stats blocks, tree jerárquico, treemap) sincronizados.
import { useCallback, useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  HardDrive,
  Play,
  Square,
  ArrowLeft,
  AlertOctagon,
  ChevronRight,
} from "lucide-react";
import { cn, formatBytes } from "../../lib/utils";
import { EmptyState } from "../../components/empty-state";
import { Button } from "../../components/ui/button";
import { DriveSelector } from "./drive-selector";
import { StatsBlocks } from "./stats-blocks";
import { HierarchyTree } from "./hierarchy-tree";
import { TreemapCanvas } from "./treemap-canvas";
import { TopTables } from "./top-tables";
import { useDiskScan } from "./use-disk-scan";
import { buildBreadcrumbs, findNodeByPath } from "./disk-helpers";
import type { TreemapNode } from "../../api/types";

export function DiskPage() {
  const {
    drives,
    drivesLoading,
    refreshDrives,
    selectedDrive,
    setSelectedDrive,
    scanState,
    result,
    progress,
    startScan,
    cancel,
  } = useDiskScan();

  const [navStack, setNavStack] = useState<TreemapNode[]>([]);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  const scanning = scanState.kind === "scanning";
  const rootNode = result?.root ?? null;
  const currentNode = navStack.length > 0 ? navStack[navStack.length - 1] : rootNode;

  // Sync selectedPath con la navegación: cuando empezamos un nuevo scan,
  // reseteamos pila y selección.
  const startingScan = useCallback(async () => {
    if (!selectedDrive) return;
    setNavStack([]);
    setSelectedPath(null);
    await startScan(selectedDrive);
  }, [selectedDrive, startScan]);

  const onTreemapNavigate = useCallback(
    (node: TreemapNode) => {
      if (node.children.length === 0) return;
      setNavStack((s) => [...s, node]);
      setSelectedPath(node.path);
    },
    [],
  );

  const onTreemapSelect = useCallback((node: TreemapNode) => {
    setSelectedPath(node.path);
  }, []);

  const onTreeZoom = useCallback((node: TreemapNode) => {
    if (node.kind !== "Dir" || node.children.length === 0) return;
    setNavStack((s) => [...s, node]);
    setSelectedPath(node.path);
  }, []);

  const onTableSelect = useCallback(
    (path: string) => {
      setSelectedPath(path);
      if (rootNode) {
        const node = findNodeByPath(rootNode, path);
        if (node && node.kind === "Dir" && node.children.length > 0) {
          // Build navStack to that path
          const stack = navStackTo(rootNode, path);
          if (stack) setNavStack(stack.slice(1));
        }
      }
    },
    [rootNode],
  );

  const handleBack = useCallback(() => {
    setNavStack((s) => s.slice(0, -1));
  }, []);

  const handleBreadcrumbClick = useCallback(
    (index: number) => {
      // 0 = root → stack vacía. n>0 → conservar primeros (n-1) elementos del stack
      // (porque el root no está en el stack).
      if (!rootNode) return;
      setNavStack((s) => s.slice(0, Math.max(0, index)));
    },
    [rootNode],
  );

  const crumbs = useMemo(() => {
    if (!rootNode || !currentNode) return [] as string[];
    return buildBreadcrumbs(rootNode.path, currentNode.path);
  }, [rootNode, currentNode]);

  // ── render ─────────────────────────────────────────────────────────────

  const hasData = !!rootNode;
  const showEmpty = !hasData && !scanning;

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <header className="flex items-center gap-2 px-4 py-3 border-b border-edge-default/10 flex-shrink-0">
        <div className="flex items-center gap-3 flex-1">
          <DriveSelector
            drives={drives}
            value={selectedDrive}
            onChange={setSelectedDrive}
            onRefresh={() => void refreshDrives()}
            loading={drivesLoading}
          />
          {scanning ? (
            <Button
              variant="destructive"
              onClick={() => void cancel()}
              className="h-9 px-3 text-xs"
            >
              <Square className="h-3 w-3 mr-1.5" fill="currentColor" />
              Cancelar
            </Button>
          ) : (
            <Button
              onClick={() => void startingScan()}
              disabled={!selectedDrive}
              className="h-9 px-3 text-xs"
            >
              <Play className="h-3 w-3 mr-1.5" fill="currentColor" />
              {hasData ? "Re-analizar" : "Analizar"}
            </Button>
          )}
        </div>

        <div className="flex items-center gap-3 text-[11px]">
          {scanning && progress && (
            <ScanLiveBadge progress={progress} />
          )}
          {scanState.kind === "error" && (
            <span className="inline-flex items-center gap-1 text-red-400">
              <AlertOctagon className="h-3 w-3" />
              {scanState.message}
            </span>
          )}
          {scanState.kind === "cancelled" && (
            <span className="text-ink-muted">Escaneo cancelado</span>
          )}
        </div>
      </header>

      {/* Breadcrumbs */}
      {hasData && (
        <div className="flex items-center gap-1 px-4 py-1.5 border-b border-edge-default/10 flex-shrink-0 overflow-x-auto">
          {navStack.length > 0 && (
            <button
              onClick={handleBack}
              className="inline-flex items-center justify-center h-6 w-6 rounded text-ink-tertiary hover:text-ink-primary hover:bg-white/[0.04]"
              title="Subir un nivel"
            >
              <ArrowLeft className="h-3 w-3" />
            </button>
          )}
          {crumbs.map((c, i) => (
            <span key={`${c}-${i}`} className="inline-flex items-center text-[11px]">
              {i > 0 && (
                <ChevronRight className="h-3 w-3 text-ink-muted mx-0.5" strokeWidth={1.5} />
              )}
              <button
                onClick={() => handleBreadcrumbClick(i)}
                className={cn(
                  "px-1.5 py-0.5 rounded font-mono",
                  i === crumbs.length - 1
                    ? "text-ink-primary"
                    : "text-ink-tertiary hover:text-ink-primary hover:bg-white/[0.04]",
                )}
              >
                {c}
              </button>
            </span>
          ))}
        </div>
      )}

      {/* Body */}
      {showEmpty ? (
        <EmptyState
          icon={HardDrive}
          title="Sin análisis de disco"
          description={
            selectedDrive
              ? `Pulsa Analizar para escanear ${selectedDrive.rootPath}.`
              : "Selecciona un disco del sistema para comenzar."
          }
          action={
            selectedDrive
              ? { label: `Analizar ${selectedDrive.rootPath}`, onClick: () => void startingScan() }
              : undefined
          }
        />
      ) : (
        <div
          className="flex-1 min-h-0 grid gap-3 p-3"
          style={{
            gridTemplateColumns: "minmax(280px, 360px) 1fr",
            gridTemplateRows: "1fr",
          }}
        >
          {/* Tree */}
          <div className="panel-raised rounded-lg border border-edge-default/10 overflow-hidden min-h-0">
            <HierarchyTree
              root={rootNode}
              selectedPath={selectedPath}
              onSelect={setSelectedPath}
              onZoom={onTreeZoom}
            />
          </div>

          {/* Main */}
          <div className="flex flex-col gap-3 min-h-0 min-w-0">
            <StatsBlocks
              report={result?.report ?? null}
              drive={selectedDrive}
              scanning={scanning}
            />

            <div className="flex-1 min-h-0 grid grid-rows-[1fr_auto] gap-3">
              <AnimatePresence mode="wait">
                {currentNode && (
                  <motion.div
                    key={currentNode.path}
                    initial={{ opacity: 0, scale: 0.99 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.15 }}
                    className="min-h-0 min-w-0"
                  >
                    <TreemapCanvas
                      root={currentNode}
                      selectedPath={selectedPath}
                      onHover={() => undefined}
                      onSelect={onTreemapSelect}
                      onNavigate={onTreemapNavigate}
                    />
                  </motion.div>
                )}
              </AnimatePresence>

              {/* Top tables */}
              {result && (
                <div className="panel-raised rounded-lg border border-edge-default/10 overflow-hidden min-h-[180px] max-h-[260px]">
                  <TopTables
                    report={result.report}
                    onSelectPath={onTableSelect}
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ── helpers ──────────────────────────────────────────────────────────────

function navStackTo(
  root: TreemapNode,
  targetPath: string,
  stack: TreemapNode[] = [],
): TreemapNode[] | null {
  const next = [...stack, root];
  if (root.path === targetPath) return next;
  for (const c of root.children) {
    if (c.kind === "Dir" && c.children.length > 0) {
      const r = navStackTo(c, targetPath, next);
      if (r) return r;
    }
  }
  return null;
}

// ── pequeño badge "live" del progreso ─────────────────────────────────────

function ScanLiveBadge({
  progress,
}: {
  progress: { bytesScanned: number; filesScanned: number; dirsScanned: number; currentPath: string; elapsedMs: number };
}) {
  return (
    <div className="inline-flex items-center gap-2 px-2.5 h-7 rounded-md bg-signal-cyan/[0.08] border border-signal-cyan/20">
      <span className="relative flex h-2 w-2">
        <span className="absolute inline-flex h-full w-full rounded-full bg-signal-cyan opacity-50 animate-ping" />
        <span className="relative inline-flex rounded-full h-2 w-2 bg-signal-cyan" />
      </span>
      <span className="font-mono tabular-nums text-ink-secondary">
        {formatBytes(progress.bytesScanned)}
      </span>
      <span className="text-ink-muted">·</span>
      <span className="font-mono tabular-nums text-ink-tertiary">
        {progress.filesScanned.toLocaleString("es-ES")} f
      </span>
      <span className="text-ink-muted">·</span>
      <span className="font-mono tabular-nums text-ink-tertiary">
        {progress.dirsScanned.toLocaleString("es-ES")} d
      </span>
      <span className="text-ink-muted text-[10px] truncate max-w-[280px]" title={progress.currentPath}>
        {progress.currentPath}
      </span>
    </div>
  );
}
