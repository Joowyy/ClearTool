# 10 — Performance

## Presupuestos (qué consideramos "rápido")

| Operación | Objetivo MVP |
|-----------|-------------|
| `list_drives` | < 50 ms |
| Primer evento `disk:progress` desde que se pulsa Escanear | < 500 ms |
| Escaneo C:\ típico (200 GB, 1M archivos, SSD NVMe) | < 2 min |
| Resize del treemap (recalcular layout 10 k leaves) | < 150 ms |
| Hover → tooltip aparece | < 30 ms |
| Click → zoom in / out | < 80 ms |

## Cuellos típicos

1. **IO del FS** — predomina sobre CPU. Ganamos poco con paralelismo en
   SSD, mucho en HDD (cabezal). Iteración 2: probar `rayon` sobre los
   hijos del root y medir.
2. **Serialización Tauri** — devolver un árbol con 100 k nodos cuesta. Por
   eso emitimos solo nodos con `size >= min_size_mb`. El árbol completo
   nunca cruza el IPC, sólo el podado.
3. **Render canvas** — bien siempre que mantengamos el dpr en check (no
   subir de 2.0 incluso en pantallas 4×).

## Throttling de eventos

```rust
let mut last_emit = Instant::now();
let interval = Duration::from_millis(200);
let counter = AtomicU64::new(0);

// dentro del walker:
let c = counter.fetch_add(1, Relaxed);
if c % 5000 == 0 || last_emit.elapsed() > interval {
    emit_progress();
    last_emit = Instant::now();
}
```

Frontend usa `requestAnimationFrame` para coalescer múltiples eventos en
una sola actualización de estado por frame.

## Memoria

- `AggregateNode` interno (no se serializa) usa `String` para path. En un
  disco con 1 M archivos sólo guardamos los que sobreviven a la poda
  (~10–100 k) — perfectamente manejable.
- `by_extension: HashMap<String, u64>` por nodo se colapsa hacia arriba
  y se borra al terminar el escaneo.
- Una sola sesión de escaneo viva. El `TreemapNode` raíz se conserva en
  memoria de Rust hasta el siguiente escaneo o cancelación explícita.

## Anti-patrones a evitar

- ❌ Hacer `read_dir` con `for entry in fs::read_dir(...).collect::<Vec<_>>()`
  para volcar todo en RAM antes de iterar: peor cache locality.
- ❌ Calcular `formatBytes` mil veces en el frontend en un map; mejor en
  el render de cada fila.
- ❌ Pasar el árbol entero al `ResizeObserver` callback; sólo el size
  cambia, no la jerarquía.

## Profiling

Reservar tiempo para `cargo flamegraph` en un escaneo real antes de
declarar el MVP completo. Específicamente medir:

- Tiempo en `metadata()` vs `read_dir()`.
- Coste real de la cancelación atómica por iteración.
- Tamaño final del JSON serializado a TS.
