// use-disk-scan — orquesta el escaneo del disco con streaming de progreso.
//
// Mantiene en estado:
//   - El drive elegido y la lista de drives disponibles.
//   - El estado del escaneo (idle / scanning / completed / cancelled / error).
//   - El reporte y la raíz del treemap cuando el backend responde.
//   - Los contadores en vivo emitidos por `disk:progress`.
//
// La UI se suscribe leyendo del hook directamente; no es un Zustand store
// porque el ámbito vive solo dentro de la página.
import { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  buildTreemapData,
  cancelDiskScan,
  listDrives,
} from "../../api/client";
import type {
  DiskAnalysisResult,
  DiskScanProgressPayload,
  DriveListing,
} from "../../api/types";

export type ScanState =
  | { kind: "idle" }
  | { kind: "scanning"; scanId: string; startedAt: number }
  | { kind: "completed"; scanId: string }
  | { kind: "cancelled"; scanId: string }
  | { kind: "error"; scanId: string; message: string };

interface UseDiskScanReturn {
  drives: DriveListing[];
  drivesLoading: boolean;
  refreshDrives: () => Promise<void>;
  selectedDrive: DriveListing | null;
  setSelectedDrive: (d: DriveListing | null) => void;
  scanState: ScanState;
  result: DiskAnalysisResult | null;
  progress: DiskScanProgressPayload | null;
  startScan: (drive: DriveListing) => Promise<void>;
  cancel: () => Promise<void>;
}

const LAST_DRIVE_KEY = "cleartool.diskAnalyzer.lastDrive";

function newScanId(): string {
  try {
    if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
      return crypto.randomUUID();
    }
  } catch {
    // ignore
  }
  return `scan-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export function useDiskScan(): UseDiskScanReturn {
  const [drives, setDrives] = useState<DriveListing[]>([]);
  const [drivesLoading, setDrivesLoading] = useState(false);
  const [selectedDrive, setSelectedDriveState] = useState<DriveListing | null>(null);
  const [scanState, setScanState] = useState<ScanState>({ kind: "idle" });
  const [result, setResult] = useState<DiskAnalysisResult | null>(null);
  const [progress, setProgress] = useState<DiskScanProgressPayload | null>(null);
  const activeScanIdRef = useRef<string | null>(null);

  const refreshDrives = useCallback(async () => {
    setDrivesLoading(true);
    try {
      const list = await listDrives();
      setDrives(list);

      // Restaurar selección persistida cuando aún no se eligió ningún disco.
      setSelectedDriveState((current) => {
        if (current) {
          const refreshed = list.find((d) => d.rootPath === current.rootPath);
          return refreshed ?? null;
        }
        const persisted = localStorage.getItem(LAST_DRIVE_KEY);
        const ready = list.filter((d) => d.isReady);
        if (persisted) {
          const found = ready.find((d) => d.rootPath === persisted);
          if (found) return found;
        }
        return ready[0] ?? null;
      });
    } catch (err) {
      console.error("Failed to list drives", err);
    } finally {
      setDrivesLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshDrives();
  }, [refreshDrives]);

  const setSelectedDrive = useCallback((d: DriveListing | null) => {
    setSelectedDriveState(d);
    if (d) {
      try {
        localStorage.setItem(LAST_DRIVE_KEY, d.rootPath);
      } catch {
        // ignore
      }
    }
  }, []);

  // Suscribirse a eventos de progreso/finalización del scan activo.
  useEffect(() => {
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenComplete: UnlistenFn | undefined;
    let unlistenError: UnlistenFn | undefined;
    let cancelled = false;

    void (async () => {
      try {
        unlistenProgress = await listen<DiskScanProgressPayload>(
          "disk:progress",
          (e) => {
            if (
              activeScanIdRef.current &&
              e.payload.scanId === activeScanIdRef.current
            ) {
              setProgress(e.payload);
            }
          },
        );
        unlistenComplete = await listen<{ scanId: string }>(
          "disk:complete",
          (e) => {
            if (
              activeScanIdRef.current &&
              e.payload.scanId === activeScanIdRef.current
            ) {
              setScanState({ kind: "completed", scanId: e.payload.scanId });
            }
          },
        );
        unlistenError = await listen<{ scanId: string; error: string }>(
          "disk:error",
          (e) => {
            if (
              activeScanIdRef.current &&
              e.payload.scanId === activeScanIdRef.current
            ) {
              if (e.payload.error === "cancelled") {
                setScanState({
                  kind: "cancelled",
                  scanId: e.payload.scanId,
                });
              } else {
                setScanState({
                  kind: "error",
                  scanId: e.payload.scanId,
                  message: e.payload.error,
                });
              }
            }
          },
        );
      } catch (err) {
        if (!cancelled) console.warn("Disk listeners unavailable", err);
      }
    })();

    return () => {
      cancelled = true;
      unlistenProgress?.();
      unlistenComplete?.();
      unlistenError?.();
    };
  }, []);

  const startScan = useCallback(async (drive: DriveListing) => {
    const scanId = newScanId();
    activeScanIdRef.current = scanId;
    setProgress(null);
    setResult(null);
    setScanState({ kind: "scanning", scanId, startedAt: Date.now() });

    try {
      const r = await buildTreemapData({
        root: drive.rootPath,
        scanId,
        maxDepthEmit: 6,
        minSizeMb: 10,
        followReparsePoints: false,
        includeHidden: true,
        includeSystem: true,
      });
      // Sólo aceptar resultados del scan activo (evita race con cancel + start rápido).
      if (activeScanIdRef.current === scanId) {
        setResult(r);
        setScanState({ kind: "completed", scanId });
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      if (activeScanIdRef.current === scanId) {
        if (msg.includes("cancelled") || (typeof err === "object" && err && "kind" in err && (err as { kind: unknown }).kind === "cancelled")) {
          setScanState({ kind: "cancelled", scanId });
        } else {
          setScanState({ kind: "error", scanId, message: msg });
        }
      }
    }
  }, []);

  const cancel = useCallback(async () => {
    if (scanState.kind !== "scanning") return;
    try {
      await cancelDiskScan(scanState.scanId);
    } catch (err) {
      console.warn("cancel_disk_scan failed", err);
    }
  }, [scanState]);

  return {
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
  };
}
