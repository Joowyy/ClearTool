# v2 · 01 — Error Model Fix (P0)

## El bug que mata la UX

En las 3 pantallas críticas (Debloat, Restauración, Auditoría) el usuario ve:

> `Error al detectar paquetes: [object Object]`
> `Error al cargar puntos de restauración: [object Object]`

Esto NO es un error de una operación específica. Es un fallo del **modelo de errores cross-cutting**. Hasta que esto no se arregle, el resto de mejoras son inútiles: cualquier nueva feature heredará el mismo problema.

## Causa raíz

### Lado Rust
`AppError` se serializa así (en `src-tauri/src/core/error.rs`):

```rust
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "message", rename_all = "kebab-case")]
pub enum AppError {
    Io(String),
    Registry(String),
    Powershell(String),
    Permission(String),
    NotElevated(String),
    // ...
}
```

Cuando Tauri devuelve `Err(e)` desde un comando, lo serializa como:

```json
{ "kind": "powershell", "message": "Get-ComputerRestorePoint: Access denied" }
```

### Lado TypeScript
El código actual hace:

```ts
catch (err) {
  setError(String(err));  // ← BUG: serializa el objeto a "[object Object]"
}
```

O en el ejemplo del audit page:

```tsx
Error: {String(removeMutation.error)}
```

`String({kind:..., message:...})` produce literalmente `"[object Object]"`. Eso es lo que se ve.

## El fix tiene dos partes

### Parte A — `src/lib/errors.ts` (NUEVO)

Una utilidad centralizada que convierte cualquier `unknown` que venga del catch en algo legible:

```ts
// src/lib/errors.ts
import type { AppErrorPayload } from "../api";

export interface NormalizedError {
  kind: string;          // "powershell", "registry", "permission", "unknown"
  message: string;       // texto legible
  raw: unknown;          // por si el caller quiere logging completo
  isPermission: boolean; // shortcut útil para mostrar el banner "requiere admin"
}

export function normalizeError(err: unknown): NormalizedError {
  // 1) AppError de Tauri: { kind, message }
  if (typeof err === "object" && err !== null && "kind" in err && "message" in err) {
    const p = err as AppErrorPayload;
    return {
      kind: p.kind,
      message: p.message,
      raw: err,
      isPermission: p.kind === "permission" || p.kind === "not-elevated",
    };
  }
  // 2) Error standard de JS
  if (err instanceof Error) {
    return { kind: "javascript", message: err.message, raw: err, isPermission: false };
  }
  // 3) string suelto
  if (typeof err === "string") {
    return { kind: "unknown", message: err, raw: err, isPermission: false };
  }
  // 4) fallback: serializa como JSON para que al menos sea legible
  try {
    return {
      kind: "unknown",
      message: JSON.stringify(err),
      raw: err,
      isPermission: false,
    };
  } catch {
    return { kind: "unknown", message: "Error desconocido", raw: err, isPermission: false };
  }
}

export function formatError(err: unknown): string {
  const n = normalizeError(err);
  // Para mostrar al usuario, sin el kind técnico:
  return n.message;
}

export function formatErrorWithKind(err: unknown): string {
  const n = normalizeError(err);
  return `[${n.kind}] ${n.message}`;
}
```

### Parte B — Reemplazar `String(err)` en todas las páginas

Sed/regex en el codebase:

```
String(removeMutation.error)            → formatError(removeMutation.error)
String(error)                           → formatError(error)
{String(...)}  (en JSX de errores)      → {formatError(...)}
```

Archivos a tocar:
- `src/features/audit-log/audit-page.tsx`
- `src/features/restore-points/restore-page.tsx`
- `src/features/debloat/debloat-page.tsx`
- `src/features/cache-cleaner/cache-page.tsx`
- `src/features/registry-tweaks/registry-page.tsx`
- `src/features/services/services-page.tsx`
- Cualquier `catch (err)` en hooks.

### Parte C — ErrorBoundary global (router level)

React Router v6 muestra el "Hey developer 👋" feo cuando un componente tira. Añadir un `errorElement` en el router:

```tsx
// src/App.tsx o src/router.tsx
import { ErrorBoundary } from "./components/error-boundary";

const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    errorElement: <ErrorBoundary />, // ← captura crashes de render
    children: [...]
  }
]);
```

Y el componente:

