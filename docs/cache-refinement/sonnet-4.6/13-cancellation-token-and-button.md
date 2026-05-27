# 13 — Cancelación de la limpieza (token E2E + botón)

> **Severidad:** 🟡 P1 — funcionalidad ausente. El usuario pidió
> explícitamente un botón de cancelación.
> **Modelo:** Sonnet 4.6.
> **Bloque:** 1 de 4 del Set C (quality of life).
> **Depende de:** docs 09-12 ya implementados (eventos v2 + consola).

## 1. Problema

Hoy, una vez que el usuario pulsa **Limpiar**, no hay manera de
parar. Si la limpieza tarda 10 minutos (caso real, ver doc 16) y el
usuario quiere abortarla, su única opción es **cerrar la app entera**
— lo que deja:

- El restore point ya creado pero no marcado como "asociado al
  cancel".
- Archivos a medio borrar (los que ya se eliminaron están perdidos;
  el resto sigue ahí).
- El audit log sin entrada de cierre.
- El `cache_background` con caché inválida pero sin re-scan.

## 2. Objetivo

Cancelación cooperativa **fina**:

- El usuario pulsa **Cancelar** en la consola.
- La consola pasa a estado `cancelling` (botón con spinner, deshabilitado).
- El backend deja de procesar nuevos archivos / nuevas ubicaciones.
- El archivo actual termina o se aborta limpio (no se rompe a medias).
- Se emite `cache:summary` con `success: false`, `cancelled: true`,
  totales actuales y duración.
- La UI muestra el resumen con copy distinto ("Cancelado a petición").

No buscamos cancelación instantánea — un par de segundos de gracia es
aceptable y mucho más seguro que `process::exit`.

## 3. Cambios en el backend

### 3.1 Modelo

`src-tauri/src/models/cache.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanSummaryPayload {
    // ... campos existentes ...
    /// NUEVO. true si la limpieza terminó por cancelación del usuario.
    pub cancelled: bool,
}
```

(Setear `false` por defecto en `Default::default()`; en
`build_summary()` setear según el tracker.)

### 3.2 Registro global de tokens activos

Como el `execute_clean_plan` se invoca desde Tauri como comando async,
y `cancel_clean_plan` se invoca como **otro** comando, hay que tener
un canal entre ambos. La solución estándar:

`src-tauri/src/domain/cache_cancellation.rs` (NUEVO):

```rust
//! Registro de tokens de cancelación activos para limpiezas en curso.
//! Permite a un comando IPC señalar a otro (que ya está ejecutando un
//! plan) que aborte cooperativamente.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

static TOKENS: Lazy<Mutex<HashMap<String, CancellationToken>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Crea y registra un nuevo token para `run_id`. Si ya existía uno con
/// ese id, lo reemplaza (no debería ocurrir si los run_id son UUID v4).
pub fn register(run_id: &str) -> CancellationToken {
    let token = CancellationToken::new();
    if let Ok(mut guard) = TOKENS.lock() {
        guard.insert(run_id.to_string(), token.clone());
    }
    token
}

/// Cancela el token registrado para `run_id` si existe.
/// Devuelve true si encontró y canceló; false si no había token.
pub fn cancel(run_id: &str) -> bool {
    let Ok(mut guard) = TOKENS.lock() else { return false; };
    if let Some(token) = guard.remove(run_id) {
        token.cancel();
        true
    } else {
        false
    }
}

/// Elimina el token tras finalizar la limpieza (limpio, fallido o
/// cancelado). No-op si ya fue retirado por cancel().
pub fn deregister(run_id: &str) {
    if let Ok(mut guard) = TOKENS.lock() {
        guard.remove(run_id);
    }
}
```

Añadir al `Cargo.toml`:

```toml
tokio-util = { version = "0.7", features = ["rt"] }
```

(`tokio_util::sync::CancellationToken` es el estándar de facto.)

### 3.3 Propagación en `execute_plan`

`src-tauri/src/domain/cache.rs::execute_plan` recibe el token y lo
chequea en cada punto de gracia:

```rust
pub async fn execute_plan<E: CleanEmitter>(
    plan: CleanPlan,
    opts: ExecutePlanOpts,
    emitter: &mut E,
    cancel: CancellationToken,    // NUEVO parámetro
) -> AppResult<CleanReportV2> {
    let run_id = Uuid::new_v4().to_string();
    let mut tracker = ProgressTracker::new(&run_id, ...);

    macro_rules! check_cancel {
        () => {
            if cancel.is_cancelled() {
                tracker.cancelled = true;
                tracker.set_phase(emitter, CleanPhase::Failed);
                let summary = build_summary_cancelled(&tracker, &per_location, &plan);
                emitter.summary(summary);
                return Ok(build_report_partial(&tracker, &plan, per_location));
            }
        };
    }

    // ... fase Preparing ...
    check_cancel!();

    // Restore point — punto de cancelación tras crear, antes de
    // empezar destructivo.
    let restore_point_seq = if !opts.dry_run && opts.create_restore_point {
        tracker.set_phase(emitter, CleanPhase::CreatingRestorePoint);
        crate::platform::restore_point::create(&desc, 12, true).ok()
    } else {
        None
    };
    check_cancel!();

    // Cierre de procesos — comprobar antes de cada cierre.
    if opts.auto_close_blocking {
        tracker.set_phase(emitter, CleanPhase::ClosingProcesses);
        for blocked in &plan.blocked {
            check_cancel!();
            // ... close_gracefully ...
        }
    }

    // Limpieza — comprobar entre ubicaciones Y dentro del walk.
    tracker.set_phase(emitter, CleanPhase::Cleaning);
    let mut per_location: Vec<LocationResult> = Vec::new();
    for (idx, ready_loc) in plan.ready.iter().enumerate() {
        check_cancel!();
        tracker.current_index = (idx + 1) as u32;
        // execute_one_location recibe el token también.
        let result = execute_one_location(
            ready_loc,
            &opts,
            &mut tracker,
            emitter,
            &cancel,
        ).await;
        per_location.push(result);
    }

    // ... resto idéntico ...

    Ok(report)
}
```

Y en `execute_one_location` + `walk_and_delete`:

```rust
async fn walk_and_delete(
    path: &std::path::Path,
    tracker: &mut ProgressTracker,
    emitter: &mut impl CleanEmitter,
    cancel: &CancellationToken,
    /* ... otros params ... */
) -> AppResult<()> {
    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(it) => it.filter_map(|r| r.ok()).collect(),
        Err(e) => { return Err(/*...*/); }
    };

    for entry in entries {
        // Cancelación dentro del loop — granularidad por archivo.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        // ... resto del walk ...
    }
    Ok(())
}
```

Añadir variante a `AppError`:

```rust
#[derive(Debug, thiserror::Error, Serialize)]
pub enum AppError {
    // ... existentes ...
    #[error("Cancelado por el usuario")]
    Cancelled,
}
```

### 3.4 Build del summary cancelado

```rust
fn build_summary_cancelled(
    tracker: &ProgressTracker,
    per_location: &[LocationResult],
    plan: &CleanPlan,
) -> CleanSummaryPayload {
    let mut s = build_summary(tracker, per_location, plan);
    s.success = false;
    s.cancelled = true;
    s
}
```

### 3.5 IPC

`src-tauri/src/ipc/cache.rs`:

```rust
#[tauri::command]
pub async fn execute_clean_plan(
    app: tauri::AppHandle,
    plan: CleanPlan,
    opts: ExecutePlanOpts,
) -> AppResult<CleanReportV2> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let cancel = crate::domain::cache_cancellation::register(&run_id);

    // Emitir un evento "started" con el run_id para que el frontend
    // sepa qué runId está corriendo (para luego cancelarlo).
    let _ = app.emit("cache:started", serde_json::json!({ "runId": &run_id }));

    let mut emitter = TauriCleanEmitter { app, run_id: run_id.clone() };
    let result = domain::cache::execute_plan(plan, opts, &mut emitter, cancel).await;

    crate::domain::cache_cancellation::deregister(&run_id);
    result
}

#[tauri::command]
pub async fn cancel_clean_plan(run_id: String) -> AppResult<bool> {
    Ok(crate::domain::cache_cancellation::cancel(&run_id))
}
```

Y registrar `cancel_clean_plan` en `src-tauri/src/lib.rs` junto al
resto.

**Nota:** el `run_id` ahora se genera en el IPC, no en `execute_plan`.
Pasarlo como argumento al tracker:

```rust
// En execute_plan, recibir run_id como param:
pub async fn execute_plan<E: CleanEmitter>(
    run_id: &str,
    plan: CleanPlan,
    opts: ExecutePlanOpts,
    emitter: &mut E,
    cancel: CancellationToken,
) -> AppResult<CleanReportV2> {
    let mut tracker = ProgressTracker::new(run_id, /*...*/);
    // ...
}
```

