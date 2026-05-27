---
name: react-frontend-cleartool
description: Convenciones del frontend React/TypeScript de ClearTool — invocación tipada de comandos Tauri vía wrappers en src/lib/tauri.ts usando los bindings de ts-rs, estado de servidor con TanStack Query, estado de UI con Zustand, toasts/errores vía el skill error-model, confirm dialogs estandarizados para operaciones destructivas, virtualización de listas grandes, themes con CSS variables, i18n (ES/EN) y lazy loading de rutas pesadas. Usar al crear o modificar cualquier componente, página, hook o feature del frontend. NO es una skill de diseño visual genérica: codifica las reglas concretas de este proyecto.
---

# Skill: react-frontend-cleartool

Reglas del frontend de ClearTool. Complementa (no sustituye) las skills de diseño visual importadas en `.claude/skills/disenio-frontend/`; esta skill manda en arquitectura, datos y patrones de interacción.

## Cuándo usar

- Creas/modificas un componente, página, hook o feature en `src/`.
- Añades una pantalla nueva para un comando Tauri.
- Tocas routing, estado, themes o i18n.

## Estructura

```
src/
  bindings/        # tipos generados por ts-rs (NO editar a mano)
  lib/             # tauri.ts (wrappers IPC), errors.ts, toast.ts, i18n.ts
  components/      # UI reutilizable (shadcn/ui), error-boundary, confirm-dialog, table-skeleton, empty-state
  features/<x>/    # una carpeta por módulo: <x>-page.tsx, hooks, componentes locales
  store/           # stores Zustand (UI state)
```

## 1. IPC tipado — nunca `invoke` suelto en un componente

Todo comando Tauri se envuelve una vez en `src/lib/tauri.ts`, tipado con los bindings de `ts-rs`. Los componentes importan el wrapper, nunca `invoke` directo.

```ts
// src/lib/tauri.ts
import { invoke } from "@tauri-apps/api/core";
import type { ProcessInfo, CleanPlan, CleanReport } from "@/bindings";

export const listProcesses     = () => invoke<ProcessInfo[]>("list_processes");
export const closeGracefully   = (pid: number, timeoutMs = 5000) =>
  invoke<boolean>("close_gracefully", { pid, timeoutMs });
export const analyzeLocations  = (ids: string[]) => invoke<CleanPlan>("analyze_locations", { ids });
export const executePlan       = (plan: CleanPlan, dryRun = false) =>
  invoke<CleanReport>("execute_plan", { plan, dryRun });
```

Regla: el nombre del comando y de los args en camelCase deben coincidir con el `#[tauri::command]` (ver skill `tauri-command-builder`).

## 2. Estado de servidor = TanStack Query; estado de UI = Zustand

- **Datos del backend** (listados, planes, estados): `useQuery` / `useMutation`. Nunca metas resultados de IPC en `useState` manual.
- **Estado de UI** (sidebar abierto, theme, density, filtros, selección): store Zustand con selectores.

```ts
const { data: procs, isLoading, error } = useQuery({
  queryKey: ["processes"],
  queryFn: listProcesses,
  refetchInterval: tabActive ? 2000 : false,   // polling solo con la pestaña activa
});

const kill = useMutation({
  mutationFn: (pid: number) => killProcess(pid),
  onError: (err) => toast.error("No se pudo terminar el proceso", err),
  onSuccess: () => qc.invalidateQueries({ queryKey: ["processes"] }),
});
```

## 3. Errores y toasts: vía skill error-model

Nunca `String(err)` en JSX. `onError: (err) => toast.error("mensaje", err)`. Errores de render → `ErrorBoundary` del router. Detalle completo en el skill `error-model`.

## 4. Operaciones destructivas: ConfirmDialog + dry-run primero

Toda acción destructiva pasa por el componente `ConfirmDialog` estandarizado, que muestra **exactamente qué se va a hacer** (principio de transparencia total del proyecto). Cuando el comando soporta `dry_run`, se ejecuta primero en dry-run y se muestra el plan antes de confirmar.

```tsx
const ok = await confirm({
  title: "Liberar para limpieza",
  body: "Esto cerrará Chrome, Edge, Spotify, Discord. ¿Continuar?",
  confirmLabel: "Cerrar apps",
  tone: "destructive",
});
if (ok) releaseMutation.mutate();
```

## 5. Listas grandes: virtualizar siempre

Árboles de archivos, procesos (300+), catálogos de debloat (120+) y treemap: usar `@tanstack/react-virtual` / `react-virtuoso` / `react-arborist`. Nunca renderizar miles de filas. El streaming por eventos (`explorer:batch`) se acumula en el store y la lista virtual lee de ahí.

```ts
listen<TreeNode[]>("explorer:batch", (e) => store.appendNodes(e.payload));
```

## 6. Estados de UI completos

Cada pantalla cubre los 4 estados: loading (`TableSkeleton`), vacío (`EmptyState` con acción), error (inline o toast según rompa o no render), y datos. No dejar pantallas en blanco mientras carga.

## 7. Themes con CSS variables

3 themes (dark-cyan, dark-amber, light) + follow-system, vía variables CSS en `:root[data-theme=...]`. Los componentes usan tokens semánticos (`bg-card`, `text-muted-foreground`, `border`, `text-signal-red`), nunca colores hardcodeados. El theme vive en el store Zustand y se persiste.

## 8. i18n (ES base, EN)

Strings de UI vía `t("clave")` desde `src/lib/i18n.ts`. ES es el idioma base (coherente con la convención del proyecto: docs/comentarios en español). No hardcodear texto visible en componentes.

## 9. Command Palette y atajos

Ctrl+K abre el command palette (busca en rutas + catálogos, objetivo <100ms). Ctrl+/ muestra el modal de atajos. Las acciones se registran en un único registry para que palette y sidebar compartan fuente.

## 10. Lazy loading de rutas pesadas

Disk Analyzer (treemap), Explorer y Procesos se cargan con `React.lazy` + `Suspense` para no inflar el bundle inicial.

```ts
const DiskAnalyzer = lazy(() => import("@/features/disk-analyzer/disk-analyzer-page"));
```

## Anti-patrones (rechazar en review)

- `invoke(...)` directo dentro de un `.tsx` de feature.
- `String(err)` / `${err}` en JSX.
- Resultados de IPC en `useState` en vez de TanStack Query.
- Colores hex hardcodeados en vez de tokens del theme.
- Listas no virtualizadas con N grande.
- Operación destructiva sin ConfirmDialog ni dry-run.

## Tests

- Hooks de mutación: mockear el wrapper de `tauri.ts` y verificar `onError` → toast.
- Render de cada `*-page` en sus 4 estados (loading/empty/error/data) con Testing Library.
- ConfirmDialog: no dispara la mutación si el usuario cancela.
