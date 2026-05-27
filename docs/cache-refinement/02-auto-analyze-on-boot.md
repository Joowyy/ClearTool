# 02 — Auto-análisis en background al iniciar la app

> **Severidad:** 🟡 P1 — el usuario quiere "que no tenga que analizar
> antes de borrar; que se analice todo según se inicia la app en segundo
> plano".

## 1. Problema

Flujo actual:

1. Usuario abre ClearTool → ve `SelectionView` con el catálogo de cachés.
2. Selecciona ubicaciones (o usa el preset "Total").
3. Pulsa **Analizar** → espera 1-10 segundos (depende del tamaño).
4. Aparece `PlanView` con el plan.
5. Pulsa **Limpiar**.

Son 4 clicks + una espera. Para un usuario que se considera "tonto",
es mucho. Además, si vuelve a entrar a la pestaña, el plan se pierde y
hay que re-analizar.

## 2. Causa raíz

- `src/features/cache-cleaner/cache-page.tsx:651-674` — la página renderiza
  `SelectionView` o `PlanView` según `plan === null`.
- `analyzeMutation` se ejecuta sólo al pulsar el botón.
- No hay caché del plan entre navegaciones de pestaña.

## 3. Fix propuesto

### 3.1 Worker de background al iniciar la app

Crear `src-tauri/src/domain/cache_background.rs`:

```rust
//! Análisis perezoso del catálogo de cachés en background.
//! Se dispara al arrancar la app y cachea el plan en memoria
//! (no en disco — no merece la pena la complejidad).

use crate::core::AppResult;
use crate::models::cache::CleanPlan;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CachedPlan {
    pub plan: CleanPlan,
    pub computed_at: Instant,
    pub all_ids: Vec<String>,
}

static CACHED: Lazy<RwLock<Option<CachedPlan>>> = Lazy::new(|| RwLock::new(None));

/// TTL del plan en memoria. Pasado este tiempo se considera obsoleto
/// y se recalcula al siguiente request.
const TTL: Duration = Duration::from_secs(90);

/// Dispara un escaneo en background con TODOS los IDs del catálogo.
/// Se llama al `setup` de Tauri, sin bloquear el splash de la app.
pub fn spawn_initial_scan() {
    tokio::spawn(async move {
        log::info!("cache_background: iniciando primer análisis perezoso");
        if let Err(e) = recompute_and_store().await {
            log::warn!("cache_background: primer análisis falló: {}", e);
        }
    });
}

pub fn get_cached() -> Option<CachedPlan> {
    let guard = CACHED.read().ok()?;
    let cached = guard.as_ref()?.clone();
    if cached.computed_at.elapsed() > TTL {
        return None;
    }
    Some(cached)
}

pub fn get_cached_stale_ok() -> Option<CachedPlan> {
    CACHED.read().ok()?.as_ref().cloned()
}

pub async fn recompute_and_store() -> AppResult<CleanPlan> {
    let catalog = crate::domain::catalog::load_cache_locations()?;
    let ids: Vec<String> = catalog.iter().map(|l| l.id.clone()).collect();
    let plan = crate::domain::cache::analyze_locations(&ids)?;
    let cached = CachedPlan {
        plan: plan.clone(),
        computed_at: Instant::now(),
        all_ids: ids,
    };
    if let Ok(mut guard) = CACHED.write() {
        *guard = Some(cached);
    }
    Ok(plan)
}

pub fn invalidate() {
    if let Ok(mut guard) = CACHED.write() {
        *guard = None;
    }
}
```

### 3.2 Disparar el escaneo en `setup`

`src-tauri/src/lib.rs`, dentro del `tauri::Builder::default().setup(|app| { ... })`:

```rust
.setup(|app| {
    // ... resto del setup ...

    // Análisis perezoso del catálogo de cachés al arrancar.
    // No bloquea la UI; el resultado se cachea en memoria.
    crate::domain::cache_background::spawn_initial_scan();

    Ok(())
})
```

### 3.3 Nuevo comando IPC `get_cache_plan_warm`

`src-tauri/src/ipc/cache.rs`:

```rust
#[tauri::command]
pub async fn get_cache_plan_warm() -> Result<Option<CleanPlan>, AppError> {
    Ok(crate::domain::cache_background::get_cached().map(|c| c.plan))
}

#[tauri::command]
pub async fn refresh_cache_plan() -> Result<CleanPlan, AppError> {
    crate::domain::cache_background::recompute_and_store().await
}
```

