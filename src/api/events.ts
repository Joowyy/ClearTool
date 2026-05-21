// Catálogo central de eventos Tauri tipados.
//
// Mantener este enum sincronizado con los `app.emit("...")` del backend
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

export const TauriEvents = {
  ExplorerNode: "explorer:node",
  ExplorerDone: "explorer:done",
  CacheDebug: "cache:debug",
} as const;

export type TauriEventName = (typeof TauriEvents)[keyof typeof TauriEvents] | "cache:debug";

export interface EventPayloads {
  "explorer:node": TreeNode;
  "explorer:done": string;
  "cache:debug": CacheDebugPayload;
}