## 4. Cambios en el frontend

### 4.1 API client

`src/api/client.ts` — añadir:

```ts
export const cancelCleanPlan = (runId: string) =>
  invoke<boolean>("cancel_clean_plan", { runId });
```

### 4.2 Hook `useCleanStream` — capturar `runId`

`src/features/cache-cleaner/use-clean-stream.ts`:

```ts
type Action =
  | { type: "start"; runId: string }
  | { type: "started_from_backend"; runId: string }   // NUEVO
  | /* resto igual */;

function reducer(state: State, action: Action): State {
  switch (action.type) {
    // ...
    case "started_from_backend":
      // El backend nos dice qué runId está corriendo.
      return { ...state, runId: action.runId };
    // ...
  }
}

export function useCleanStream() {
  const [state, dispatch] = useReducer(reducer, /*...*/);
  // ... oyentes existentes ...
  useTauriEvent("cache:started", (p: { runId: string }) =>
    dispatch({ type: "started_from_backend", runId: p.runId })
  );
  // ...
  return { ...state, start, reset };
}
```

Y exportar el `runId` para que la consola pueda cancelarlo.

### 4.3 Estado `cancelling` en el stream

```ts
export type CleanState =
  | { kind: "idle" }
  | { kind: "running"; progress: CleanProgressV2Payload; phase: CleanPhasePayload["phase"] }
  | { kind: "cancelling"; progress: CleanProgressV2Payload }   // NUEVO
  | { kind: "complete"; summary: CleanSummaryPayload }
  | { kind: "failed"; summary: CleanSummaryPayload };
```

Y acción + reducer:

```ts
| { type: "cancelling" }

case "cancelling":
  if (state.status.kind !== "running") return state;
  return { ...state, status: { kind: "cancelling", progress: state.status.progress } };
```

El backend cuando reciba `cancel_clean_plan` no emite nada nuevo
inmediatamente — sólo cuando los chequeos `cancel.is_cancelled()` se
disparen. Pero la UI debe responder al instante; por eso el reducer
local tiene un estado intermedio.

### 4.4 Botón Cancelar en `CleanConsole`

`src/features/cache-cleaner/clean-console.tsx`:

```tsx
import { cancelCleanPlan } from "../../api";

export function CleanConsole({ open, onClose, isRunning }: Props) {
  const { status, log, runId, dispatch } = useCleanStream();
  // ...

  const [cancelling, setCancelling] = useState(false);

  async function handleCancel() {
    if (!runId || cancelling) return;
    setCancelling(true);
    dispatch?.({ type: "cancelling" });
    try {
      await cancelCleanPlan(runId);
    } catch (e) {
      console.error("Error cancelando", e);
      setCancelling(false);
    }
    // Si la cancelación tiene éxito, el evento cache:summary
    // llegará con success=false, cancelled=true; el reducer lo
    // mueve a "failed" y el botón "Listo" se habilita.
  }

  useEffect(() => {
    if (status.kind === "complete" || status.kind === "failed") {
      setCancelling(false);
    }
  }, [status.kind]);

  const showCancel =
    (status.kind === "running" || status.kind === "cancelling") && !!runId;

  return (
    // ... mismo wrapper ...
    <div className="flex justify-end gap-2">
      {isDone && (
        <Button variant="ghost" onClick={() => setShowFullLog((s) => !s)}>
          {showFullLog ? "Ocultar log" : "Ver log completo"}
        </Button>
      )}
      {showCancel && (
        <Button
          variant="outline"
          onClick={handleCancel}
          disabled={cancelling || status.kind === "cancelling"}
        >
          {cancelling || status.kind === "cancelling" ? (
            <>
              <Loader2 className="h-4 w-4 mr-1.5 animate-spin" />
              Cancelando…
            </>
          ) : (
            "Cancelar"
          )}
        </Button>
      )}
      <Button
        onClick={onClose}
        disabled={isRunning && !cancelling}
        variant={status.kind === "complete" ? "default" : "outline"}
      >
        {isRunning ? "Limpiando…" : "Listo"}
      </Button>
    </div>
  );
}
```

(Hay que exportar `dispatch` desde `useCleanStream` o crear una acción
pública `markCancelling()` — preferible la segunda por encapsulación.)

### 4.5 `CleanSummaryHero` con caso `cancelled`