Registrar los handlers en `src-tauri/src/lib.rs` junto al resto.

### 3.4 Invalidar el cache tras limpieza

En `execute_plan` (`src-tauri/src/domain/cache.rs`), al final de la
función — después de obtener el `CleanReportV2`:

```rust
// El plan cacheado deja de ser válido tras una limpieza real.
if !opts.dry_run {
    crate::domain::cache_background::invalidate();
    // Y disparar un re-análisis en background para el próximo entry.
    tokio::spawn(async {
        let _ = crate::domain::cache_background::recompute_and_store().await;
    });
}
```

### 3.5 Frontend — entrar directo a PlanView si hay cache caliente

`src/features/cache-cleaner/cache-page.tsx`:

```tsx
export function CachePage() {
  // ... estado existente ...

  // Intentar cargar el plan caliente al montar.
  const warmPlanQuery = useQuery({
    queryKey: ["cache-plan-warm"],
    queryFn: getCachePlanWarm,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  // Si llega un plan caliente y todavía no hay plan en estado, usarlo.
  useEffect(() => {
    if (warmPlanQuery.data && !plan) {
      setPlan(warmPlanQuery.data);
      // Seleccionar todo lo que el plan tocó (para "Re-analizar" y "Limpiar"
      // operen sobre lo mismo).
      const allIds = new Set<string>([
        ...warmPlanQuery.data.ready.map(r => r.id),
        ...warmPlanQuery.data.blocked.map(b => b.id),
        ...warmPlanQuery.data.permissionIssues.map(p => p.id),
      ]);
      setSelectedIds(allIds);
    }
  }, [warmPlanQuery.data, plan]);

  // ... resto ...
}
```

Y añadir a `src/api/client.ts`:

```ts
export const getCachePlanWarm = () =>
  invoke<CleanPlan | null>("get_cache_plan_warm");

export const refreshCachePlan = () =>
  invoke<CleanPlan>("refresh_cache_plan");
```

### 3.6 Refresco automático cada 90s mientras estás en la pestaña

Reemplazar el botón "Re-analizar" por un refresh silencioso:

```tsx
useEffect(() => {
  const id = setInterval(() => {
    if (!executeMutation.isPending) {
      void refreshCachePlan().then(p => {
        // Sólo actualiza si los totales cambiaron para evitar parpadeos.
        if (p.totalEstimatedBytes !== plan?.totalEstimatedBytes) {
          setPlan(p);
        }
      });
    }
  }, 90_000);
  return () => clearInterval(id);
}, [plan, executeMutation.isPending]);
```

### 3.7 Eliminar el botón "Re-analizar" visible — convertirlo en sutil

El usuario quiere que la app "no moleste". El botón "Re-analizar" en la
esquina superior es ruido. Mover a un icono `RefreshCw` pequeño en la
cabecera con tooltip "Forzar nuevo análisis (Ctrl+R)" y mantener el
atajo de teclado.

## 4. Criterio de done

- [ ] Al abrir ClearTool por primera vez tras login, en menos de 5
      segundos el módulo Caché ya muestra `PlanView` (no `SelectionView`)
      con los totales calculados.
- [ ] Al navegar a otra pestaña y volver, NO hay re-análisis (mismo plan
      cacheado mientras esté dentro del TTL).
- [ ] Tras pulsar "Limpiar", el plan se invalida y al volver entra ya
      con valores recalculados.
- [ ] El botón "Re-analizar" deja de ser un CTA grande.

## 5. Riesgos / efectos secundarios

- **Coste de CPU al boot**: el primer análisis recorre 50+ ubicaciones
  con `scan_path_stats` recursivo. En SSD moderno tarda 1-3s, pero en
  HDD legacy puede subir a 10-15s. **Mitigación**: el escaneo va en un
  worker tokio, no bloquea el frontend. Si tarda, el usuario ve
  `SelectionView` mientras se computa y se actualiza solo cuando termina.
- **TTL desincronizado**: si el usuario borra cosas manualmente desde
  Explorer mientras ClearTool está abierto, el plan cacheado queda
  obsoleto hasta el TTL. **Mitigación**: el refresh cada 90s lo corrige.
- **Memory footprint**: un `CleanPlan` con 50 ubicaciones pesa <20 KB.
  Despreciable.
