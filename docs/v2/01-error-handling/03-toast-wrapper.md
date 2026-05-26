# Paso 03 — Crear wrapper `src/lib/toast.ts`

**Área**: 01-error-handling
**Tiempo estimado**: 1 hora
**Dependencias**: Paso 01 (`errors.ts`) + Paso 02 (`sonner`)

## Qué hacemos

Wrapper sobre `sonner.toast` que entiende el error model. En vez de hacer `toast.error("Algo: " + formatError(err))` por todas partes, hacemos `toast.error("Algo", err)` y el wrapper formatea.

## Por qué

DRY: cada llamada actual a `toast.error` tendría que importar `formatError` y construir el string. El wrapper centraliza el patrón.

## Archivos que tocamos

- `src/lib/toast.ts` (nuevo)

## Cómo

```ts
// src/lib/toast.ts
import { toast as sonner } from "sonner";
import { normalizeError } from "./errors";

interface ToastErrorOptions {
  /** Acción opcional con label + handler. */
  action?: { label: string; onClick: () => void };
  /** Forzar duración (ms). Default: 8s para errores. */
  duration?: number;
  /** Si la acción debe destacar como "primary" (azul) o "destructive" (rojo). */
  actionVariant?: "primary" | "destructive";
}

export const toast = {
  /**
   * Toast de éxito. Auto-dismiss 4s.
   * Ejemplo: `toast.success("Restore point creado")`.
   */
  success(message: string, opts?: { description?: string; duration?: number }) {
    sonner.success(message, {
      description: opts?.description,
      duration: opts?.duration ?? 4000,
    });
  },

  /**
   * Toast de info. Auto-dismiss 5s.
   */
  info(message: string, opts?: { description?: string }) {
    sonner.info(message, { description: opts?.description, duration: 5000 });
  },

  /**
   * Toast de warning. Auto-dismiss 6s.
   */
  warning(message: string, opts?: { description?: string; action?: ToastErrorOptions["action"] }) {
    sonner.warning(message, {
      description: opts?.description,
      duration: 6000,
      action: opts?.action,
    });
  },

  /**
   * Toast de error con normalización automática.
   *
   * @param message Título principal (ej. "No se pudo limpiar caché").
   * @param err Cualquier error: AppError, Error, string, etc. (opcional).
   * @param opts Opciones extra.
   *
   * @example
   * try { await removeBloatware(...); }
   * catch (err) { toast.error("Falló la desinstalación", err); }
   */
  error(message: string, err?: unknown, opts?: ToastErrorOptions) {
    if (err === undefined) {
      sonner.error(message, { duration: opts?.duration ?? 8000, action: opts?.action });
      return;
    }
    const n = normalizeError(err);
    // Si es permission y no hay action custom, sugerimos relanzar como admin
    const action = opts?.action ?? (n.isPermission
      ? {
          label: "Reiniciar como admin",
          onClick: () => {
            // se importa lazy para no crear ciclo con el api client
            void import("../api").then(m => m.relaunchAsAdmin?.());
          },
        }
      : undefined);

    sonner.error(message, {
      description: n.message,
      duration: opts?.duration ?? 8000,
      action,
    });
  },

  /**
   * Toast con loading state que se resuelve a success o error.
   * Para operaciones async largas.
   *
   * @example
   * toast.promise(removeBloatware(args), {
   *   loading: "Eliminando paquetes...",
   *   success: "Paquetes eliminados",
   *   error: "Error al eliminar",
   * });
   */
  promise<T>(
    promise: Promise<T>,
    msgs: { loading: string; success: string | ((data: T) => string); error: string }
  ) {
    return sonner.promise(promise, {
      loading: msgs.loading,
      success: msgs.success,
      error: (err) => `${msgs.error}: ${normalizeError(err).message}`,
    });
  },

  /**
   * Descartar todos los toasts visibles.
   */
  dismiss() {
    sonner.dismiss();
  },
};
```

## Uso esperado en páginas

Antes:
```tsx
{removeMutation.isError && (
  <div className="p-3 bg-red-900/30 ...">
    Error: {String(removeMutation.error)}
  </div>
)}
```

Después:
```tsx
const removeMutation = useMutation({
  mutationFn: ...,
  onError: (err) => toast.error("No se pudieron eliminar los paquetes", err),
  onSuccess: () => toast.success("Paquetes eliminados"),
});
```

Y se elimina el banner inline del JSX.

## Criterio de done

- [ ] `src/lib/toast.ts` existe con `toast.success`, `toast.info`, `toast.warning`, `toast.error`, `toast.promise`, `toast.dismiss`.
- [ ] `toast.error` con un `AppError` muestra `[powershell] mensaje` en description.
- [ ] `toast.error` con error de permission incluye action button "Reiniciar como admin".
- [ ] `toast.promise` funciona con una promesa real (test rápido con un `setTimeout`).
- [ ] La importación de `toast` desde `src/lib/toast.ts` funciona sin warnings TS.
