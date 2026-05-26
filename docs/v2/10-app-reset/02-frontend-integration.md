# Paso 02 — Integración UI en Debloat

**Área**: 10-app-reset
**Tiempo estimado**: 1.5 horas
**Dependencias**: Paso 01

## Qué hacemos

Añadir botón secundario "Reset" en cada fila de Debloat que tenga `removalMethod: "appx-*"`, con confirm modal explícito.

## Archivos

- `src/features/debloat/debloat-page.tsx` (modificar)
- `src/features/debloat/components/reset-button.tsx` (nuevo)

## Cómo

### Botón

```tsx
// src/features/debloat/components/reset-button.tsx
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { RotateCcw } from "lucide-react";
import { Button } from "../../../components/ui/button";
import { resetUwpApp } from "../../../api";
import { toast } from "../../../lib/toast";

interface ResetButtonProps {
  displayName: string;
  packageFamilyName: string | null;
  isElevated: boolean;
}

export function ResetButton({ displayName, packageFamilyName, isElevated }: ResetButtonProps) {
  const [confirming, setConfirming] = useState(false);

  const reset = useMutation({
    mutationFn: (pfn: string) => resetUwpApp(pfn),
    onSuccess: () => toast.success(`${displayName} reseteada`, {
      description: "Login y caché local se han borrado.",
    }),
    onError: (err) => toast.error(`Error reseteando ${displayName}`, err),
  });

  if (!packageFamilyName) return null;

  if (confirming) {
    return (
      <div className="flex gap-1">
        <Button
          size="sm"
          variant="destructive"
          onClick={(e) => {
            e.stopPropagation();
            reset.mutate(packageFamilyName);
            setConfirming(false);
          }}
          disabled={!isElevated || reset.isPending}
        >
          Confirmar
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={(e) => {
            e.stopPropagation();
            setConfirming(false);
          }}
        >
          Cancelar
        </Button>
      </div>
    );
  }

  return (
    <Button
      size="sm"
      variant="outline"
      onClick={(e) => {
        e.stopPropagation();
        setConfirming(true);
      }}
      disabled={!isElevated}
      title={!isElevated ? "Requiere administrador" : "Resetear app (pierde login y data local)"}
    >
      <RotateCcw className="h-3 w-3 mr-1" />
      Reset
    </Button>
  );
}
```

### Integración en tabla de Debloat

```tsx
// debloat-page.tsx — en la fila de la tabla:
<td className="p-3 text-right">
  <ResetButton
    displayName={entry.displayName}
    packageFamilyName={entry.appxPackageFamilyName ?? null}
    isElevated={isElevated}
  />
</td>
```

Añadir columna nueva "Acciones" en el `<thead>`:

```tsx
<th className="text-right p-3 ...">Acciones</th>
```

### Modo experto

Por defecto la columna está oculta. Settings → Avanzado → toggle "Modo experto" la activa. Por qué: para usuarios normales, "Reset" es confuso vs "Eliminar". Sólo lo ofrecemos a quien sepa qué es.

```tsx
const expertMode = useAppStore(s => s.expertMode);
// ...
{expertMode && <th>Acciones</th>}
// ...
{expertMode && <td><ResetButton ... /></td>}
```

## Criterio de done

- [ ] Columna "Acciones" con ResetButton aparece solo si Modo experto activo.
- [ ] Click "Reset" abre confirm inline (2 botones).
- [ ] Confirmar ejecuta `resetUwpApp` con toast.
- [ ] Sin admin, el botón está deshabilitado con tooltip.
- [ ] Entradas sin `appxPackageFamilyName` no muestran el botón.
