---
name: error-model
description: Modelo de errores end-to-end de ClearTool — AppError tipado en Rust (thiserror + serde tag/content) hasta normalizeError/formatError, toasts (sonner) y ErrorBoundary en React. Uso obligatorio al crear o tocar cualquier comando Tauri que pueda fallar, cualquier catch en el frontend, o al añadir una variante de error nueva. Garantiza que ningún error llegue al usuario como "[object Object]" y que los errores accionables (permiso, archivo bloqueado) ofrezcan una acción.
---

# Skill: error-model

Contrato de errores que cruza Rust → IPC → React. Es la base de M1 ("el UX no sangra"): cualquier módulo nuevo hereda este patrón. Si lo rompes, vuelve a aparecer `[object Object]`.

## Cuándo usar este skill

- Creas o modificas un comando Tauri que devuelve `Result<_, AppError>`.
- Añades una variante a `AppError` (Rust) o un `kind` nuevo en el frontend.
- Escribes un `catch (err)` o un `onError` de TanStack Query.
- Implementas un mensaje de error accionable (botón "Reiniciar como admin", "Cerrar Spotify", etc.).

## Cuándo NO usar

- Logging puramente interno de Rust (usa `tracing`/`log`, no `AppError`).

## Regla de oro

**Nunca** `String(err)`, `${err}`, ni `err.toString()` en JSX o en mensajes de usuario. Siempre pasa por `formatError(err)` / `normalizeError(err)`.

## Lado Rust — `src-tauri/src/core/error.rs`

Enum único, serializado con `tag = "kind"` y `content = "message"` para que el frontend lo discrimine:

```rust
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "message", rename_all = "kebab-case")]
pub enum AppError {
    #[error("io: {0}")]            Io(String),
    #[error("registry: {0}")]     Registry(String),
    #[error("powershell: {0}")]   Powershell(String),
    #[error("permission: {0}")]   Permission(String),
    #[error("not elevated: {0}")] NotElevated(String),
    #[error("not found: {0}")]    NotFound(String),
    #[error("file locked: {0}")]  FileLocked(String),   // archivo en uso por un proceso
    #[error("process: {0}")]      Process(String),       // process-not-found, access-denied-process
    #[error("cancelled")]         Cancelled,
    #[error("not implemented")]   NotImplemented,
}

pub type AppResult<T> = Result<T, AppError>;
```

### Wrapping con contexto (no perder el inner)

```rust
impl AppError {
    pub fn with_context<C: std::fmt::Display>(self, ctx: C) -> Self {
        match self {
            AppError::Io(m)         => AppError::Io(format!("{ctx}: {m}")),
            AppError::Powershell(m) => AppError::Powershell(format!("{ctx}: {m}")),
            AppError::Registry(m)   => AppError::Registry(format!("{ctx}: {m}")),
            other => other, // kinds sin payload se mantienen
        }
    }
}
```

Uso idiomático en backend (siempre añade qué operación se intentaba):

```rust
fs::remove_dir_all(&path)
    .map_err(|e| AppError::Io(e.to_string()).with_context(format!("eliminando {}", path.display())))?;
```

### Convención de mensajes

- El `message` es **legible por un humano no técnico** y, cuando aplica, sugiere la causa.
- Para errores accionables usa el `kind` correcto (`permission`, `not-elevated`, `file-locked`, `process`) porque el frontend mapea el `kind` a una acción.
- Mal: `"file in use"`. Bien: `AppError::FileLocked("Spotify tiene abierto este archivo".into())`.

## Lado TypeScript — `src/lib/errors.ts`

