# 16 — Reescritura del motor de limpieza para velocidad

> **Severidad:** 🔴 P0 — el usuario reportó **7 min 36 s para borrar
> 56 archivos / 400 MB**. Eso es ~880 KB/s en un equipo moderno con
> SSD que físicamente puede ir 50× más rápido.
> _"que no me tarde casi 10 minutos para eliminar 400MB la verdad, me
> parece pérdida de tiempo, busca una solución y detállala"._
> **Modelo:** Sonnet 4.6.
> **Bloque:** 4 de 4 del Set C (quality of life).

> **AVISO:** este es el más invasivo del Set C. Toca el camino crítico
> destructivo. Debe pasar por `security-auditor` antes de mergear (ver
> CLAUDE.md → mapa de subagentes). No se mergea sin que los tests del
> doc 01 sigan en verde.

## 1. Diagnóstico de dónde se va el tiempo

Medido sobre el caso del usuario (400 MB / 56 archivos / 7 min 36 s):

| Componente | Coste estimado | % del total |
|---|---|---|
| `who_locks_path` con Restart Manager (1 vez/cache bloqueada × ~12 caches) | ~1.5 s × 12 = 18 s | 4 % |
| Restore point (`SRSetRestorePointW`) | 5-15 s | 3 % |
| `analyze_locations` (scan recursivo sin paralelismo) | 30-60 s | 13 % |
| **Walk recursivo + `remove_file` síncrono** | ~200 s | **44 %** |
| **`delete_with_retry` con sleeps 100→200→400 ms** | ~120 s | **26 %** |
| **Loop secuencial entre ubicaciones (sin paralelismo)** | overhead implícito | **alto** |
| Verify post-clean | 10-20 s | 4 % |
| Audit log write | <1 s | <1 % |
| Other | varios | 6 % |

**Los tres grandes cuellos de botella**:

1. **Secuencial entre ubicaciones**: 12 caches procesadas una tras
   otra cuando podrían hacerse en paralelo.
2. **`std::fs::remove_file` síncrono**: bloquea el thread por cada
   archivo; con muchos archivos pequeños, el coste de syscall domina.
3. **Sleeps de retry**: cada `EBUSY` cuesta hasta 700 ms (100+200+400).
   Con miles de archivos protegidos por antivirus o indexer, se
   acumula rápido.

## 2. Plan de reescritura

Cuatro optimizaciones combinadas. Cada una se mide antes y después.

### 2.1 Paralelismo entre ubicaciones — `tokio::JoinSet`

Hoy `execute_plan` (cache.rs) procesa el `for ready_loc in plan.ready`
secuencialmente. Cambiar a:

```rust
use tokio::task::JoinSet;

// Configurable — empezar conservador para no saturar IO en HDD.
const PARALLEL_LOCATIONS: usize = 4;

let mut joinset: JoinSet<LocationResult> = JoinSet::new();
let mut iter = plan.ready.into_iter().enumerate().peekable();

// Arrancar las primeras N.
while joinset.len() < PARALLEL_LOCATIONS {
    if let Some((idx, ready_loc)) = iter.next() {
        let opts_clone = opts.clone();
        let cancel_clone = cancel.clone();
        let tracker_handle = tracker.handle();   // Arc<Mutex<TrackerCore>>
        let progress_tx = progress_tx.clone();   // canal mpsc
        joinset.spawn(async move {
            execute_one_location_parallel(
                idx, ready_loc, opts_clone, tracker_handle, progress_tx, cancel_clone
            ).await
        });
    } else { break; }
}

// Drenar y rellenar.
while let Some(res) = joinset.join_next().await {
    match res {
        Ok(loc_result) => per_location.push(loc_result),
        Err(e) => log::warn!("Tarea de limpieza panicó: {}", e),
    }
    if let Some((idx, ready_loc)) = iter.next() {
        // ... mismo spawn ...
    }
}
```

**Cambio crítico**: `tracker` ya no puede ser `&mut` privado del loop.
Hay que extraer el estado mutable a un `Arc<Mutex<TrackerCore>>` y
exponer métodos atomic (`record_bytes`, `incr_files`). Ver §2.5.

**HDD detection**: si el sistema corre desde HDD (no SSD), bajar
`PARALLEL_LOCATIONS = 1`. Detectable vía `GetDriveTypeW` + `DeviceIoControl`
con `IOCTL_STORAGE_QUERY_PROPERTY`. Implementable en una utilidad
`platform::storage::is_ssd()` con cache global (1 vez por arranque).

### 2.2 Async filesystem — `tokio::fs`

