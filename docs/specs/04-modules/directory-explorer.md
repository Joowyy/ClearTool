# 04.1 — Módulo Directory Explorer

## Propósito

Permitir al usuario navegar el sistema de archivos en forma de árbol con tamaños reales calculados, identificar qué carpetas pesan más y abrir un punto de entrada rápido al **Cache cleaner** o al **Explorer del sistema** según convenga.

No es un gestor de archivos: **no copia, no mueve, no renombra**. Solo lee y, opcionalmente, ofrece "abrir en Explorer" o "marcar como sospechoso para limpieza manual".

## Stakeholders

- Usuario final que quiere ver dónde se le va el espacio.
- Subagente líder: `react-frontend` (UI) + `tauri-rust-backend` (escaneo). Consulta a `windows-systems-expert` para temas de paths largos / reparse points.

## Capabilities Tauri requeridas

- `default.json` — solo lectura. Este módulo nunca escribe, así que opera en modo limitado sin admin (con la salvedad de que algunos directorios protegidos devuelven `Permission`).

## Comandos expuestos

```rust
// commands/explorer.rs

#[tauri::command]
pub async fn scan_tree(
    app: AppHandle,
    input: ScanTreeInput,
) -> Result<ScanTreeHandle, AppError>;

#[tauri::command]
pub async fn cancel_scan(
    handle: ScanTreeHandle,
) -> Result<(), AppError>;

#[tauri::command]
pub async fn compute_directory_size(
    path: String,
    follow_reparse_points: bool,
) -> Result<DirectorySize, AppError>;
```

### Modelos

```rust
// models/tree.rs

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScanTreeInput {
    pub root: String,           // path absoluto, validado
    pub max_depth: u8,          // 0 = solo nivel raíz; recomendado 3
    pub follow_reparse_points: bool, // default false
    pub include_hidden: bool,   // default true (este es un explorer técnico)
    pub min_size_bytes: Option<u64>, // filtra entradas pequeñas
    pub size_strategy: SizeStrategy, // ver abajo
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SizeStrategy {
    /// Tamaño lógico (suma de file sizes).
    Logical,
    /// Tamaño físico en disco (cluster size aware).
    Physical,
    /// No calcular tamaño de subárboles, solo del directorio actual.
    Lazy,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScanTreeHandle {
    pub scan_id: Uuid,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TreeNode {
    pub path: String,
    pub name: String,
    pub kind: NodeKind, // Dir | File | Symlink | Junction
    pub size_bytes: u64,
    pub last_modified: Option<DateTime<Utc>>,
    pub children_count: Option<u32>, // None hasta que el subárbol termine
    pub is_protected: bool,          // true si requirió un fallback de permisos
    pub error: Option<String>,       // path inaccesible: razón humanizada
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DirectorySize {
    pub path: String,
    pub logical_bytes: u64,
    pub physical_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
}
```

## Eventos

| Evento | Payload | Cuándo |
|---|---|---|
| `explorer:progress` | `{ scan_id, scanned_dirs, scanned_files, current_path }` | cada 100 ms o cada 5 000 entradas, lo que ocurra antes |
| `explorer:node` | `TreeNode` | a medida que se completa cada subdirectorio raíz; permite render incremental |
| `explorer:done` | `{ scan_id, total_bytes, total_files, total_dirs, duration_ms }` | al terminar |
| `explorer:error` | `{ scan_id, path, message }` | path saltado por permisos o IO |

Frontend conecta con `useTauriEvent` en `features/explorer/use-tree-stream.ts` y va llenando un store local indexado por `scan_id`.

## Algoritmo de escaneo

1. Validar el `root`:
   - Existir, ser directorio, estar dentro de la allowlist (drives locales y montados; nunca `\\?\GLOBALROOT`, nunca `\\Device\\…`).
   - Normalizar con `dunce::canonicalize` para evitar paths UNC inesperados.
