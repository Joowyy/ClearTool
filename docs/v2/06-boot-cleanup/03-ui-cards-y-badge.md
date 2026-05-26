# Paso 03 — UI: card en Settings + badge en titlebar

**Área**: 06-boot-cleanup
**Tiempo estimado**: 3-4 horas
**Dependencias**: Paso 02

## Qué hacemos

1. Card en Settings → Avanzado con la lista de pendings + acción de cancelar.
2. Badge "Reiniciar pronto" en la titlebar si `count > 50`.

## Archivos

- `src/features/settings/components/pending-renames-card.tsx` (nuevo)
- `src/features/settings/settings-page.tsx` (integrar)
- `src/components/layout/titlebar.tsx` (modificar para badge)
- `src/hooks/use-pending-count.ts` (nuevo)

## Cómo

### 1. Hook para count global

```ts
// src/hooks/use-pending-count.ts
import { useQuery } from "@tanstack/react-query";
import { countPendingRenames } from "../api";

export function usePendingCount() {
  return useQuery({
    queryKey: ["pending-renames-count"],
    queryFn: countPendingRenames,
    refetchInterval: 30_000,  // cada 30s, no más
  });
}
```

### 2. Card en Settings

```tsx
// src/features/settings/components/pending-renames-card.tsx
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listPendingRenames, cancelPendingRename } from "../../../api";
import { toast } from "../../../lib/toast";
import { Trash2 } from "lucide-react";

export function PendingRenamesCard() {
  const qc = useQueryClient();
  const { data: pendings = [], refetch } = useQuery({
    queryKey: ["pending-renames"],
    queryFn: listPendingRenames,
  });

  const cancelMut = useMutation({
    mutationFn: (sourcePath: string) => cancelPendingRename(sourcePath),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["pending-renames"] });
      void qc.invalidateQueries({ queryKey: ["pending-renames-count"] });
      toast.success("Operación cancelada");
    },
    onError: (err) => toast.error("Error", err),
  });

  return (
    <div className="border border-border rounded-lg p-4">
      <div className="flex items-center justify-between mb-3">
        <div>
          <h3 className="font-medium">Operaciones pendientes para próximo reinicio</h3>
          <p className="text-sm text-muted-foreground">
            {pendings.length} archivos programados para eliminarse al arrancar
          </p>
        </div>
      </div>

      {pendings.length === 0 ? (
        <p className="text-sm text-muted-foreground">Sin operaciones pendientes.</p>
      ) : (
        <div className="space-y-1 max-h-96 overflow-auto">
          {pendings.slice(0, 100).map((p, i) => (
            <div key={i} className="flex items-center gap-2 text-xs py-1 border-b border-border/30">
              <code className="flex-1 truncate">{p.source}</code>
              <span className="text-muted-foreground">
                {p.isDelete ? "DELETE" : `→ ${p.destination}`}
              </span>
              <button
                onClick={() => cancelMut.mutate(p.source)}
                disabled={cancelMut.isPending}
                className="text-signal-red hover:text-red-300"
                title="Cancelar"
              >
                <Trash2 className="h-3 w-3" />
              </button>
            </div>
          ))}
          {pendings.length > 100 && (
            <div className="text-xs text-muted-foreground text-center py-2">
              +{pendings.length - 100} más (no mostradas)
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

### 3. Integrar en settings-page

```tsx
// src/features/settings/settings-page.tsx (sección Avanzado)
import { PendingRenamesCard } from "./components/pending-renames-card";

// ...
<TabsContent value="advanced">
  <PendingRenamesCard />
  {/* otros cards */}
</TabsContent>
```

### 4. Badge en titlebar

```tsx
// src/components/layout/titlebar.tsx (modificar)
import { usePendingCount } from "../../hooks/use-pending-count";
import { AlertCircle } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function Titlebar() {
  const { data: pendingCount = 0 } = usePendingCount();
  const navigate = useNavigate();

  return (
    <header data-tauri-drag-region className="pillar ...">
      {/* logo, tabs, etc */}

      {pendingCount > 50 && (
        <button
          onClick={() => navigate("/settings?tab=advanced")}
          className="no-drag flex items-center gap-1 px-2 py-1 rounded bg-warning/10 text-warning text-xs"
          title={`${pendingCount} archivos pendientes para reboot`}
        >
          <AlertCircle className="h-3 w-3" />
          Reiniciar pronto ({pendingCount})
        </button>
      )}

      {/* status chips, window controls */}
    </header>
  );
}
```

## Criterio de done

- [ ] Settings → Avanzado muestra lista de pendings con cancel inline.
- [ ] Cancelar refresca la lista y el count.
- [ ] Badge "Reiniciar pronto" aparece en titlebar cuando count > 50.
- [ ] Click en badge navega a Settings → Avanzado.
- [ ] Count se refresca cada 30s automáticamente.
- [ ] Sin admin, el listado puede estar vacío sin crash (silent fail OK aquí).
