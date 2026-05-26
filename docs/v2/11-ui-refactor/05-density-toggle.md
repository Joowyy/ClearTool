# Paso 05 — Density toggle (comfortable / compact)

**Área**: 11-ui-refactor
**Tiempo estimado**: 2 horas
**Dependencias**: Paso 04 (themes ya tiene patrón de CSS vars)

## Qué hacemos

Toggle "Comfortable" vs "Compact" que reduce padding/row-height para power users con mucha data.

## Archivos

- `src/globals.css` (CSS vars para density)
- `src/features/settings/components/density-toggle.tsx` (UI)
- `src/hooks/use-density.ts` (aplicador)

## Cómo

### CSS

```css
:root[data-density="comfortable"] {
  --table-cell-padding-y: 0.75rem;
  --table-cell-padding-x: 1rem;
  --row-height: 2.75rem;
  --font-size-base: 0.875rem;  /* 14px */
}

:root[data-density="compact"] {
  --table-cell-padding-y: 0.375rem;
  --table-cell-padding-x: 0.625rem;
  --row-height: 2rem;
  --font-size-base: 0.8125rem;  /* 13px */
}
```

Tailwind:
```ts
// tailwind.config.ts
theme: {
  extend: {
    spacing: {
      "cell-y": "var(--table-cell-padding-y)",
      "cell-x": "var(--table-cell-padding-x)",
    },
    height: {
      row: "var(--row-height)",
    },
  },
},
```

Usar en tablas:
```tsx
<td className="px-cell-x py-cell-y">...</td>
```

### Hook

```ts
// src/hooks/use-density.ts
import { useEffect } from "react";
import { useAppStore } from "../lib/store";

export function useDensity() {
  const density = useAppStore(s => s.settings?.appearance.density ?? "comfortable");

  useEffect(() => {
    document.documentElement.setAttribute("data-density", density);
  }, [density]);
}
```

Llamar en AppShell junto a `useTheme()`.

### UI

```tsx
// src/features/settings/components/density-toggle.tsx
export function DensityToggle() {
  const settings = useAppStore(s => s.settings);
  const updateSettings = useAppStore(s => s.updateSettings);

  if (!settings) return null;

  return (
    <div className="space-y-2">
      <h3 className="font-medium">Densidad</h3>
      <div className="flex gap-2">
        {(["comfortable", "compact"] as const).map(d => (
          <button
            key={d}
            onClick={() => updateSettings({
              ...settings,
              appearance: { ...settings.appearance, density: d },
            })}
            className={`px-3 py-1.5 rounded border text-sm ${
              settings.appearance.density === d
                ? "border-signal-accent bg-signal-accent/10"
                : "border-border hover:bg-accent/30"
            }`}
          >
            {d === "comfortable" ? "Cómoda" : "Compacta"}
          </button>
        ))}
      </div>
    </div>
  );
}
```

## Criterio de done

- [ ] Toggle persistente en settings.
- [ ] Cambio aplica al instante.
- [ ] Tablas usan `px-cell-x py-cell-y`.
- [ ] Diferencia visible: filas más altas/bajas, font ligeramente más pequeña.
