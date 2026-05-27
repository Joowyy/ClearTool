# 04 — React Router v7 future flag warning

> **Severidad:** 🟢 P2 — warning informativo. No rompe nada hoy, pero
> conviene activar el flag para que el comportamiento futuro ya esté
> sincronizado con v7.

## 1. Problema

Warning en consola:

```
React Router Future Flag Warning: React Router will begin wrapping
state updates in `React.startTransition` in v7. You can use the
`v7_startTransition` future flag to opt-in early. For more information,
see https://reactrouter.com/v6/upgrading/future#v7_starttransition.
```

## 2. Causa raíz

`src/router.tsx:30` — `createBrowserRouter` se llama sin opciones
futuras:

```tsx
export const router = createBrowserRouter([...]);
```

React Router v6 introduce flags `future.*` para activar gradualmente el
comportamiento de v7. El warning se dispara porque la app no las activa.

## 3. Fix propuesto

`src/router.tsx`:

```tsx
export const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <AppShell />,
      errorElement: <ErrorBoundary />,
      children: [
        // ... rutas existentes ...
      ],
    },
  ],
  {
    future: {
      v7_startTransition: true,
      v7_relativeSplatPath: true,
      v7_fetcherPersist: true,
      v7_normalizeFormMethod: true,
      v7_partialHydration: true,
      v7_skipActionErrorRevalidation: true,
    },
  }
);
```

Activar **todas** las flags v7 disponibles en una sola tanda:

| Flag | Qué cambia |
|---|---|
| `v7_startTransition` | Las actualizaciones de estado del router van dentro de `React.startTransition`, evitando bloquear el render durante navegaciones. |
| `v7_relativeSplatPath` | Cambia cómo se resuelven rutas splat (`*`) relativas. ClearTool no usa splats, sin impacto. |
| `v7_fetcherPersist` | Persiste fetchers tras desmontar el componente owner. ClearTool no usa `useFetcher`, sin impacto. |
| `v7_normalizeFormMethod` | `formMethod` siempre en MAYÚSCULAS. ClearTool no usa forms del router. |
| `v7_partialHydration` | Permite hidratar parcialmente en SSR. ClearTool es CSR puro, sin impacto. |
| `v7_skipActionErrorRevalidation` | No revalida tras errores en `action`. ClearTool no usa actions. |

→ Sólo `v7_startTransition` cambia algo real en ClearTool, el resto es
preparación para v7 sin coste.

## 4. Criterio de done

- [ ] Warning de React Router desaparece de la consola.
- [ ] Navegación entre páginas sigue funcionando idéntica.
- [ ] Lazy loading de páginas (`React.lazy` + `<Suspense>`) sigue
      mostrando el skeleton mientras carga el chunk.

## 5. Riesgos / efectos secundarios

- `v7_startTransition` puede hacer que algunas transiciones sean
  "interrumpibles". Es lo que queremos: si el usuario navega rápido
  entre pestañas, no se queda atascado en una pesada.
- Cuando hagamos upgrade real a React Router v7 (próximos meses), el
  upgrade será una no-op porque ya estamos con el comportamiento v7.
