# 03 — Scan engine

## Premisa

El escaneo de un disco entero puede tardar varios minutos. La UI **no
puede** quedarse en blanco. Soluciones:

1. Devolver árbol completo al final (lo que hace hoy `build_treemap_data`)
   → mal: el usuario espera 60 s sin feedback.
2. Streaming por eventos → bueno: el usuario ve crecer el progreso y
   bloques parciales.

Adoptamos el patrón **streaming + snapshot final**: durante el escaneo
emitimos `disk:progress` con contadores acumulados; al terminar, emitimos
`disk:complete` con el árbol y el reporte enteros.

## Algoritmo

### Fase 1 — recorrido recursivo (paralelizable)

```
fn scan_directory(root: &Path, ctx: &ScanCtx) -> AggregateNode {
    let mut bytes = 0;
    let mut files = 0;
    let mut dirs = 0;
    let mut by_extension: HashMap<String, u64> = HashMap::new();
    let mut children: Vec<AggregateNode> = Vec::new();
    let mut last_modified = 0_u64;

    for entry in read_dir(root)? {
        if cancelled() { break; }
        ctx.touched_files.fetch_add(1, Relaxed);

        if entry.is_dir() && !entry.is_reparse_point() {
            let child = scan_directory(&entry.path, ctx);  // recursión
            bytes += child.bytes;
            files += child.files;
            dirs += 1 + child.dirs;
            for (k,v) in &child.by_extension { *by_extension.entry(k.clone()).or_default() += v; }
            children.push(child);
        } else if entry.is_file() {
            let meta = entry.metadata()?;
            bytes += meta.len();
            files += 1;
            *by_extension.entry(ext_of(&entry.path)).or_default() += meta.len();
            last_modified = last_modified.max(mtime(&meta));
        }

        ctx.maybe_emit_progress();
    }

    AggregateNode { path, bytes, files, dirs, by_extension, children, last_modified }
}
```

### Fase 2 — poda y serialización

Después de tener el árbol agregado, lo filtramos para emitir solo los
nodos relevantes:

- Carpeta visible si `size >= MIN_SIZE_FOR_TREEMAP_MB * 1024 * 1024`
  (default 10 MB; configurable desde header).
- Archivos individuales del root sólo si son lo suficientemente grandes
  (`>= 100 MB`). El resto se agrega en un placeholder
  `"(otros archivos)"`.
- Profundidad: ilimitada para el cómputo de tamaños, pero el árbol
  emitido se trunca en `max_depth_emit` (default 6). Los descendientes
  se cargan lazy si el usuario hace zoom.

### Fase 3 — métricas globales

Construimos `DiskAnalysisReport`:

- `scannedAt`, `durationMs`.
- `root`, `totalBytes`, `totalFiles`, `totalDirs`.
- `topExtensions`: vector de `{ ext, bytes, files }` ordenado desc, top 20.
- `largestFolders`: top 20 carpetas por bytes (con path completo).
- `ageDistribution`: cubetas por `lastModified`:
  - `< 7 días`, `< 30 días`, `< 90 días`, `< 1 año`, `< 5 años`, `> 5 años`.
- `largestFiles`: top 20 archivos absolutos (incluye los podados del árbol).

## Cancelación

Estructura `Arc<AtomicBool>` en `ScanCtx`. El comando IPC `cancel_disk_scan`
la pone a `true`. Cada iteración del walker chequea y aborta.

## Streaming de progreso

Cada N entries (configurable, default 5000) o cada 200 ms, emitimos
`disk:progress`:

```ts
interface DiskScanProgressPayload {
  scanId: string;
  bytesScanned: number;
  filesScanned: number;
  dirsScanned: number;
  currentPath: string;     // path actualmente siendo recorrido (best-effort)
  elapsedMs: number;
}
```

## Paralelismo (iteración 2)

MVP: recorrido sequencial. Iteración 2: `rayon` para dividir los hijos
del root entre N hilos. Hay que medir antes de añadir complejidad — en
SSD la diferencia es menor de la esperada porque el cuello es el FS, no
el CPU.

## Reglas de saneamiento

- **No seguir reparse points** salvo que el usuario lo active explícitamente.
  Detectar con `meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT`.
- **Permisos denegados** → contar el nodo como tamaño 0 y marcarlo con
  `error: "access denied"`. Continuar.
- **Paths > MAX_PATH (260)** → en Windows usar prefijo `\\?\` cuando se
  abren via Win32 directo. `walkdir`/`std::fs` ya lo manejan internamente
  desde Rust ≥ 1.70 en la mayoría de casos; documentar excepciones si
  aparecen.
- **Symlinks circulares** → al no seguir reparse points por defecto, no
  es un problema. Cuando se habilite, llevar `HashSet<canonical_path>`.