`src/features/cache-cleaner/clean-summary-hero.tsx`:

```tsx
const isCancelled = summary.cancelled;
const success = summary.success && !isCancelled && summary.totalFilesFailed === 0;

const Icon = isCancelled ? CircleSlash : success ? CheckCircle2 : AlertTriangle;
const title = isCancelled
  ? "Cancelado"
  : success
  ? "¡Hecho!"
  : "Terminó con avisos";
const subtitle = isCancelled
  ? `Detuviste la limpieza tras ${formatDuration(durationSecs)}. Liberaste ${formatBytes(summary.totalBytesFreed)} antes de cancelar.`
  : /* copy actual */;
```

Importar `CircleSlash` de `lucide-react`. Color del icono cancelado:
`text-ink-tertiary` (gris neutro, no rojo — no es un error).

## 5. Tests

`src-tauri/src/domain/cache.rs::tests`:

```rust
#[tokio::test]
async fn execute_plan_respects_cancellation() {
    use tokio_util::sync::CancellationToken;

    let plan = CleanPlan {
        plan_id: "p".into(),
        generated_at: "".into(),
        ready: vec![/* 5 ubicaciones con paths que existen */],
        blocked: vec![],
        permission_issues: vec![],
        skipped: vec![],
        total_estimated_bytes: 0,
        total_blocked_bytes: 0,
    };

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    // Cancelar tras 100 ms.
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let mut em = MockEmitter::default();
    let result = execute_plan("test-run", plan, ExecutePlanOpts::default(), &mut em, cancel).await;
    assert!(result.is_ok());

    let summary = em.summary.expect("summary debe emitirse");
    assert!(summary.cancelled);
    assert!(!summary.success);
}

#[test]
fn cancellation_registry_returns_false_for_unknown_runid() {
    assert!(!crate::domain::cache_cancellation::cancel("no-existe"));
}

#[test]
fn cancellation_registry_cancels_active_token() {
    let token = crate::domain::cache_cancellation::register("active-run");
    assert!(!token.is_cancelled());
    assert!(crate::domain::cache_cancellation::cancel("active-run"));
    assert!(token.is_cancelled());
}
```

## 6. Criterio de done

- [ ] Comando `cancel_clean_plan(runId)` registrado y funcional.
- [ ] `execute_plan` chequea `cancel.is_cancelled()` en al menos: entre
      ubicaciones, dentro del walk de archivos, antes de programar
      reboot, antes de verificar.
- [ ] Pulsar el botón **Cancelar** cambia el estado a `cancelling` con
      spinner < 100 ms.
- [ ] En menos de ~3 s, el backend emite `cache:summary` con
      `cancelled: true`.
- [ ] `CleanSummaryHero` muestra el caso cancelado con icono `CircleSlash`
      y copy diferenciado.
- [ ] El restore point creado se **conserva** (el usuario puede
      revertir desde la pestaña Restauración).
- [ ] Tests Rust pasan: `execute_plan_respects_cancellation`,
      `cancellation_registry_returns_false_for_unknown_runid`,
      `cancellation_registry_cancels_active_token`.

## 7. Riesgos

- **Carrera entre `cancel_clean_plan` y el final natural**: si el
  usuario cancela justo cuando la limpieza está terminando,
  `deregister()` y `cancel()` pueden cruzarse. `HashMap::remove` es
  atómico bajo el `Mutex`. El peor caso es: cancel llega después de
  deregister → no encuentra el token → devuelve `false` → la UI ya
  recibió `cache:summary` con `success=true`. Coherente.
- **Cancelación durante restore point**: `SRSetRestorePointW` no se
  puede abortar — bloquea el thread. Mitigación: chequeo **antes** y
  **después**, pero no durante. Si tarda 30 s, el usuario lo verá
  como espera. El doc 16 (perf) puede sacar el restore point a un
  worker separado en futuro.
- **`tokio_util::sync::CancellationToken` cloneable**: `clone()` no
  copia el token, sólo el handle — todos los clones se cancelan a la
  vez. Comportamiento correcto para nuestra propagación.
- **Cancel sin runId conocido**: si el evento `cache:started` se
  pierde por timing, `runId` queda `null` en el frontend → el botón
  Cancelar no aparece. Mitigación: fallback con `setTimeout(300ms,
  forzarRunIdDelPrimerProgress)` o aceptar runId del primer evento
  `cache:progress-v2`.
