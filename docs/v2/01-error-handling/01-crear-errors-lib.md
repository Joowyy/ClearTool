# Paso 01 — Crear `src/lib/errors.ts`

**Área**: 01-error-handling
**Tiempo estimado**: 2 horas
**Dependencias**: ninguna

## Qué hacemos

Crear una utilidad centralizada para convertir cualquier `unknown` que venga de un catch en algo legible. Este será el `formatError` que reemplaza todos los `String(err)` del código.

## Por qué

El backend serializa `AppError` como `{ kind: "powershell", message: "..." }`. El frontend hace `String(obj)` → `"[object Object]"`. Necesitamos un middleware tipado.

## Archivos que tocamos

- `src/lib/errors.ts` (nuevo)
- `src/lib/errors.test.ts` (nuevo, tests unitarios)

## Cómo

### 1. Crear el archivo

```ts
// src/lib/errors.ts
import type { AppErrorPayload } from "../api";

export interface NormalizedError {
  /** Discriminator del tipo de error (`powershell`, `registry`, `permission`, etc.). */
  kind: string;
  /** Mensaje legible para mostrar al usuario. */
  message: string;
  /** Objeto crudo, para logging. */
  raw: unknown;
  /** Atajo: ¿es un error de permisos / no elevation? */
  isPermission: boolean;
  /** Atajo: ¿es un error transitorio (red, timeout) que vale la pena reintentar? */
  isRetryable: boolean;
}

const RETRYABLE_KINDS = new Set(["io", "powershell", "external"]);

export function normalizeError(err: unknown): NormalizedError {
  // 1) AppError de Tauri: { kind, message }
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
  // 2) Error standard de JS
  if (err instanceof Error) {
    return {
      kind: "javascript",
      message: err.message,
      raw: err,
      isPermission: false,
      isRetryable: false,
    };
  }
  // 3) string suelto
  if (typeof err === "string") {
    return { kind: "unknown", message: err, raw: err, isPermission: false, isRetryable: false };
  }
  // 4) fallback: serializa como JSON
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

/** Devuelve sólo el mensaje (sin kind). Uso principal: mostrar al usuario. */
export function formatError(err: unknown): string {
  return normalizeError(err).message;
}

/** Devuelve `[kind] message`. Uso: debugging, logs, banner técnico. */
export function formatErrorWithKind(err: unknown): string {
  const n = normalizeError(err);
  return `[${n.kind}] ${n.message}`;
}
```

### 2. Tests unitarios

```ts
// src/lib/errors.test.ts
import { describe, it, expect } from "vitest";
import { normalizeError, formatError } from "./errors";

describe("normalizeError", () => {
  it("normaliza AppError de Tauri", () => {
    const err = { kind: "powershell", message: "Get-AppxPackage falló" };
    const n = normalizeError(err);
    expect(n.kind).toBe("powershell");
    expect(n.message).toBe("Get-AppxPackage falló");
    expect(n.isPermission).toBe(false);
  });

  it("detecta permisos", () => {
    const err = { kind: "permission", message: "Access denied" };
    expect(normalizeError(err).isPermission).toBe(true);
  });

  it("normaliza Error standard", () => {
    const err = new Error("boom");
    expect(normalizeError(err).message).toBe("boom");
  });

  it("normaliza string", () => {
    expect(normalizeError("texto").message).toBe("texto");
  });

  it("fallback con objeto extraño", () => {
    expect(normalizeError({ foo: 1 }).message).toBe('{"foo":1}');
  });

  it("formatError nunca devuelve [object Object]", () => {
    const inputs: unknown[] = [
      { kind: "io", message: "x" },
      new Error("y"),
      "z",
      { random: true },
      null,
      undefined,
    ];
    for (const i of inputs) {
      expect(formatError(i)).not.toContain("[object Object]");
    }
  });
});
```

### 3. Instalar vitest si no está

```bash
npm i -D vitest
```

En `package.json`:
```json
"scripts": {
  "test": "vitest run",
  "test:watch": "vitest"
}
```

### 4. Ejecutar tests

```bash
npm test
```

Deberían pasar los 6 tests.

## Criterio de done

- [ ] `src/lib/errors.ts` existe con las 3 funciones públicas (`normalizeError`, `formatError`, `formatErrorWithKind`).
- [ ] `src/lib/errors.test.ts` con 6+ tests verdes.
- [ ] `formatError` NUNCA devuelve `[object Object]` con ningún input.
- [ ] Comando `npm test` corre exitosamente.
- [ ] El tipo `AppErrorPayload` se importa correctamente desde `../api` (existe ya).
