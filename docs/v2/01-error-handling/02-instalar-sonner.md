# Paso 02 — Instalar sonner + Toaster en AppShell

**Área**: 01-error-handling
**Tiempo estimado**: 30 minutos
**Dependencias**: ninguna (puede ir en paralelo al Paso 01)

## Qué hacemos

Añadir [sonner](https://sonner.emilkowal.ski/) como librería de toasts y montar el `<Toaster />` en el shell.

## Por qué

Las páginas actuales muestran errores como banners inline grandes. Para errores transitorios (operación falla pero la página sigue funcional), un toast es más apropiado: no rompe el layout, no exige atención permanente, auto-dismiss.

## Archivos que tocamos

- `package.json` (añade sonner como dep)
- `src/components/layout/app-shell.tsx` (monta Toaster)

## Cómo

### 1. Instalar

```bash
npm i sonner
```

Verifica que se añadió a `package.json`:
```json
"dependencies": {
  "sonner": "^1.x.x",
  ...
}
```

### 2. Montar Toaster en AppShell

Edita `src/components/layout/app-shell.tsx`:

```tsx
// src/components/layout/app-shell.tsx
import { Outlet } from "react-router-dom";
import { Toaster } from "sonner";
import { Titlebar } from "./titlebar";
import { useElevation } from "../../hooks/use-elevation";
import { DebugConsole } from "../debug/debug-console";

export function AppShell() {
  useElevation();

  return (
    <div className="flex h-screen w-screen flex-col bg-surface-canvas text-ink-primary overflow-hidden">
      <Titlebar />
      <main className="relative flex-1 min-h-0 overflow-auto">
        <Outlet />
      </main>
      <DebugConsole />
      <Toaster
        theme="dark"
        position="bottom-right"
        richColors
        closeButton
        toastOptions={{
          classNames: {
            toast: "!bg-card !border-border",
          },
        }}
      />
    </div>
  );
}
```

Notas:
- `theme="dark"` coincide con el tema actual cyan oscuro. Si en M4 implementas tema light, cambiar a `theme="system"`.
- `position="bottom-right"` para no chocar con la titlebar.
- `richColors` da colores success (verde) / error (rojo) / warning (amarillo).
- `closeButton` añade una X para descartar manualmente.

### 3. Verificar visualmente

Lanza `npm run tauri dev` y, en cualquier componente, prueba temporalmente:

```tsx
import { toast } from "sonner";
// ...
<Button onClick={() => toast.success("test")}>Probar toast</Button>
```

Si ves el toast en bottom-right, todo bien. Después borra ese código de prueba.

## Criterio de done

- [ ] `sonner` aparece en `package.json` como dependencia.
- [ ] `<Toaster />` montado dentro de `<AppShell>`.
- [ ] Configuración: `theme="dark"`, `position="bottom-right"`, `richColors`, `closeButton`.
- [ ] Toast de prueba renderiza visualmente sin errores en consola.
- [ ] La app sigue compilando sin errores TypeScript.
