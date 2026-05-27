# 14 — Bug: el ETA siempre dice "calculando…"

> **Severidad:** 🟡 P1 — bug funcional reportado por el usuario.
> _"el tiempo estimado nunca sale, se queda calculando, me tardó 7 min
> y 36 segs para eliminar 56 archivos"._
> **Modelo:** Sonnet 4.6.
> **Bloque:** 2 de 4 del Set C (quality of life).

## 1. Problema

`src-tauri/src/domain/cache.rs:103-111`:

```rust
fn eta_secs(&self) -> Option<f32> {
    let tp = self.throughput_bps();
    // Requerir al menos 1 MB/s de muestra para dar una estimación fiable.
    if tp < 1_000_000 {
        return None;
    }
    let remaining = self.total_estimated_bytes.saturating_sub(self.bytes_freed);
    Some(remaining as f32 / tp as f32)
}
```

El umbral `< 1_000_000` (1 MB/s) es **demasiado alto** para el caso
real del usuario:

- Caso real: 400 MB en 456 s → throughput medio = **877 KB/s**.
- Por debajo de 1 MB/s en TODO momento → `eta_secs()` siempre `None`
  → frontend muestra `"calculando…"` indefinidamente.

Tres causas que mantienen el throughput bajo:

1. **Archivos muy pequeños**: `Prefetch`, `INetCache`, `thumbcache`
   suelen tener miles de archivos de 1-100 KB. El cost domina
   `std::fs::remove_file` syscall, no el bandwidth del disco.
2. **Restart Manager queries** caras antes de cada caché bloqueada.
3. **Sleeps de retry**: `delete_with_retry` espera 100→200→400 ms.

Aparte, la ventana móvil de 2 s puede caer a 0 muestras entre dos
emits del throttle, devolviendo `tp = 0` durante ráfagas.

## 2. Fix propuesto

### 2.1 Bajar el umbral mínimo

`src-tauri/src/domain/cache.rs:106`:

```rust
const MIN_THROUGHPUT_FOR_ETA: u64 = 50_000; // 50 KB/s
```

Con 50 KB/s todavía es una muestra plausible (1 archivo de 50 KB cada
segundo), suficiente para una estimación útil aunque imprecisa.

### 2.2 Fallback con throughput acumulado total

Si la ventana móvil de 2 s no tiene muestras suficientes, calcular
desde el **acumulado total** (sin ventana):

```rust
fn eta_secs(&self) -> Option<f32> {
    let remaining = self.total_estimated_bytes.saturating_sub(self.bytes_freed);
    if remaining == 0 {
        return Some(0.0);
    }
    let elapsed = self.started.elapsed().as_secs_f64();
    if elapsed < 1.0 {
        return None;     // muestra insuficiente — menos de 1 s
    }

    // 1) Throughput de la ventana móvil (preferido — reacciona a cambios).
    let window_tp = self.throughput_bps();

    // 2) Throughput acumulado (fallback — siempre tiene datos).
    let cumulative_tp = if self.bytes_freed > 0 {
        (self.bytes_freed as f64 / elapsed) as u64
    } else {
        0
    };

    let tp = if window_tp >= Self::MIN_THROUGHPUT_FOR_ETA {
        window_tp
    } else if cumulative_tp >= Self::MIN_THROUGHPUT_FOR_ETA {
        cumulative_tp
    } else {
        // Aún sin datos útiles tras el primer segundo: estimación
        // optimista con 1 MB/s para no dejar al usuario sin info.
        // Se marca con un flag aparte para que la UI lo señale.
        1_024_000
    };

    Some(remaining as f32 / tp as f32)
}
```

### 2.3 Distinguir "estimación fiable" vs "estimación amplia"

Añadir un flag al payload:

`src-tauri/src/models/cache.rs`:

```rust
pub struct CleanProgressPayload {
    // ... existentes ...
    pub eta_secs: Option<f32>,
    /// true cuando el ETA viene de la ventana móvil reciente
    /// (suficientes muestras + throughput estable).
    /// false cuando es estimación amplia (acumulado o fallback).
    pub eta_is_precise: bool,
}
```

Actualizar `maybe_emit` para setear el flag:

```rust
fn maybe_emit<E: CleanEmitter>(...) {
    // ...
    let (eta, precise) = self.eta_with_precision();
    emitter.progress(CleanProgressPayload {
        // ...
        eta_secs: eta,
        eta_is_precise: precise,
    });
}

fn eta_with_precision(&self) -> (Option<f32>, bool) {
    let remaining = /* ... */;
    let window_tp = self.throughput_bps();
    let elapsed = self.started.elapsed().as_secs_f64();
    let cumulative_tp = if elapsed > 0.5 && self.bytes_freed > 0 {
        (self.bytes_freed as f64 / elapsed) as u64
    } else { 0 };

    if window_tp >= Self::MIN_THROUGHPUT_FOR_ETA {
        return (Some(remaining as f32 / window_tp as f32), true);
    }
    if cumulative_tp >= Self::MIN_THROUGHPUT_FOR_ETA {
        return (Some(remaining as f32 / cumulative_tp as f32), false);
    }
    if elapsed > 5.0 {
        // Fallback muy conservador después de 5 s.
        return (Some(remaining as f32 / 100_000_f32), false);
    }
    (None, false)
}
```

### 2.4 UI — distinguir precisión

`src/features/cache-cleaner/clean-console-header.tsx`:

```tsx
<span className="text-ink-secondary">
  Tiempo restante:{" "}
  <span className="text-ink-primary font-medium">
    {progress.etaSecs != null
      ? progress.etaIsPrecise
        ? `~${formatDuration(progress.etaSecs)}`
        : `${formatDuration(progress.etaSecs * 0.6)} – ${formatDuration(progress.etaSecs * 1.6)}`
      : "calculando…"}
  </span>
</span>
```

Cuando la estimación es amplia (no precisa), mostrarla como un
**rango** (`40 s – 1 min 50 s`) para que el usuario entienda que hay
incertidumbre. Esto es **mucho** mejor que el actual silencio
infinito.

### 2.5 Mejorar la ventana móvil

`src-tauri/src/domain/cache.rs:73-88` — `record_bytes`:

```rust
fn record_bytes(&mut self, delta: u64) {
    self.bytes_freed += delta;
    let now = Instant::now();
    self.samples.push_back((self.bytes_freed, now));

    // Mantener al menos 1 muestra cada 100 ms en el último segundo,
    // y al menos 5 muestras en total para tener "ventana".
    while self.samples.len() > 60 {
        self.samples.pop_front();
    }
    while let Some((_, t)) = self.samples.front() {
        if t.elapsed() > Duration::from_secs(3) && self.samples.len() > 5 {
            self.samples.pop_front();
        } else {
            break;
        }
    }
}
```

La ventana sube de 2 s a 3 s **siempre que haya >5 muestras**, lo que
ayuda con las ráfagas. Y obliga a tener al menos 5 muestras antes de
empezar a tirar viejas — sin esto, la primera ráfaga deja la ventana
vacía instantáneamente.

### 2.6 Forzar emit en bordes de fase

El throttle de 150 ms es bueno para no saturar, pero al cambiar de
fase queremos que la UI vea el nuevo estado y reset de progresivo de
inmediato. `set_phase` ya emite `cache:phase`, pero también conviene
forzar un `progress` con el contador actualizado:

```rust
fn set_phase<E: CleanEmitter>(&mut self, emitter: &mut E, phase: CleanPhase) {
    self.phase = phase.clone();
    emitter.phase(CleanPhaseEvent {
        run_id: self.run_id.clone(),
        phase,
        elapsed_ms: self.started.elapsed().as_millis() as u64,
    });
    // Forzar también un progress con el nuevo phase y los contadores.
    self.maybe_emit(emitter, /* force = */ true, None, None, None);
}
```

(Si la actual `maybe_emit` ya recibe `force=true` en `set_phase`,
verificar; si no, añadir.)

## 3. Tests

```rust
#[test]
fn eta_falls_back_to_cumulative_when_window_is_slow() {
    let mut tracker = ProgressTracker::new("t", 1, 10_000_000);
    // Simular 5 segundos transcurridos con 500 KB borrados.
    // Como Instant no es mockeable directamente, usar un wrapper en
    // futuras revisiones; aquí hacemos sleep real corto:
    tracker.record_bytes(100_000);
    std::thread::sleep(std::time::Duration::from_millis(500));
    tracker.record_bytes(100_000);

    let (eta, precise) = tracker.eta_with_precision();
    assert!(eta.is_some(), "ETA debe estar presente con 200 KB en ~0.5 s");
    assert!(!precise, "Con throughput bajo, debe marcarse como no preciso");
}

#[test]
fn eta_marks_precise_when_window_has_high_throughput() {
    let mut tracker = ProgressTracker::new("t", 1, 100_000_000);
    // Throughput simulado: 10 MB en pocos ms.
    for _ in 0..10 {
        tracker.record_bytes(1_000_000);
    }
    std::thread::sleep(std::time::Duration::from_millis(150));
    let (eta, precise) = tracker.eta_with_precision();
    assert!(eta.is_some());
    assert!(precise);
}

#[test]
fn eta_returns_none_before_one_second() {
    let tracker = ProgressTracker::new("t", 1, 1_000_000);
    let (eta, _) = tracker.eta_with_precision();
    assert!(eta.is_none());
}
```

## 4. Criterio de done

- [ ] Threshold mínimo de throughput baja a 50 KB/s (`MIN_THROUGHPUT_FOR_ETA`).
- [ ] Fallback de throughput acumulado activo después de 1 s
      transcurrido.
- [ ] Flag `etaIsPrecise` en el payload, propagado a frontend.
- [ ] Frontend muestra `~5 min` cuando precise=true, `3 min – 8 min`
      cuando precise=false.
- [ ] En el caso real del usuario (400 MB en 456 s = 877 KB/s), el ETA
      deja de decir "calculando…" en menos de 3 s desde el inicio.
- [ ] Tests pasan: 3 nuevos en `domain::cache::tests`.

## 5. Riesgos

- **Estimaciones malas al inicio**: con sólo 1-2 archivos procesados,
  el throughput acumulado es ruidoso. El flag `precise=false` y el
  rango en la UI cubren esa incertidumbre. Es mejor que silencio
  total.
- **Carrera entre throttle y record_bytes**: si entre emits cae la
  ventana móvil, el fallback al acumulado entra solo. No hay race.
- **Flag retrocompatible**: añadir un campo nuevo al payload puede
  romper consumidores que serialicen estrictamente. Sólo lo consume
  nuestro frontend; añadir `#[serde(default)]` por si acaso.
- **El usuario podría no querer ver el rango amplio**: si después de
  los primeros 30 s la estimación sigue siendo amplia, considerar
  fallback a un texto neutro "más de 5 minutos" para no agobiar.
  Iterable, no bloquea el done.