2. Crear `scan_id = Uuid::new_v4()` y registrarlo en un `DashMap<Uuid, ScanContext>` con un `CancellationToken`.
3. `tokio::task::spawn_blocking`:
   - Recorrido BFS con `walkdir::WalkDir` configurado:
     - `.follow_links(input.follow_reparse_points)`
     - `.same_file_system(true)` (no saltar a otros drives)
     - `.max_depth(input.max_depth as usize)`
   - Por cada entrada:
     - Si el cancellation token está disparado → break.
     - Si error → emitir `explorer:error`, continuar.
     - Acumular tamaño según `SizeStrategy`.
4. Para `SizeStrategy::Physical`, usar `GetCompressedFileSizeW` cuando esté disponible y caer a `metadata().len()` como fallback.
5. Emitir nodos a granularidad de "primer nivel debajo del root". Profundidades mayores se incluyen como children del nodo padre antes de emitir.

## Manejo de paths problemáticos

| Caso | Estrategia |
|---|---|
| Paths > 260 caracteres (MAX_PATH) | Prefijo `\\?\` automático cuando llamamos a APIs Win32. |
| Reparse points / Junctions | `kind = Junction`; no se descienden salvo `follow_reparse_points`. |
| Directorios sin permisos | `error = "Acceso denegado"`, `size_bytes = 0`, `is_protected = true`. UI los pinta en gris cursiva. |
| Archivos abiertos exclusivamente | Tamaño se obtiene igual; la UI no necesita abrir handles. |
| Drives offline / desconectados | Se filtran del listado de roots. |

## Perfil de uso típico

1. UI muestra al abrir `/explorer` los drives detectados (`C:\`, `D:\`, …) con su tamaño total y libre vía `GetDiskFreeSpaceExW`.
2. Click en un drive → `scan_tree({ root: 'C:\\', max_depth: 2 })`.
3. Mientras llegan eventos `explorer:node`, el árbol se va llenando ordenado **por tamaño descendente**.
4. Click en un nodo no-hoja → si su `children_count` aún es `None`, se dispara un `scan_tree` adicional con `root` = ese path.

## UI (resumen, detalle en spec 03)

- Componente raíz: `<FileTree>` (react-arborist).
- Cada fila: nombre, icono kind, tamaño humanizado, barra proporcional contra el padre, badge de "protegido" si aplica.
- Toolbar: filtro de tamaño mínimo, toggle ocultos, toggle "seguir junctions", input de path manual, botón "Cancelar escaneo".
- Click derecho:
  - "Abrir en Explorer de Windows" (Tauri shell `open`).
  - "Copiar ruta".
  - "Sugerir como ubicación de caché custom" → abre el modal de `cache-cleaner` precargando ese path.

## Performance — objetivos

| Operación | Target |
|---|---|
| `C:\` profundidad 2 en SSD NVMe | < 5 s |
| `C:\Users\<u>` profundidad 4 | < 15 s |
| Render del primer nivel | < 200 ms tras primer evento |
| Memoria pico para 1M entradas | < 250 MB |

## Errores esperados

| Variante | Causa | UX |
|---|---|---|
| `Permission` | path fuera de allowlist o requiere admin | Toast: "Esa ruta requiere permisos elevados. ¿Reiniciar como admin?" |
| `Io` | drive desconectado a mitad de scan | Banner: "Se interrumpió el escaneo: …" + botón reintentar |
| `Cancelled` | usuario canceló | Sin toast; marcar el handle como abortado |

## Tests

- Unit Rust: dado un fixture de árbol controlado en `temp_dir()`, validar tamaños lógicos y físicos, conteos, ordenamiento.
- Unit Rust: paths con junctions hechos por test (`mklink /J`) — descender o no según flag.
- Unit Rust: cancelación a mitad de escaneo no fuga el task ni el emitter.
- Vitest: `<FileTree>` recibiendo eventos sintéticos renderiza incrementalmente.
- Playwright: smoke "abrir explorer y ver `C:\` con tamaño calculado".

## Futuro (no en MVP)

- Mapa treemap estilo WinDirStat.
- Detección de archivos duplicados por hash.
- Exportar reporte a CSV.
