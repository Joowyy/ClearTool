# Paso 08 — Atajos de teclado + i18n + lazy routes

**Área**: 11-ui-refactor
**Tiempo estimado**: 6-8 horas
**Dependencias**: Paso 01 (sidebar), Paso 03 (palette)

## Qué hacemos

Tres mejoras finales del refactor:
1. Atajos de teclado documentados + modal Ctrl+/ con la cheatsheet.
2. i18n con react-i18next (ES + EN).
3. Lazy loading de rutas pesadas.

## Archivos

- `src/hooks/use-shortcuts.ts` (nuevo)
- `src/components/shortcuts-modal.tsx` (nuevo)
- `package.json` (i18next, react-i18next)
- `src/i18n/index.ts` (nuevo)
- `src/locales/es/*.json` (nuevo)
- `src/locales/en/*.json` (nuevo)
- `src/main.tsx` (init i18n + lazy routes)

## Cómo

### 1. Atajos

```ts
// src/hooks/use-shortcuts.ts
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { usePaletteStore } from "./use-command-palette";

export function useShortcuts() {
  const navigate = useNavigate();
  const setPalette = usePaletteStore(s => s.setOpen);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const cmd = e.ctrlKey || e.metaKey;

      // No interferir cuando el foco está en input
      const tgt = e.target as HTMLElement;
      if (tgt.tagName === "INPUT" || tgt.tagName === "TEXTAREA" || tgt.isContentEditable) {
        // Excepto Escape/Ctrl+K (esos siempre)
        if (e.key !== "Escape" && !(cmd && e.key === "k")) return;
      }

      if (cmd && e.key === "k") { e.preventDefault(); setPalette(true); }
      else if (cmd && e.key === ",") { e.preventDefault(); navigate("/settings"); }
      else if (cmd && e.key === "e") { e.preventDefault(); navigate("/explorer"); }
      else if (cmd && e.key === "b") { e.preventDefault(); navigate("/debloat"); }
      else if (cmd && e.key === "l") { e.preventDefault(); navigate("/cache"); }
      else if (cmd && e.key === "/") { e.preventDefault(); window.dispatchEvent(new Event("open-shortcuts")); }
      else if (e.key === "?" && !cmd) {
        // Solo si no estoy en input
        window.dispatchEvent(new Event("open-shortcuts"));
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate, setPalette]);
}
```

Llamar en `<AppShell>`: `useShortcuts();`.

### 2. Modal de cheatsheet

```tsx
// src/components/shortcuts-modal.tsx
import { useEffect, useState } from "react";

const SHORTCUTS: Array<[string, string]> = [
  ["Ctrl+K", "Command palette"],
  ["Ctrl+/", "Mostrar este modal"],
  ["Ctrl+,", "Ajustes"],
  ["Ctrl+E", "Ir a Explorer"],
  ["Ctrl+B", "Ir a Debloat"],
  ["Ctrl+L", "Ir a Caché"],
  ["Esc", "Cerrar modal/palette"],
];

export function ShortcutsModal() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const handler = () => setOpen(true);
    window.addEventListener("open-shortcuts", handler);
    const esc = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); };
    window.addEventListener("keydown", esc);
    return () => {
      window.removeEventListener("open-shortcuts", handler);
      window.removeEventListener("keydown", esc);
    };
  }, []);

  if (!open) return null;

  return (
    <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4"
         onClick={() => setOpen(false)}>
      <div className="bg-card border border-border rounded-lg max-w-md w-full p-6"
           onClick={(e) => e.stopPropagation()}>
        <h2 className="text-lg font-bold mb-4">Atajos de teclado</h2>
        <div className="space-y-2">
          {SHORTCUTS.map(([key, action]) => (
            <div key={key} className="flex justify-between items-center text-sm">
              <span className="text-muted-foreground">{action}</span>
              <kbd className="px-2 py-1 bg-muted rounded font-mono text-xs">{key}</kbd>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
```

Montar en AppShell.

### 3. i18n

```bash
npm i i18next react-i18next
```

```ts
// src/i18n/index.ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import esCommon from "../locales/es/common.json";
import esNav from "../locales/es/nav.json";
import enCommon from "../locales/en/common.json";
import enNav from "../locales/en/nav.json";

i18n.use(initReactI18next).init({
  resources: {
    es: { common: esCommon, nav: esNav },
    en: { common: enCommon, nav: enNav },
  },
  lng: detectLang(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
  ns: ["common", "nav"],
  defaultNS: "common",
});

function detectLang(): string {
  const browser = navigator.language.toLowerCase();
  if (browser.startsWith("es")) return "es";
  return "en";
}

export default i18n;
```

```json
// src/locales/es/nav.json
{
  "home": "Inicio",
  "explorer": "Explorador",
  "cache": "Caché",
  "debloat": "Debloat",
  "services": "Servicios",
  "registry": "Registro",
  "restore": "Restauración",
  "audit": "Auditoría",
  "processes": "Procesos",
  "startup": "Arranque",
  "disk": "Disco",
  "privacy": "Privacidad",
  "settings": "Ajustes"
}
```

```json
// src/locales/en/nav.json
{
  "home": "Home",
  "explorer": "Explorer",
  "cache": "Cache",
  "debloat": "Debloat",
  "services": "Services",
  "registry": "Registry",
  "restore": "Restore",
  "audit": "Audit log",
  "processes": "Processes",
  "startup": "Startup",
  "disk": "Disk",
  "privacy": "Privacy",
  "settings": "Settings"
}
```

Import en `main.tsx`:
```tsx
import "./i18n";
```

Uso en sidebar (paso 01):
```tsx
import { useTranslation } from "react-i18next";
const { t } = useTranslation("nav");
// ...
<span>{t(item.labelKey)}</span>
```

### 4. Lazy routes

```tsx
// src/main.tsx
import { lazy, Suspense } from "react";

const HomePage = lazy(() => import("./features/home/home-page").then(m => ({ default: m.HomePage })));
const DebloatPage = lazy(() => import("./features/debloat/debloat-page").then(m => ({ default: m.DebloatPage })));
const ProcessesPage = lazy(() => import("./features/processes/processes-page").then(m => ({ default: m.ProcessesPage })));
const DiskAnalyzerPage = lazy(() => import("./features/disk-analyzer/disk-analyzer-page").then(m => ({ default: m.DiskAnalyzerPage })));

const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    errorElement: <ErrorBoundary />,
    children: [
      { index: true, element: <Suspense fallback={<TableSkeleton />}><HomePage /></Suspense> },
      { path: "/debloat", element: <Suspense fallback={<TableSkeleton />}><DebloatPage /></Suspense> },
      // ... resto
    ],
  },
]);
```

## Criterio de done

- [ ] Ctrl+K, Ctrl+/, Ctrl+,, Ctrl+E, Ctrl+B, Ctrl+L funcionan.
- [ ] Modal Ctrl+/ lista los 7+ atajos.
- [ ] Sidebar labels vienen de `t("nav.X")`.
- [ ] Cambiar idioma en Settings cambia el sidebar inmediatamente.
- [ ] `npm run build` produce chunks separados para rutas lazy.
- [ ] First Contentful Paint < 800ms en hardware medio.