```ts
import type { AppErrorPayload } from "@/bindings"; // generado por ts-rs

export interface NormalizedError {
  kind: string;          // "powershell" | "registry" | "permission" | "file-locked" | "process" | "unknown" ...
  message: string;
  raw: unknown;
  isPermission: boolean; // permission || not-elevated → banner "requiere admin"
  isLocked: boolean;     // file-locked → ofrecer "cerrar proceso"
}

export function normalizeError(err: unknown): NormalizedError {
  if (typeof err === "object" && err !== null && "kind" in err && "message" in err) {
    const p = err as AppErrorPayload;
    return {
      kind: p.kind,
      message: p.message,
      raw: err,
      isPermission: p.kind === "permission" || p.kind === "not-elevated",
      isLocked: p.kind === "file-locked",
    };
  }
  if (err instanceof Error)    return { kind: "javascript", message: err.message, raw: err, isPermission: false, isLocked: false };
  if (typeof err === "string") return { kind: "unknown", message: err, raw: err, isPermission: false, isLocked: false };
  try   { return { kind: "unknown", message: JSON.stringify(err), raw: err, isPermission: false, isLocked: false }; }
  catch { return { kind: "unknown", message: "Error desconocido", raw: err, isPermission: false, isLocked: false }; }
}

export const formatError = (err: unknown) => normalizeError(err).message;
export const formatErrorWithKind = (err: unknown) => {
  const n = normalizeError(err);
  return `[${n.kind}] ${n.message}`;
};
```

## Toasts accionables — `src/lib/toast.ts` (wrapper de sonner)

```ts
import { toast as sonner } from "sonner";
import { normalizeError } from "./errors";

export const toast = {
  error: (msg: string, err?: unknown) => {
    if (!err) return void sonner.error(msg);
    const n = normalizeError(err);
    sonner.error(msg, {
      description: n.message,
      action: n.isPermission
        ? { label: "Reiniciar como admin", onClick: () => relaunchElevated() }
        : undefined,
    });
  },
  success: (m: string) => sonner.success(m),
  info:    (m: string) => sonner.info(m),
  warning: (m: string, opts?: Parameters<typeof sonner.warning>[1]) => sonner.warning(m, opts),
};
```

`<Toaster theme="dark" position="bottom-right" richColors />` va una sola vez dentro de `<AppShell>`.

## ErrorBoundary de router — `src/components/error-boundary.tsx`

Registrar como `errorElement` en el router para capturar crashes de render (evita el "Hey developer 👋" de React Router):

```tsx
import { useRouteError } from "react-router-dom";
import { formatError } from "@/lib/errors";

export function ErrorBoundary() {
  const err = useRouteError();
  return (
    <div className="p-8 flex flex-col items-center gap-4 max-w-2xl mx-auto">
      <h2 className="text-2xl font-bold">Algo se rompió en la UI</h2>
      <p className="text-muted-foreground">{formatError(err)}</p>
      <details className="text-xs">
        <summary>Detalles técnicos</summary>
        <pre className="mt-2 p-3 bg-card border rounded font-mono">{JSON.stringify(err, null, 2)}</pre>
      </details>
      <button onClick={() => window.location.reload()}>Reiniciar la app</button>
    </div>
  );
}
```

## Patrón de uso en una página

```ts
removeMutation.mutate(args, {
  onError: (err) => toast.error("No se pudieron eliminar los paquetes", err),
});
```

Para el caso `file-locked`, el caller usa `normalizeError(err).isLocked` para ofrecer "Cerrar <proceso>" (ver skill `win32-process-manager`).

## Checklist al añadir un error nuevo

1. ¿Existe ya un `kind` adecuado? Reutilízalo; no inventes uno por capricho.
2. Si es accionable, el `kind` debe permitir al frontend decidir la acción (no lo metas en un `Io` genérico).
3. Mensaje legible y orientado a causa/solución.
4. Añade el nuevo `kind` al union de TypeScript si tocaste el enum Rust, y regenera bindings (`ts-rs`).

## Tests obligatorios

- `errors.test.ts`: `normalizeError` cubre las 4 ramas (AppError, Error, string, fallback) y marca bien `isPermission`/`isLocked`.
- Rust: un test por comando que fuerza el error y confirma el `kind` serializado correcto.
- Regla CI/lint: grep que falla si aparece `String(` o `${` aplicado a `err`/`error` dentro de JSX (`src/**/*.tsx`).

## Done (criterio M1)

- Ningún error muestra `[object Object]`.
- `sonner` integrado en AppShell; `ErrorBoundary` registrado en el router.
- Los 3 errores históricos (detect debloat, list restore, cache clean) tienen mensaje específico + acción.
