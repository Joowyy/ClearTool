# Paso 04 — ErrorBoundary global en el router

**Área**: 01-error-handling
**Tiempo estimado**: 1.5 horas
**Dependencias**: Paso 01 (`errors.ts`)

## Qué hacemos

Reemplazar la pantalla "Hey developer 👋" de React Router cuando un componente tira, con un ErrorBoundary serio que use nuestro `formatError` y ofrezca recovery.

## Por qué

Hoy, si cualquier render falla (como pasó con el bug del audit log antes del fix), el usuario ve un mensaje técnico inútil. Con un ErrorBoundary podemos: explicar qué pasó, dar opción de reload, mostrar stack en details, ofrecer copiar logs.

## Archivos que tocamos

- `src/components/error-boundary.tsx` (nuevo)
- `src/main.tsx` o donde esté definido el router (modificado)

## Cómo

### 1. Crear el componente

```tsx
// src/components/error-boundary.tsx
import { useRouteError, isRouteErrorResponse } from "react-router-dom";
import { AlertOctagon, RefreshCw, Copy } from "lucide-react";
import { Button } from "./ui/button";
import { formatError, normalizeError } from "../lib/errors";
import { toast } from "../lib/toast";

export function ErrorBoundary() {
  const err = useRouteError();
  const n = normalizeError(err);
  const isRouteError = isRouteErrorResponse(err);

  const handleReload = () => window.location.reload();

  const handleCopy = async () => {
    const text = JSON.stringify(
      {
        kind: n.kind,
        message: n.message,
        raw: typeof err === "object" ? err : { value: err },
        userAgent: navigator.userAgent,
        timestamp: new Date().toISOString(),
        url: window.location.href,
      },
      null,
      2
    );
    try {
      await navigator.clipboard.writeText(text);
      toast.success("Detalles copiados al portapapeles");
    } catch {
      toast.error("No se pudo copiar al portapapeles");
    }
  };

  return (
    <div className="h-screen w-screen flex items-center justify-center bg-surface-canvas text-ink-primary p-8">
      <div className="max-w-2xl flex flex-col gap-4 items-center">
        <AlertOctagon className="h-16 w-16 text-signal-red" strokeWidth={1.5} />

        <h1 className="text-2xl font-bold">
          {isRouteError ? "Página no encontrada" : "Algo se rompió en la UI"}
        </h1>

        <p className="text-muted-foreground text-center">
          {isRouteError
            ? `La ruta "${window.location.pathname}" no existe en esta versión.`
            : formatError(err)}
        </p>

        <details className="w-full mt-4">
          <summary className="cursor-pointer text-sm text-muted-foreground hover:text-foreground">
            Ver detalles técnicos
          </summary>
          <pre className="mt-2 p-4 bg-card border border-border rounded-lg font-mono text-xs overflow-auto max-h-64">
            {JSON.stringify(err, null, 2)}
          </pre>
        </details>

        <div className="flex gap-2 mt-4">
          <Button onClick={handleReload} variant="default">
            <RefreshCw className="h-4 w-4 mr-2" />
            Recargar la app
          </Button>
          <Button onClick={handleCopy} variant="outline">
            <Copy className="h-4 w-4 mr-2" />
            Copiar detalles
          </Button>
        </div>

        <p className="text-xs text-muted-foreground mt-4">
          Si esto se repite, abre un issue con los detalles copiados.
        </p>
      </div>
    </div>
  );
}
```

### 2. Registrar en el router

Edita donde esté el router (probablemente `src/main.tsx` o `src/App.tsx`):

```tsx
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { ErrorBoundary } from "./components/error-boundary";
import { AppShell } from "./components/layout/app-shell";
// ... resto de imports

const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    errorElement: <ErrorBoundary />, // ← añadir aquí
    children: [
      { index: true, element: <HomePage /> },
      // ... otras rutas
    ],
  },
]);
```

`errorElement` en la ruta raíz captura errores de TODAS las rutas hijas. Si necesitas un fallback diferente por ruta, añade `errorElement` por ruta.

### 3. Verificar con un crash forzado

Temporalmente, en una página cualquiera:

```tsx
export function CachePage() {
  throw new Error("test crash");
  // ...
}
```

Navega a esa página → debes ver el ErrorBoundary con el mensaje y los botones.

Borra el throw después.

## Criterio de done

- [ ] `src/components/error-boundary.tsx` existe.
- [ ] ErrorBoundary registrado en el router como `errorElement` de la ruta raíz.
- [ ] Forzar un `throw new Error("test")` muestra el ErrorBoundary, NO la pantalla feo de react-router.
- [ ] Botón "Recargar la app" funciona.
- [ ] Botón "Copiar detalles" copia JSON al clipboard.
- [ ] Ruta inexistente (`/no-existe-esto`) muestra el ErrorBoundary con mensaje "Página no encontrada".
- [ ] Layout no se rompe en pantallas pequeñas (≥ 1024px).
