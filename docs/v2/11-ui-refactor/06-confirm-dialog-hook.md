# Paso 06 — `useConfirm()` hook estandarizado

**Área**: 11-ui-refactor
**Tiempo estimado**: 3 horas
**Dependencias**: ninguna

## Qué hacemos

Hook promise-based para confirmar acciones, eliminando los `confirmDelete` state local repetidos en cada página.

## Archivos

- `src/hooks/use-confirm.ts` (nuevo)
- `src/components/confirm-provider.tsx` (nuevo)
- `src/main.tsx` (envolver app con provider)

## Cómo

### Implementación

```tsx
// src/components/confirm-provider.tsx
import { createContext, useContext, useState, useCallback, useRef } from "react";
import { AlertTriangle, AlertOctagon, Info } from "lucide-react";
import { Button } from "./ui/button";

interface ConfirmOptions {
  title: string;
  description?: string;
  consequences?: string[];
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}

type Resolver = (ok: boolean) => void;

const ConfirmContext = createContext<((opts: ConfirmOptions) => Promise<boolean>) | null>(null);

export function ConfirmProvider({ children }: { children: React.ReactNode }) {
  const [opts, setOpts] = useState<ConfirmOptions | null>(null);
  const resolverRef = useRef<Resolver | null>(null);

  const confirm = useCallback((options: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      resolverRef.current = resolve;
      setOpts(options);
    });
  }, []);

  const handle = (ok: boolean) => {
    resolverRef.current?.(ok);
    resolverRef.current = null;
    setOpts(null);
  };

  return (
    <ConfirmContext.Provider value={confirm}>
      {children}
      {opts && (
        <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4"
             onClick={() => handle(false)}>
          <div className="bg-card border border-border rounded-lg max-w-md w-full p-6 flex flex-col gap-4"
               onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center gap-3">
              {opts.danger ? <AlertOctagon className="h-7 w-7 text-signal-red" /> :
                <AlertTriangle className="h-7 w-7 text-yellow-400" />}
              <h2 className="text-lg font-bold">{opts.title}</h2>
            </div>
            {opts.description && <p className="text-sm text-muted-foreground">{opts.description}</p>}
            {opts.consequences && opts.consequences.length > 0 && (
              <ul className="text-sm space-y-1 list-disc list-inside text-muted-foreground">
                {opts.consequences.map((c, i) => <li key={i}>{c}</li>)}
              </ul>
            )}
            <div className="flex gap-2 justify-end mt-2">
              <Button variant="outline" onClick={() => handle(false)}>
                {opts.cancelLabel ?? "Cancelar"}
              </Button>
              <Button
                variant={opts.danger ? "destructive" : "default"}
                onClick={() => handle(true)}
              >
                {opts.confirmLabel ?? "Confirmar"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </ConfirmContext.Provider>
  );
}

export function useConfirm() {
  const ctx = useContext(ConfirmContext);
  if (!ctx) throw new Error("useConfirm fuera de <ConfirmProvider>");
  return ctx;
}
```

### Envolver app

```tsx
// src/main.tsx
import { ConfirmProvider } from "./components/confirm-provider";

ReactDOM.createRoot(...).render(
  <QueryClientProvider client={queryClient}>
    <ConfirmProvider>
      <RouterProvider router={router} />
    </ConfirmProvider>
  </QueryClientProvider>
);
```

### Uso

```tsx
import { useConfirm } from "../../hooks/use-confirm";

function DebloatPage() {
  const confirm = useConfirm();

  const handleRemove = async () => {
    const ok = await confirm({
      title: "Eliminar 25 packages",
      description: "Esto desinstalará los packages seleccionados.",
      consequences: [
        "Se creará un restore point antes.",
        "Algunas apps cloud pueden necesitar re-login.",
      ],
      confirmLabel: "Eliminar",
      danger: true,
    });
    if (ok) removeMutation.mutate(...);
  };
  // ...
}
```

### Cleanup en páginas existentes

Buscar `const [confirm*, setConfirm*] = useState(...)` en las páginas y reemplazar por `useConfirm()`:

- `restore-page.tsx` → `confirmRestore` state se elimina
- `audit-page.tsx` (si tuviera) → idem
- etc.

## Criterio de done

- [ ] `<ConfirmProvider>` envuelve la app.
- [ ] `useConfirm()` devuelve función promise-based.
- [ ] Modal con icono según `danger`, botones de confirmar/cancelar.
- [ ] Click fuera del modal cierra como cancel.
- [ ] Mínimo 3 páginas migradas (debloat, restore, services o lo que aplique).
- [ ] Cero state local de `confirmDelete`/`confirmRestore` etc.
