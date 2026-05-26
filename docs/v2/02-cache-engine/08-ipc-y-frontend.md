# Paso 08 — IPC commands + frontend rediseñado

**Área**: 02-cache-engine
**Tiempo estimado**: 10-12 horas
**Dependencias**: Pasos 01-07

## Qué hacemos

Exponer los comandos nuevos vía Tauri IPC y rediseñar la página Caché para mostrar el `CleanPlan` con sus 4 secciones (ready / blocked / permission / skipped).

## Por qué

El backend está hecho; falta la cara visible. Esta es la pieza que el usuario realmente toca.

## Archivos que tocamos

- `src-tauri/src/ipc/cache.rs` (modificado)
- `src-tauri/src/lib.rs` (registrar comandos)
- `src/api/client.ts` (wrappers TS)
- `src/api/types.ts` (ya hecho en paso 03)
- `src/features/cache-cleaner/cache-page.tsx` (rediseño completo)
- `src/features/cache-cleaner/components/*.tsx` (nuevos componentes)

## Cómo

### 1. Comandos Tauri

```rust
// src-tauri/src/ipc/cache.rs (añadir)

use crate::core::AppResult;
use crate::domain::cache;
use crate::models::cache::*;
use tauri::Emitter;

#[tauri::command]
pub async fn analyze_cache_locations(ids: Vec<String>) -> AppResult<CleanPlan> {
    cache::analyze_locations(&ids)
}

#[tauri::command]
pub async fn execute_clean_plan(
    app: tauri::AppHandle,
    plan: CleanPlan,
    opts: ExecutePlanOpts,
) -> AppResult<CleanReportV2> {
    let emit = move |evt: CleanProgressEvent| {
        let _ = app.emit("cache:progress", evt);
    };
    cache::execute_plan(plan, opts, emit).await
}

#[tauri::command]
pub async fn verify_clean(plan: CleanPlan, report: CleanReportV2) -> AppResult<VerifyReport> {
    cache::verify_after_clean(&plan, &report)
}

#[tauri::command]
pub async fn reset_uwp_app(package_family_name: String) -> AppResult<()> {
    cache::reset_uwp_app(&package_family_name).await
}
```

Registrar en `lib.rs::run` dentro de `invoke_handler`:

```rust
ipc::cache::analyze_cache_locations,
ipc::cache::execute_clean_plan,
ipc::cache::verify_clean,
ipc::cache::reset_uwp_app,
```

### 2. Wrappers TypeScript

```ts
// src/api/client.ts (añadir)

export const analyzeCacheLocations = (ids: string[]) =>
  invoke<CleanPlan>("analyze_cache_locations", { ids });

export const executeCleanPlan = (plan: CleanPlan, opts: ExecutePlanOpts) =>
  invoke<CleanReportV2>("execute_clean_plan", { plan, opts });

export const verifyClean = (plan: CleanPlan, report: CleanReportV2) =>
  invoke<VerifyReport>("verify_clean", { plan, report });

export const resetUwpApp = (packageFamilyName: string) =>
  invoke<void>("reset_uwp_app", { packageFamilyName });
```

### 3. Página Caché — estructura

```tsx
// src/features/cache-cleaner/cache-page.tsx

import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import {
  listCacheLocations,
  analyzeCacheLocations,
  executeCleanPlan,
  verifyClean,
} from "../../api";
import { toast } from "../../lib/toast";
import { ReadySection } from "./components/ready-section";
import { BlockedSection } from "./components/blocked-section";
import { PermissionSection } from "./components/permission-section";
import { SkippedSection } from "./components/skipped-section";
import { useTauriEvent } from "../../hooks/use-tauri-event";

export function CachePage() {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [plan, setPlan] = useState<CleanPlan | null>(null);

  const { data: catalog = [] } = useQuery({
    queryKey: ["cache-catalog"],
    queryFn: listCacheLocations,
  });

  const analyzeMutation = useMutation({
    mutationFn: () => analyzeCacheLocations([...selectedIds]),
    onSuccess: (p) => setPlan(p),
    onError: (err) => toast.error("Error analizando ubicaciones", err),
  });

  const executeMutation = useMutation({
    mutationFn: ({ plan, opts }: { plan: CleanPlan; opts: ExecutePlanOpts }) =>
      executeCleanPlan(plan, opts).then(async (report) => {
        const verify = await verifyClean(plan, report);
        return { report, verify };
      }),
    onSuccess: ({ verify }) => {
      toast.success(`Liberados ${formatBytes(verify.totalActuallyFreed)}`, {
        description:
          verify.totalStillPresent > 0
            ? `${formatBytes(verify.totalStillPresent)} no se pudieron eliminar.`
            : undefined,
      });
      setPlan(null);
      setSelectedIds(new Set());
    },
    onError: (err) => toast.error("Error ejecutando limpieza", err),
  });

  useTauriEvent("cache:progress", (evt) => {
    // Actualizar UI con progreso fino (opcional fase 1)
  });

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <Header />
      {!plan ? (
        <SelectionView
          catalog={catalog}
          selectedIds={selectedIds}
          onChange={setSelectedIds}
          onAnalyze={() => analyzeMutation.mutate()}
          isAnalyzing={analyzeMutation.isPending}
        />
      ) : (
        <PlanView
          plan={plan}
          onExecute={(opts) => executeMutation.mutate({ plan, opts })}
          onCancel={() => setPlan(null)}
          isExecuting={executeMutation.isPending}
        />
      )}
    </div>
  );
}
```