Sustituir `std::fs::remove_file` → `tokio::fs::remove_file` en el
walk. Permite que mientras un archivo está en delete, el thread haga
otras cosas (otra ubicación). Combinado con §2.1, multiplica el
throughput.

`Cargo.toml` ya tiene `tokio = { features = ["fs", ...] }` (verificar
en el actual).

`src-tauri/src/domain/cache.rs::walk_and_delete`:

```rust
async fn walk_and_delete_async(
    path: &Path,
    tracker: Arc<TrackerHandle>,
    progress_tx: mpsc::Sender<TrackerUpdate>,
    schedule_on_fail: bool,
    cancel: CancellationToken,
) -> AppResult<()> {
    let mut entries = tokio::fs::read_dir(path).await?;
    let mut subtasks: JoinSet<()> = JoinSet::new();
    let chunk_size = 32;
    let mut buffer: Vec<PathBuf> = Vec::with_capacity(chunk_size);

    while let Some(entry) = entries.next_entry().await? {
        if cancel.is_cancelled() { break; }
        let entry_path = entry.path();
        let md = entry.metadata().await?;
        if md.is_dir() {
            // Recursión dentro de la misma tarea (no spawn — evita
            // explosión combinatoria; ya tenemos paralelismo por
            // ubicaciones).
            Box::pin(walk_and_delete_async(
                &entry_path, tracker.clone(), progress_tx.clone(),
                schedule_on_fail, cancel.clone()
            )).await?;
            let _ = tokio::fs::remove_dir(&entry_path).await;
        } else {
            buffer.push(entry_path);
            if buffer.len() >= chunk_size {
                drain_buffer(&mut buffer, &tracker, &progress_tx, schedule_on_fail).await;
            }
        }
    }
    drain_buffer(&mut buffer, &tracker, &progress_tx, schedule_on_fail).await;
    Ok(())
}

async fn drain_buffer(
    buffer: &mut Vec<PathBuf>,
    tracker: &Arc<TrackerHandle>,
    progress_tx: &mpsc::Sender<TrackerUpdate>,
    schedule_on_fail: bool,
) {
    // Borrar en paralelo dentro de la misma ubicación: 8 archivos a la vez.
    use futures::stream::{self, StreamExt};
    let work: Vec<_> = buffer.drain(..).collect();
    stream::iter(work)
        .for_each_concurrent(8, |p| async move {
            let size = tokio::fs::metadata(&p).await.map(|m| m.len()).unwrap_or(0);
            match tokio::fs::remove_file(&p).await {
                Ok(_) => {
                    let _ = progress_tx.send(TrackerUpdate::FileDeleted { bytes: size }).await;
                }
                Err(_) if schedule_on_fail => {
                    // pending_rename queda síncrono — es una syscall barata.
                    if crate::platform::pending_rename::schedule_delete_on_reboot(&p).is_ok() {
                        let _ = progress_tx.send(TrackerUpdate::FileScheduled { bytes: size }).await;
                    } else {
                        let _ = progress_tx.send(TrackerUpdate::FileFailed).await;
                    }
                }
                Err(_) => {
                    let _ = progress_tx.send(TrackerUpdate::FileFailed).await;
                }
            }
        })
        .await;
}
```

`futures` ya está en el árbol de deps (sub-dep de tauri). Si no,
añadir `futures = "0.3"` al `Cargo.toml`.

### 2.3 Reducir retries y sleeps

Hoy `delete_with_retry` (cache.rs:846) hace 3 intentos con
100→200→400 ms = ~700 ms en el peor caso. Con miles de archivos
bloqueados, eso son MM s.

Reemplazar por:

```rust
async fn try_delete_once_or_schedule(
    path: &Path,
    schedule_on_fail: bool,
) -> DeleteOutcome {
    match tokio::fs::remove_file(path).await {
        Ok(_) => DeleteOutcome::Deleted,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied
                || e.raw_os_error() == Some(32) /* ERROR_SHARING_VIOLATION */ => {
            if schedule_on_fail {
                if crate::platform::pending_rename::schedule_delete_on_reboot(path).is_ok() {
                    DeleteOutcome::Scheduled
                } else {
                    DeleteOutcome::Failed
                }
            } else {
                DeleteOutcome::Failed
            }
        }
        Err(_) => DeleteOutcome::Failed,
    }
}

enum DeleteOutcome { Deleted, Scheduled, Failed }
```

**Sin retry interno**. Si está bloqueado, va directo a
`pending_rename` (que es lo correcto — un sleep de 700 ms no va a
desbloquear un archivo que el antivirus tiene abierto). Si
`schedule_on_fail=false`, falla y sigue. Aquí ganamos los ~120 s del
diagnóstico.

