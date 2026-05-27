// Catálogo central de eventos Tauri tipados.
//
// Mantener sincronizado con los `app.emit("...")` del backend
// (ver src-tauri/src/ipc/*.rs). El payload de cada evento se declara en
// `EventPayloads` para que `useTauriEvent` pueda inferirlo.

import type { TreeNode } from "./types";

export interface CacheDebugPayload {
  level: string;
  location: string;
  message: string;
  bytesFreed: number;
  filesDeleted: number;
}

// ── Doc 09: pipeline de eventos enriquecidos ─────────────────────────────

export type CleanPhase =
  | "preparing"
  | "creatingRestorePoint"
  | "closingProcesses"
  | "cleaning"
  | "schedulingReboot"
  | "verifying"
  | "complete"
  | "failed";

export interface CleanLogLinePayload {
  level: string;
  location: string;
  message: string;
  timestampMs: number;
}

export interface CleanProgressV2Payload {
  runId: string;
  phase: CleanPhase;
  currentLocationId: string | null;
  currentLocationDisplayName: string | null;
  currentLocationIndex: number;
  totalLocations: number;
  bytesFreed: number;
  bytesScheduled: number;
  totalEstimatedBytes: number;
  filesDeleted: number;
  filesScheduled: number;
  filesFailed: number;
  elapsedMs: number;
  etaSecs: number | null;
  /** true = ETA de ventana móvil (fiable), false = acumulado/fallback */
  etaIsPrecise: boolean;
  throughputBytesPerSec: number;
  lastLine: CleanLogLinePayload | null;
}

export interface CleanPhasePayload {
  runId: string;
  phase: CleanPhase;
  elapsedMs: number;
}

export interface ResidualEntry {
  path: string;
  bytes: number;
  reason: ResidualReason;
  sampleFiles: string[];
}

export type ResidualReason =
  | { kind: "lockedBySystem"; suggestedAction: string }
  | { kind: "pendingReboot" }
  | { kind: "accessDenied" }
  | { kind: "filteredOut" }
  | { kind: "reparsePoint" }
  | { kind: "unknown" };

export interface CleanSummaryPayload {
  runId: string;
  success: boolean;
  cancelled: boolean;
  durationMs: number;
  totalBytesFreed: number;
  totalBytesScheduledReboot: number;
  totalFilesDeleted: number;
  totalFilesScheduledReboot: number;
  totalFilesFailed: number;
  locationsProcessed: number;
  locationsWithErrors: string[];
  restorePointSeq: number | null;
  meanThroughputBytesPerSec: number;
  report: unknown;
}

export interface CacheStartedPayload {
  runId: string;
}

export const TauriEvents = {
  ExplorerNode: "explorer:node",
  ExplorerDone: "explorer:done",
  CacheDebug: "cache:debug",
  CacheLine: "cache:line",
  CacheProgressV2: "cache:progress-v2",
  CachePhase: "cache:phase",
  CacheSummary: "cache:summary",
  CacheStarted: "cache:started",
} as const;

export type TauriEventName = (typeof TauriEvents)[keyof typeof TauriEvents];

export interface EventPayloads {
  "explorer:node": TreeNode;
  "explorer:done": string;
  "cache:debug": CacheDebugPayload;
  "cache:line": CleanLogLinePayload;
  "cache:progress-v2": CleanProgressV2Payload;
  "cache:phase": CleanPhasePayload;
  "cache:summary": CleanSummaryPayload;
  "cache:started": CacheStartedPayload;
}