### 4. Componente `PlanView` con 4 secciones

```tsx
// src/features/cache-cleaner/components/plan-view.tsx

interface PlanViewProps {
  plan: CleanPlan;
  onExecute: (opts: ExecutePlanOpts) => void;
  onCancel: () => void;
  isExecuting: boolean;
}

export function PlanView({ plan, onExecute, onCancel, isExecuting }: PlanViewProps) {
  const [autoClose, setAutoClose] = useState(false);
  const [scheduleReboot, setScheduleReboot] = useState(true);
  const [dryRun, setDryRun] = useState(false);

  return (
    <div className="flex flex-col gap-4">
      <PlanSummary plan={plan} />

      <ReadySection items={plan.ready} />

      {plan.blocked.length > 0 && (
        <BlockedSection
          items={plan.blocked}
          onCloseProcess={(pid) => /* close + re-analyze */}
        />
      )}

      {plan.permissionIssues.length > 0 && (
        <PermissionSection items={plan.permissionIssues} />
      )}

      {plan.skipped.length > 0 && (
        <SkippedSection items={plan.skipped} />
      )}

      <OptionsBar
        autoClose={autoClose}
        scheduleReboot={scheduleReboot}
        dryRun={dryRun}
        onChange={{ autoClose: setAutoClose, scheduleReboot: setScheduleReboot, dryRun: setDryRun }}
      />

      <div className="flex gap-2 mt-2">
        <Button variant="outline" onClick={onCancel} disabled={isExecuting}>
          Cancelar
        </Button>
        <Button
          variant="default"
          onClick={() => onExecute({
            planId: plan.planId,
            autoCloseBlocking: autoClose,
            scheduleBlockedForReboot: scheduleReboot,
            dryRun,
            createRestorePoint: true,
            timeoutPerLocationSecs: 60,
          })}
          disabled={isExecuting}
        >
          {isExecuting ? "Ejecutando..." : dryRun ? "Simular limpieza" : "Limpiar"}
        </Button>
      </div>
    </div>
  );
}
```

### 5. Componente `BlockedSection` (el más importante)

```tsx
// src/features/cache-cleaner/components/blocked-section.tsx

export function BlockedSection({ items }: { items: BlockedLocation[] }) {
  return (
    <details open className="border border-warning rounded-lg">
      <summary className="p-3 cursor-pointer flex items-center gap-2 bg-warning/10">
        <AlertTriangle className="h-5 w-5 text-warning" />
        <span className="font-medium">
          Bloqueadas por procesos ({items.length})
        </span>
        <span className="ml-auto text-sm text-muted-foreground">
          {formatBytes(items.reduce((s, i) => s + i.bytes, 0))}
        </span>
      </summary>
      <div className="divide-y divide-border">
        {items.map((b) => (
          <BlockedRow key={b.id} item={b} />
        ))}
      </div>
    </details>
  );
}

function BlockedRow({ item }: { item: BlockedLocation }) {
  const action = item.suggestedAction;
  return (
    <div className="p-3 flex items-center gap-3">
      <div className="flex-1 min-w-0">
        <div className="font-medium">{item.displayName}</div>
        <div className="text-xs text-muted-foreground truncate">{item.resolvedPath}</div>
        <div className="text-xs text-muted-foreground mt-1">
          Bloqueado por: {item.lockedBy.map(p => `${p.name} (PID ${p.pid})`).join(", ")}
        </div>
      </div>
      <div className="text-sm font-mono">{formatBytes(item.bytes)}</div>
      <ActionButton action={action} />
    </div>
  );
}

function ActionButton({ action }: { action: BlockedAction }) {
  if (action.kind === "closeProcess") {
    return <Button size="sm" variant="outline">Cerrar {action.processName}</Button>;
  }
  if (action.kind === "scheduleReboot") {
    return <Button size="sm" variant="outline">Programar reboot</Button>;
  }
  return <span className="text-xs text-muted-foreground">{action.reason}</span>;
}
```

### 6. Tests UI manuales

- Lanzar app con Spotify abierto → Caché → escanear → "Bloqueadas" debe listar Spotify.
- Click "Cerrar Spotify" → Spotify se cierra → re-analyze automático → ahora en "Listas para limpiar".
- "Limpiar" → toast con bytes liberados.
- Re-escanear → debe mostrar 0 B (verify_clean).

## Criterio de done

- [ ] Comandos `analyze_cache_locations`, `execute_clean_plan`, `verify_clean`, `reset_uwp_app` registrados.
- [ ] Wrappers TS funcionan sin errores TS.
- [ ] Página Cache muestra `SelectionView` → `PlanView` con 4 secciones.
- [ ] "Cerrar [proceso]" cierra proceso + re-analyze.
- [ ] "Limpiar" ejecuta y muestra toast con bytes reales liberados.
- [ ] Dry-run no toca nada pero muestra cuánto SE LIBERARÍA.
- [ ] Eventos `cache:progress` no rompen UI (aunque no se rendericen aún).

## Próximo paso

Pasar a `03-process-manager/` si no se hizo en paralelo, o a `04-debloat-catalog/`.