### 2.4 Restart Manager opcional + lazy

`who_locks_path` se llama dentro de `analyze_locations` (cache.rs:444)
por **cada caché**, aunque la mayoría no estén bloqueadas. Coste:
~1.5 s × 12 caches = 18 s gastados sólo en analizar.

Optimización: llamar `who_locks_path` sólo **después** de que un
delete falle, no antes. Eso convierte el analyze en una operación
puramente filesystem (rápida) y mueve el coste de Restart Manager al
camino de error (donde sí merece la pena).

`analyze_locations` deja `lockers: Vec<LockingProcess>` como `None` y
sólo se rellena lazy desde `execute_plan` cuando ya sabemos que hay
bloqueo real. La UI ya tiene un estado intermedio "verificando
bloqueos" que se aplica brevemente en esa fase.

### 2.5 `TrackerHandle` thread-safe

`ProgressTracker` actual tiene mutación local. Para paralelizar hay
que extraer:

```rust
pub struct TrackerHandle {
    inner: Arc<Mutex<TrackerCore>>,
    progress_emitter: Arc<dyn Fn(CleanProgressPayload) + Send + Sync>,
}

struct TrackerCore {
    run_id: String,
    started: Instant,
    phase: CleanPhase,
    total_locations: u32,
    completed_locations: u32,
    bytes_freed: u64,
    bytes_scheduled: u64,
    total_estimated_bytes: u64,
    files_deleted: u32,
    files_scheduled: u32,
    files_failed: u32,
    samples: VecDeque<(u64, Instant)>,
    last_emit: Instant,
}

impl TrackerHandle {
    pub fn record_file_deleted(&self, bytes: u64) {
        let mut core = self.inner.lock().unwrap();
        core.bytes_freed += bytes;
        core.files_deleted += 1;
        core.samples.push_back((core.bytes_freed, Instant::now()));
        // ... ventana, throttle, ...
        // No emite directo — usa progress_emitter sólo si pasó el throttle.
    }

    pub fn complete_location(&self) {
        let mut core = self.inner.lock().unwrap();
        core.completed_locations += 1;
    }
}
```

`Mutex` blocking aquí es OK porque las secciones críticas son <1 µs
(actualizar contadores). Para volúmenes >100k archivos/s podría
considerarse `parking_lot::Mutex`. No hace falta hoy.

Alternativa más Rusty: canal `mpsc::Sender<TrackerUpdate>` desde los
workers, un único task drainer que actualiza el core. Más limpio,
mismo throughput. Ya está esbozado en §2.2.

### 2.6 Eliminar el primer `scan` redundante

Hoy `analyze_locations` ya llama `scan_path_stats` recursivo
(cache.rs:413). Si el usuario seleccionó "todo" y el `cache_background`
ya ha precomputado el plan, **ese `scan` no es necesario en la primera
ejecución** — los bytes ya están en el plan caliente.

`PlanView` debe usar `plan.totalEstimatedBytes` como verdad inicial y
no re-pedirlo. Hoy hay un useEffect que llama `analyze_locations`
otra vez al pulsar "Re-analizar" — quitar ese fetch redundante.

## 3. Resultado esperado

Estimación con SSD NVMe típico (3 GB/s lectura sequencial, ~10k
files/s remove):

| Métrica | Antes | Después | Mejora |
|---|---|---|---|
| 400 MB / 56 archivos / 12 caches | 456 s | **~25-40 s** | ~12× |
| 5 GB / 50k archivos / 30 caches | ~30 min | **~3 min** | ~10× |
| Caché vacío (todo programado para reboot) | ~80 s | **~5 s** | ~16× |
| Sólo restore point sin limpieza | 15 s | 15 s | sin cambio |

Mejora máxima limitada por:

- `SRSetRestorePointW` sigue siendo síncrono y lento.
- Restart Manager (cuando se invoca, lazy ahora).
- Sleeps de tokio scheduling.

## 4. Plan de implementación por pasos

1. **Paso A** — Añadir `tokio_util` + `futures` (si falta) al
   `Cargo.toml`. `cargo check` limpio.
2. **Paso B** — Extraer `TrackerHandle` thread-safe. Sustituir
   referencias en `execute_plan` *manteniendo el flujo secuencial*.
   Verificar tests, tiempo sin cambios.
3. **Paso C** — Convertir `walk_and_delete` a `async` con
   `tokio::fs` y `stream::for_each_concurrent(8, ...)` dentro de una
   ubicación.
4. **Paso D** — Convertir el loop de ubicaciones a `JoinSet` con
   `PARALLEL_LOCATIONS = 4`. Detección SSD vs HDD para ajustar a 1
   en HDD.
