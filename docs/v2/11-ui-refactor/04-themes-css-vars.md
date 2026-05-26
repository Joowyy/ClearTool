# Paso 04 — Theme system (3 themes + follow-system)

**Área**: 11-ui-refactor
**Tiempo estimado**: 4-5 horas
**Dependencias**: ninguna

## Qué hacemos

3 themes built-in (`dark-cyan`, `dark-amber`, `light`) + auto-detect del OS, via CSS variables.

## Archivos

- `src/index.css` o `src/globals.css` (definir variables por theme)
- `tailwind.config.ts` (mapear variables a clases utilities)
- `src/hooks/use-theme.ts` (lógica de aplicar)
- `src/lib/store.ts` (theme state)
- `src/features/settings/components/theme-picker.tsx` (UI)

## Cómo

### 1. CSS variables

```css
/* src/globals.css */

:root {
  /* Defaults — sobrescribibles por [data-theme] */
  --bg-canvas: 15 23 42;          /* slate-900 */
  --bg-surface-1: 30 41 59;       /* slate-800 */
  --bg-card: 30 41 59;
  --border-default: 51 65 85;     /* slate-700 */
  --text-primary: 248 250 252;    /* slate-50 */
  --text-secondary: 203 213 225;  /* slate-300 */
  --text-tertiary: 148 163 184;   /* slate-400 */
  --signal-cyan: 6 182 212;
  --signal-emerald: 16 185 129;
  --signal-red: 239 68 68;
  --signal-amber: 245 158 11;
  --signal-violet: 139 92 246;
}

:root[data-theme="dark-cyan"] {
  --signal-accent: 6 182 212;
}

:root[data-theme="dark-amber"] {
  --signal-accent: 245 158 11;
  /* Override de chip cyan en amber */
  --signal-cyan: 245 158 11;
}

:root[data-theme="light"] {
  --bg-canvas: 250 250 250;
  --bg-surface-1: 244 244 245;
  --bg-card: 255 255 255;
  --border-default: 228 228 231;
  --text-primary: 24 24 27;
  --text-secondary: 82 82 91;
  --text-tertiary: 113 113 122;
  --signal-cyan: 8 145 178;
  --signal-accent: 8 145 178;
}
```

### 2. Tailwind config

```ts
// tailwind.config.ts
export default {
  theme: {
    extend: {
      colors: {
        surface: {
          canvas: "rgb(var(--bg-canvas) / <alpha-value>)",
          1: "rgb(var(--bg-surface-1) / <alpha-value>)",
        },
        card: "rgb(var(--bg-card) / <alpha-value>)",
        border: "rgb(var(--border-default) / <alpha-value>)",
        ink: {
          primary: "rgb(var(--text-primary) / <alpha-value>)",
          secondary: "rgb(var(--text-secondary) / <alpha-value>)",
          tertiary: "rgb(var(--text-tertiary) / <alpha-value>)",
        },
        signal: {
          cyan: "rgb(var(--signal-cyan) / <alpha-value>)",
          emerald: "rgb(var(--signal-emerald) / <alpha-value>)",
          red: "rgb(var(--signal-red) / <alpha-value>)",
          amber: "rgb(var(--signal-amber) / <alpha-value>)",
          violet: "rgb(var(--signal-violet) / <alpha-value>)",
          accent: "rgb(var(--signal-accent) / <alpha-value>)",
        },
      },
    },
  },
};
```

### 3. Hook que aplica

```ts
// src/hooks/use-theme.ts
import { useEffect } from "react";
import { useAppStore } from "../lib/store";

export type ThemeName = "dark-cyan" | "dark-amber" | "light";
export type ThemeMode = "dark-cyan" | "dark-amber" | "light" | "system";

export function useTheme() {
  const themeMode = useAppStore(s => s.settings?.appearance.theme as ThemeMode ?? "dark-cyan");
  const followSystem = useAppStore(s => s.settings?.appearance.followSystem ?? false);

  useEffect(() => {
    let resolved: ThemeName;
    if (followSystem || themeMode === "system") {
      resolved = window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark-cyan"
        : "light";
    } else {
      resolved = themeMode as ThemeName;
    }
    document.documentElement.setAttribute("data-theme", resolved);
  }, [themeMode, followSystem]);

  // Listener para cambios de OS
  useEffect(() => {
    if (!followSystem) return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      document.documentElement.setAttribute(
        "data-theme",
        mq.matches ? "dark-cyan" : "light"
      );
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [followSystem]);
}
```

Llamar en `<AppShell>`:
```tsx
function AppShell() {
  useElevation();
  useTheme();
  // ...
}
```

### 4. UI Settings → Apariencia

```tsx
// src/features/settings/components/theme-picker.tsx
import { useAppStore } from "../../../lib/store";

export function ThemePicker() {
  const settings = useAppStore(s => s.settings);
  const updateSettings = useAppStore(s => s.updateSettings);

  if (!settings) return null;

  const setTheme = (theme: string) => {
    updateSettings({
      ...settings,
      appearance: { ...settings.appearance, theme },
    });
  };

  return (
    <div className="space-y-2">
      <h3 className="font-medium">Tema</h3>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
        {THEME_OPTIONS.map(opt => (
          <button
            key={opt.value}
            onClick={() => setTheme(opt.value)}
            className={`p-3 border rounded-lg text-left ${
              settings.appearance.theme === opt.value
                ? "border-signal-accent bg-signal-accent/10"
                : "border-border hover:bg-accent/30"
            }`}
          >
            <div className="font-medium text-sm">{opt.label}</div>
            <div className="text-xs text-muted-foreground">{opt.desc}</div>
            <div className="mt-2 flex gap-1">
              {opt.preview.map(c => (
                <div key={c} className="h-4 w-4 rounded" style={{ background: c }} />
              ))}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

const THEME_OPTIONS = [
  { value: "dark-cyan", label: "Dark Cyan", desc: "Default", preview: ["#0f172a", "#06b6d4"] },
  { value: "dark-amber", label: "Dark Amber", desc: "Cálido", preview: ["#0f172a", "#f59e0b"] },
  { value: "light", label: "Light", desc: "Para uso diurno", preview: ["#fafafa", "#0891b2"] },
  { value: "system", label: "Sistema", desc: "Sigue el OS", preview: ["#0f172a", "#fafafa"] },
];
```

## Criterio de done

- [ ] 3 themes con CSS variables.
- [ ] Cambiar theme aplica al instante sin recarga.
- [ ] `system` detecta `prefers-color-scheme`.
- [ ] Cambio de tema del OS (en `system`) se refleja en vivo.
- [ ] Contraste WCAG AA en los 3 (verificable con axe DevTools).
- [ ] Settings → Apariencia muestra los 4 botones con preview.
