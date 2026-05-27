import { useCallback, useReducer } from "react";
import { useTauriEvent } from "../../hooks/use-tauri-event";
import type {
  CacheStartedPayload,
  CleanLogLinePayload,
  CleanProgressV2Payload,
  CleanPhasePayload,
  CleanSummaryPayload,
} from "../../api/events";

export type CleanState =
  | { kind: "idle" }
  | { kind: "running"; progress: CleanProgressV2Payload; phase: CleanPhasePayload["phase"] }
  | { kind: "cancelling"; progress: CleanProgressV2Payload; phase: CleanPhasePayload["phase"] }
  | { kind: "complete"; summary: CleanSummaryPayload }
  | { kind: "cancelled"; summary: CleanSummaryPayload }
  | { kind: "failed"; summary: CleanSummaryPayload };

interface State {
  status: CleanState;
  log: CleanLogLinePayload[];
  runId: string | null;
}

type Action =
  | { type: "started_from_backend"; payload: CacheStartedPayload }
  | { type: "start"; runId: string }
  | { type: "cancel_requested" }
  | { type: "line"; payload: CleanLogLinePayload }
  | { type: "progress"; payload: CleanProgressV2Payload }
  | { type: "phase"; payload: CleanPhasePayload }
  | { type: "summary"; payload: CleanSummaryPayload }
  | { type: "reset" };

const MAX_LOG = 500;

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "started_from_backend":
      return { status: { kind: "idle" }, log: [], runId: action.payload.runId };

    case "start":
      return { status: { kind: "idle" }, log: [], runId: action.runId };

    case "cancel_requested":
      if (state.status.kind !== "running") return state;
      return {
        ...state,
        status: { kind: "cancelling", progress: state.status.progress, phase: state.status.phase },
      };

    case "line":
      return {
        ...state,
        log:
          state.log.length >= MAX_LOG
            ? [...state.log.slice(-MAX_LOG + 1), action.payload]
            : [...state.log, action.payload],
      };

    case "progress":
      if (state.runId && action.payload.runId !== state.runId) return state;
      if (state.status.kind === "cancelling") {
        return {
          ...state,
          status: { kind: "cancelling", progress: action.payload, phase: action.payload.phase },
        };
      }
      return {
        ...state,
        status: {
          kind: "running",
          progress: action.payload,
          phase: action.payload.phase,
        },
      };

    case "phase":
      if (state.runId && action.payload.runId !== state.runId) return state;
      if (state.status.kind !== "running" && state.status.kind !== "cancelling") return state;
      return {
        ...state,
        status: { ...state.status, phase: action.payload.phase },
      };

    case "summary":
      if (state.runId && action.payload.runId !== state.runId) return state;
      return {
        ...state,
        status: {
          kind: action.payload.cancelled
            ? "cancelled"
            : action.payload.success
              ? "complete"
              : "failed",
          summary: action.payload,
        },
      };

    case "reset":
      return { status: { kind: "idle" }, log: [], runId: null };
  }
}

export function useCleanStream() {
  const [state, dispatch] = useReducer(reducer, {
    status: { kind: "idle" },
    log: [],
    runId: null,
  });

  useTauriEvent("cache:started", (p) => dispatch({ type: "started_from_backend", payload: p }));
  useTauriEvent("cache:line", (p) => dispatch({ type: "line", payload: p }));
  useTauriEvent("cache:progress-v2", (p) => dispatch({ type: "progress", payload: p }));
  useTauriEvent("cache:phase", (p) => dispatch({ type: "phase", payload: p }));
  useTauriEvent("cache:summary", (p) => dispatch({ type: "summary", payload: p }));

  const start = useCallback((runId: string) => dispatch({ type: "start", runId }), []);
  const cancelRequested = useCallback(() => dispatch({ type: "cancel_requested" }), []);
  const reset = useCallback(() => dispatch({ type: "reset" }), []);

  return { ...state, start, cancelRequested, reset };
}
