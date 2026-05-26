import type { AppErrorPayload } from "../api";

export interface NormalizedError {
  kind: string;
  message: string;
  raw: unknown;
  isPermission: boolean;
  isRetryable: boolean;
}

const RETRYABLE_KINDS = new Set(["io", "powershell", "external"]);

export function normalizeError(err: unknown): NormalizedError {
  if (err === null || err === undefined) {
    return {
      kind: "unknown",
      message: "Error desconocido",
      raw: err,
      isPermission: false,
      isRetryable: false,
    };
  }

  if (
    typeof err === "object" &&
    err !== null &&
    "kind" in err &&
    "message" in err &&
    typeof (err as Record<string, unknown>).message === "string"
  ) {
    const p = err as AppErrorPayload;
    return {
      kind: p.kind,
      message: p.message,
      raw: err,
      isPermission: p.kind === "permission" || p.kind === "not-elevated",
      isRetryable: RETRYABLE_KINDS.has(p.kind),
    };
  }

  if (err instanceof Error) {
    return {
      kind: "javascript",
      message: err.message,
      raw: err,
      isPermission: false,
      isRetryable: false,
    };
  }

  if (typeof err === "string") {
    return { kind: "unknown", message: err, raw: err, isPermission: false, isRetryable: false };
  }

  try {
    return {
      kind: "unknown",
      message: JSON.stringify(err),
      raw: err,
      isPermission: false,
      isRetryable: false,
    };
  } catch {
    return {
      kind: "unknown",
      message: "Error desconocido",
      raw: err,
      isPermission: false,
      isRetryable: false,
    };
  }
}

export function formatError(err: unknown): string {
  return normalizeError(err).message;
}

export function formatErrorWithKind(err: unknown): string {
  const n = normalizeError(err);
  return `[${n.kind}] ${n.message}`;
}