```tsx
// src/components/error-boundary.tsx
import { useRouteError } from "react-router-dom";
import { formatError } from "../lib/errors";
import { AlertOctagon } from "lucide-react";
import { Button } from "./ui/button";

export function ErrorBoundary() {
  const err = useRouteError();
  const msg = formatError(err);
  return (
    <div className="p-8 flex flex-col items-center justify-center gap-4 max-w-2xl mx-auto">
      <AlertOctagon className="h-12 w-12 text-signal-red" />
      <h2 className="text-2xl font-bold">Algo se rompió en la UI</h2>
      <p className="text-muted-foreground text-center">{msg}</p>
      <details className="text-xs text-muted-foreground">
        <summary>Detalles técnicos</summary>
        <pre className="mt-2 p-3 bg-card border border-border rounded font-mono">
          {JSON.stringify(err, null, 2)}
        </pre>
      </details>
      <Button onClick={() => window.location.reload()}>Reiniciar la app</Button>
    </div>
  );
}
```

### Parte D — Toast system para errores transitorios

Errores que no rompen render (operación que falla en backend) deberían mostrarse como toast en lugar de inline en cada página:

```tsx
// src/lib/toast.ts — wrapper sobre sonner
import { toast as sonner } from "sonner";
import { formatError, normalizeError } from "./errors";

export const toast = {
  error: (msg: string, err?: unknown) => {
    if (err) {
      const n = normalizeError(err);
      sonner.error(msg, {
        description: n.message,
        action: n.isPermission ? { label: "Reiniciar como admin", onClick: () => /* ... */ } : undefined,
      });
    } else {
      sonner.error(msg);
    }
  },
  success: (msg: string) => sonner.success(msg),
  info: (msg: string) => sonner.info(msg),
  warning: (msg: string) => sonner.warning(msg),
};
```

Uso:

```ts
removeMutation.mutate(args, {
  onError: (err) => toast.error("No se pudieron eliminar los paquetes", err),
});
```

Añade `sonner` como dep: `npm i sonner`. Ponerlo en `<AppShell>`:

```tsx
import { Toaster } from "sonner";
// ...
<AppShell>
  <Outlet />
  <Toaster theme="dark" position="bottom-right" richColors />
</AppShell>
```

## Estandarización de mensajes Rust

El Rust hoy mete a veces strings sin contexto (`"file in use"`) y a veces contexto completo (`"Access denied. (os error 5)"`). Estandarizar:

```rust
// src-tauri/src/core/error.rs — añadir helpers
impl AppError {
    /// Wrapping idiomático: añade contexto sin perder el inner.
    pub fn with_context<C: std::fmt::Display>(self, ctx: C) -> Self {
        match self {
            AppError::Io(msg) => AppError::Io(format!("{}: {}", ctx, msg)),
            AppError::Powershell(msg) => AppError::Powershell(format!("{}: {}", ctx, msg)),
            // ...
            other => other, // los kinds tipados que ya tienen estructura
        }
    }
}
```

Uso en backend:

```rust
fs::remove_dir_all(&path)
    .map_err(|e| AppError::Io(e.to_string()).with_context(format!("eliminando {}", path.display())))?;
```

## Mensajes específicos para los 3 errores del v0.1

Cuando se implemente esto, los errores actuales se ven así:

### Debloat detect
**Antes**: `Error al detectar paquetes: [object Object]`
**Después**: `Error al detectar paquetes: [powershell] Get-AppxPackage falló: el host de PowerShell no responde`

Con acción sugerida en el toast: "Reinstalar PowerShell" o "Reintentar".

### Restore points list
**Antes**: `Error al cargar puntos de restauración: [object Object]`
**Después**: `[powershell] Get-ComputerRestorePoint no devolvió resultados. Verifica que System Protection esté habilitado en C:\.`

Con acción sugerida: "Habilitar System Protection" (botón que llama a `enable_system_protection`).

### Cache clean — archivo en uso
**Antes**: `Error: file in use, could not schedule for reboot deletion`
**Después**: `Spotify tiene abierto este archivo. Cierra Spotify o reinicia el PC.`

Con acción en el toast: "Cerrar Spotify" (llama a `process_manager::kill_by_name`).

## Criterio de "done"

- [ ] `src/lib/errors.ts` existe con tests unitarios.
- [ ] `String(err)` no aparece en JSX de ninguna página.
- [ ] `sonner` integrado en AppShell.
- [ ] `ErrorBoundary` registrado en el router.
- [ ] El JSON `AppError` de cualquier comando IPC se renderiza legible en pantalla.
- [ ] Los 3 errores actuales (`detect`, `list restore`, `cache clean`) tienen mensajes específicos y accionables.

## Dependencias con otros specs
- `02-cache-engine-rewrite.md` necesita este error model para diferenciar "archivo bloqueado por proceso X" de "permiso denegado".
- `03-process-manager.md` produce errores tipados específicos (`process-not-found`, `access-denied-process`).
- `06-ui-ux-refactor.md` describe el sistema de toasts en más detalle.