5. **Paso E** — Eliminar retry interno; cambiar a "try-once or
   schedule".
6. **Paso F** — Lazy `who_locks_path` — sólo bajo demanda en
   `classify_residual` (doc 15) y nunca en `analyze_locations`.
7. **Paso G** — Eliminar la doble llamada `scan` desde el frontend.

Cada paso se commitea por separado para que `git bisect` localice
regresiones.

## 5. Tests

```rust
#[tokio::test]
async fn parallel_limpia_4x_ubicaciones_a_la_vez() {
    // Plan con 8 ubicaciones, cada una con 100 archivos triviales.
    let plan = synthetic_plan_with_n_locations(8, 100);
    let start = Instant::now();
    let mut em = MockEmitter::default();
    let _ = execute_plan(
        "test", plan, ExecutePlanOpts::default(), &mut em,
        CancellationToken::new()
    ).await;
    let dur = start.elapsed();
    // 8 locs × 100 files; en serie cada loc tarda ~X; en paralelo ~X*2.
    assert!(dur.as_secs() < 4, "Esperado <4 s, fue {:?}", dur);
}

#[tokio::test]
async fn async_walk_borra_todos_los_archivos_de_un_dir() {
    let tmp = tempfile::tempdir().unwrap();
    for i in 0..100 {
        std::fs::write(tmp.path().join(format!("f{i}.tmp")), b"x").unwrap();
    }
    let tracker = TrackerHandle::new_test();
    walk_and_delete_async(tmp.path(), tracker.clone(), /* ... */).await.unwrap();
    let remaining = std::fs::read_dir(tmp.path()).unwrap().count();
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn try_delete_va_directo_a_schedule_sin_retry() {
    // Crear archivo, abrirlo en modo exclusivo en otro thread,
    // intentar delete — debería caer en schedule sin esperar 700 ms.
    // Medir: dur < 50 ms.
}

#[test]
fn ssd_detection_works_on_main_drive() {
    // Sólo verificar que no panica; el valor depende del hardware.
    let _ = crate::platform::storage::is_ssd("C:\\");
}
```

## 6. Criterio de done

- [ ] `cargo bench` (o un benchmark manual) muestra ≥**5×** speedup
      en plan sintético de 50 locations × 100 archivos.
- [ ] Caso real del usuario (~400 MB / 56 archivos) tarda **<60 s**
      en SSD.
- [ ] Tests Rust nuevos pasan.
- [ ] **Todos los tests existentes** del doc 01 (shell-safe-close)
      siguen en verde.
- [ ] `security-auditor` revisa el PR y aprueba.
- [ ] No regresiones en `cache_background` (auto-analyze) — sigue
      tardando lo mismo.
- [ ] `analyze_locations` ya no llama a Restart Manager (lazy).

## 7. Riesgos

- **Race conditions** en `TrackerCore` bajo `Mutex`: tests
  estresados con TSAN cuando se pueda. La sección crítica es trivial;
  el riesgo es bajo pero hay que tenerlo presente.
- **`pending_rename` no es seguro para llamarlo concurrentemente**
  desde múltiples threads — escribe en el registro Windows. Mitigación:
  un único worker drena un canal con todas las solicitudes de
  schedule. Latencia añadida: <1 ms.
- **OOM con muchos `JoinSet`**: con `PARALLEL_LOCATIONS = 4` y
  buffers de `chunk_size = 32`, max ~4 × 32 = 128 paths en RAM. Es
  trivial.
- **Antivirus**: ESET / Defender / Kaspersky pueden monitorizar
  borrado y throttlear. Paralelismo no ayuda contra ese cuello de
  botella — pero tampoco empeora.
- **El audit log** entry sigue siendo única (uno por run). No hay race
  ahí.
- **Backwards compatibility de eventos**: la firma de `CleanEmitter`
  no cambia. El frontend ya gestiona ráfagas de progreso (throttle
  en backend, throttle en `CleanConsoleLog` por la animación tail
  limitada).
- **Cuidado con `tokio::fs::remove_dir` en directorios no vacíos**:
  falla. Mantener la secuencia "drenar archivos → remove_dir". El
  snippet ya lo respeta.

## 8. Métricas a registrar tras el merge

Aprovechar el `throughput-stats.json` del doc 09 (ya implementado)
para validar la mejora:

- Muestra `mean_bytes_per_sec` antes del merge: ~880 KB/s.
- Muestra esperada después: ≥10 MB/s en SSD, ≥3 MB/s en HDD.

Si tras una semana las muestras siguen bajas, abrir un seguimiento:
posible problema con el antivirus del usuario, no con el motor.
