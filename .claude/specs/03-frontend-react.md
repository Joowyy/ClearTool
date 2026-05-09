# 03 — Frontend React + TypeScript

## Stack

- React 18 + TypeScript estricto (`strict: true`, `noUncheckedIndexedAccess: true`).
- Vite 5 como bundler.
- TailwindCSS 3 + `tailwindcss-animate`.
- shadcn/ui (Radix primitives + Tailwind variants).
- Zustand para estado global pequeño (preferencias, sesión).
- React Query para datos de comandos Tauri (cache, retry, invalidación).
- React Router 6 para navegación entre páginas.
- `lucide-react` para iconos.
- `@tanstack/react-virtual` o `react-arborist` para árbol virtualizado.

## Estructura `src/`

```
src/
├── main.tsx
├── App.tsx
├── router.tsx
├── bindings/                 # autogenerado por ts-rs
│   ├── index.ts
│   └── ...
├── components/
│   ├── ui/                   # shadcn primitives
│   ├── layout/
│   │   ├── app-shell.tsx
│   │   └── sidebar-nav.tsx
│   ├── tree/
│   │   ├── file-tree.tsx
│   │   └── node-row.tsx
│   ├── confirm-destructive/
│   │   ├── index.tsx
│   │   ├── tabs/
│   │   │   ├── summary-tab.tsx
│   │   │   ├── diff-tab.tsx
│   │   │   ├── risks-tab.tsx
│   │   │   └── reversal-tab.tsx
│   ├── progress/
│   │   └── operation-progress.tsx
│   ├── data-table/
│   │   └── data-table.tsx
│   └── empty-state.tsx
├── features/
│   ├── explorer/
│   │   ├── explorer-page.tsx
│   │   ├── use-tree-stream.ts
│   │   └── breadcrumbs.tsx
│   ├── cache-cleaner/
│   │   ├── cache-page.tsx
│   │   ├── locations-table.tsx
│   │   └── use-clean-cache.ts
│   ├── debloat/
│   │   ├── debloat-page.tsx
│   │   ├── bloatware-table.tsx
│   │   └── category-filter.tsx
│   ├── services/
│   │   ├── services-page.tsx
│   │   └── service-row.tsx
│   ├── registry-tweaks/
│   │   ├── registry-page.tsx
│   │   └── tweaks-list.tsx
│   ├── restore-points/
│   │   ├── restore-page.tsx
│   │   └── points-table.tsx
│   └── settings/
│       └── settings-page.tsx
├── hooks/
│   ├── use-tauri-command.ts
│   ├── use-tauri-event.ts
│   ├── use-elevation.ts
│   └── use-system-summary.ts
├── lib/
│   ├── tauri.ts
│   ├── format.ts
│   ├── humanize-error.ts
│   ├── routes.ts
│   └── store.ts              # zustand
├── pages/
│   └── home.tsx
└── styles/
    └── globals.css
```

## Páginas

| Ruta | Página | Descripción |
|---|---|---|
| `/` | Home | Resumen sistema, accesos rápidos, alerta si no elevado. |
| `/explorer` | Explorer | Árbol de directorios con tamaños y filtros. |
| `/cache` | Cache cleaner | Lista de ubicaciones de caché con tamaños y limpieza. |
| `/debloat` | Debloat | Tabla de paquetes detectados con categorías y selección. |
| `/services` | Services | Tabla de servicios con estados y acciones. |
| `/registry` | Registry tweaks | Lista de tweaks aplicables/aplicados. |
| `/restore` | Restore points | Listar / crear / restaurar puntos. |
| `/settings` | Settings | Idioma, tema, scope de borrado, etc. |

## Patrón de invocación + eventos

```ts
// hooks/use-tauri-event.ts
import { useEffect } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export function useTauriEvent<T>(name: string, handler: (payload: T) => void) {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<T>(name, (e) => handler(e.payload)).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [name, handler]);
}
```

```ts
// features/cache-cleaner/use-clean-cache.ts
export function useCleanCache() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CleanCacheInput) => cleanCacheLocations(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['cache-locations'] });
      toast.success('Limpieza completada');
    },
    onError: (e) => toast.error(humanizeError(e as AppError)),
  });
}
```

## Flujo destructivo (UI)

1. Usuario selecciona items (paquetes, ubicaciones, tweaks).
2. Click "Aplicar".
3. Aparece `<ConfirmDestructive>` con tabs:
   - **Resumen:** "Vas a quitar 12 paquetes y liberar 3.4 GB."
   - **Detalle:** lista exacta con tamaño/riesgo por item.
   - **Riesgos:** lista agregada de consecuencias.
   - **Reversa:** "Se creará el restore point #N. Para deshacer X, ver pasos."
4. Checkbox "He revisado".
5. Botones: `Dry-run` (secundario), `Aplicar` (destructivo, disabled hasta checkbox).
6. Operación corre con barra de progreso vía eventos.
7. Resultado: toast + redirección a `/restore` mostrando el punto creado.

## Modo limitado (sin admin)

Si `is_elevated()` devuelve `false`:

- Banner persistente en el shell: "Modo limitado: algunas funciones requieren admin. [Reiniciar como admin]".
- Páginas destructivas muestran sus tablas pero los CTAs principales aparecen deshabilitados con tooltip explicativo.
- Click en "Reiniciar como admin" llama a `tauri-plugin-process::relaunch_as_admin` (o invoca un comando que ejecute `ShellExecute` con `runas`).

## Tema

```ts
// lib/store.ts
type ThemeMode = 'light' | 'dark' | 'system';
interface UiStore {
  theme: ThemeMode;
  setTheme: (t: ThemeMode) => void;
  language: 'es' | 'en';
  setLanguage: (l: 'es' | 'en') => void;
}
```

Default: dark, idioma `es`.

## i18n

Usar `react-i18next`. Carpeta `src/locales/{es,en}/common.json`. Strings en JSX van por `t('key')`. PRs deben mantener ambos idiomas.

## Tests frontend

- `vitest` + `@testing-library/react` para componentes.
- `playwright` para E2E con la app empaquetada (Tauri WebDriver).
- Tests de hooks que invocan Tauri usan mocks del `invoke`.

## Performance

- `react-arborist` para el árbol — virtualizado nativo.
- Tablas grandes: `@tanstack/react-table` + `@tanstack/react-virtual`.
- Memoización agresiva con `useMemo`/`memo` en componentes que reciben listas largas.
