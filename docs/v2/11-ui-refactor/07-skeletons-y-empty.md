# Paso 07 — TableSkeleton + EmptyState con action

**Área**: 11-ui-refactor
**Tiempo estimado**: 2-3 horas
**Dependencias**: ninguna

## Qué hacemos

Estandarizar loading states (skeleton) y empty states (con action opcional) en todas las páginas que muestran tabla/lista.

## Archivos

- `src/components/ui/table-skeleton.tsx` (nuevo)
- `src/components/empty-state.tsx` (modificar para action)

## Cómo

### 1. TableSkeleton

```tsx
// src/components/ui/table-skeleton.tsx
interface TableSkeletonProps {
  rows?: number;
  columns?: number;
}

export function TableSkeleton({ rows = 8, columns = 4 }: TableSkeletonProps) {
  return (
    <div className="border border-border rounded-lg overflow-hidden">
      <div className="border-b border-border bg-card p-3">
        <div className="flex gap-4">
          {Array.from({ length: columns }).map((_, i) => (
            <div key={i} className="h-4 bg-muted rounded animate-pulse" style={{ width: `${100/columns}%` }} />
          ))}
        </div>
      </div>
      <div>
        {Array.from({ length: rows }).map((_, i) => (
          <div key={i} className="border-b border-border/30 p-3 flex gap-4">
            {Array.from({ length: columns }).map((_, j) => (
              <div
                key={j}
                className="h-4 bg-muted rounded animate-pulse"
                style={{
                  width: `${100/columns}%`,
                  animationDelay: `${i * 100 + j * 50}ms`,
                }}
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
```

### 2. EmptyState con action

```tsx
// src/components/empty-state.tsx (modificar)
import { Button } from "./ui/button";
import type { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description: string;
  action?: {
    label: string;
    onClick: () => void;
    variant?: "default" | "outline";
  };
}

export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-4 text-center">
      <Icon className="h-12 w-12 text-muted-foreground/40 mb-4" strokeWidth={1.5} />
      <h3 className="text-lg font-medium mb-2">{title}</h3>
      <p className="text-sm text-muted-foreground max-w-md mb-4">{description}</p>
      {action && (
        <Button variant={action.variant ?? "default"} onClick={action.onClick}>
          {action.label}
        </Button>
      )}
    </div>
  );
}
```

### 3. Migrar páginas

En todas las páginas con `if (isLoading) return <div>Cargando...</div>`:

ANTES:
```tsx
if (isLoading) return <div className="p-6 text-muted-foreground">Cargando paquetes...</div>;
```

DESPUÉS:
```tsx
if (isLoading) return (
  <div className="p-6">
    <TableSkeleton rows={10} columns={5} />
  </div>
);
```

Para EmptyStates donde haya una acción sugerida:

ANTES:
```tsx
<EmptyState
  icon={FileText}
  title="Sin registros"
  description="Aún no se han realizado operaciones."
/>
```

DESPUÉS:
```tsx
<EmptyState
  icon={FileText}
  title="Sin registros"
  description="Aún no se han realizado operaciones destructivas."
  action={{
    label: "Ir a Debloat",
    onClick: () => navigate("/debloat"),
  }}
/>
```

## Criterio de done

- [ ] `<TableSkeleton />` reusable.
- [ ] `<EmptyState>` acepta `action` opcional.
- [ ] Mínimo 5 páginas migradas: cache, debloat, restore, audit, services.
- [ ] No queda ningún "Cargando..." textual.
- [ ] EmptyStates con action navegan correctamente.
