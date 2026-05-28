// domain/disk.rs — Disk Analyzer: enumeración de discos, escaneo con
// streaming de progreso, cómputo de stats y construcción de TreemapNode.
//
// Flujo:
//   1. `list_drives()`              — enumeración rápida vía Win32 (filesystem.rs).
//   2. `run_analysis(app, input)`   — escaneo asíncrono que:
//        - recorre el árbol (walker propio para tener cancelación + métricas
//          incrementales), agregando bytes/files/dirs/by_ext/last_mod por nodo,
//        - emite `disk:progress` cada 200 ms o cada 5000 entries,
//        - tras terminar, poda el árbol (min_size_mb, max_depth_emit) y produce
//          TreemapNode + DiskAnalysisReport.
//   3. `cancel_scan(scan_id)`       — flip atómico que aborta el walker activo.
//
// El estado global (`ACTIVE_SCAN`) garantiza un único análisis vivo a la vez.

use crate::core::{AppError, AppResult};
use crate::models::disk::{
    AgeBucket, AgeDistribution, BuildTreemapInput, DiskAnalysisReport, DiskAnalysisResult,
    DiskScanProgressPayload, DriveListing, ExtCategory, ExtensionStat, FileStat, FolderStat,
    NodeKind, TreemapNode,
};
use chrono::{DateTime, SecondsFormat, Utc};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;

// ── enumeración de drives ──────────────────────────────────────────────────

pub fn list_drives() -> AppResult<Vec<DriveListing>> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::filesystem::enumerate_drives()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

// ── cancelación + estado global ───────────────────────────────────────────

struct ActiveScan {
    scan_id: String,
    cancel: Arc<AtomicBool>,
}

static ACTIVE_SCAN: Lazy<Mutex<Option<ActiveScan>>> = Lazy::new(|| Mutex::new(None));

pub fn cancel_scan(scan_id: &str) {
    if let Ok(guard) = ACTIVE_SCAN.lock() {
        if let Some(active) = guard.as_ref() {
            if active.scan_id == scan_id {
                active.cancel.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn register_scan(scan_id: &str) -> AppResult<Arc<AtomicBool>> {
    let mut guard = ACTIVE_SCAN
        .lock()
        .map_err(|_| AppError::External("disk scan mutex poisoned".into()))?;
    if guard.is_some() {
        return Err(AppError::Permission(
            "another disk scan is in progress".into(),
        ));
    }
    let cancel = Arc::new(AtomicBool::new(false));
    *guard = Some(ActiveScan {
        scan_id: scan_id.to_string(),
        cancel: cancel.clone(),
    });
    Ok(cancel)
}

fn unregister_scan(scan_id: &str) {
    if let Ok(mut guard) = ACTIVE_SCAN.lock() {
        if let Some(active) = guard.as_ref() {
            if active.scan_id == scan_id {
                *guard = None;
            }
        }
    }
}

// ── árbol agregado interno (no se serializa) ──────────────────────────────

#[derive(Debug, Clone)]
struct Aggregate {
    name: String,
    path: PathBuf,
    kind: NodeKind,
    extension: Option<String>,
    own_bytes: u64,    // bytes del propio archivo (0 para dirs)
    bytes: u64,        // bytes acumulados (recursivo)
    file_count: u64,   // archivos descendientes
    dir_count: u64,    // subdirs descendientes
    last_modified: u64, // mtime más reciente (segundos epoch)
    by_ext: HashMap<String, (u64, u64)>, // ext → (bytes, file_count)
    children: Vec<Aggregate>,
    error: Option<String>,
}

impl Aggregate {
    fn leaf_file(path: &Path, bytes: u64, mtime: u64) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase());
        let mut by_ext = HashMap::new();
        let key = ext.clone().unwrap_or_default();
        by_ext.insert(key, (bytes, 1));
        Self {
            name,
            path: path.to_path_buf(),
            kind: NodeKind::File,
            extension: ext,
            own_bytes: bytes,
            bytes,
            file_count: 1,
            dir_count: 0,
            last_modified: mtime,
            by_ext,
            children: Vec::new(),
            error: None,
        }
    }
}

// ── contexto del scan ──────────────────────────────────────────────────────

struct ScanCtx<'a> {
    scan_id: String,
    cancel: Arc<AtomicBool>,
    app: &'a tauri::AppHandle,
    follow_reparse_points: bool,
    started_at: Instant,
    last_emit: Mutex<Instant>,
    current_path: Mutex<String>,
    bytes: std::sync::atomic::AtomicU64,
    files: std::sync::atomic::AtomicU64,
    dirs: std::sync::atomic::AtomicU64,
    counter: std::sync::atomic::AtomicU64,
    errors: Mutex<Vec<String>>,
}

impl<'a> ScanCtx<'a> {
    fn maybe_emit(&self) {
        let c = self.counter.fetch_add(1, Ordering::Relaxed);
        // Probar emisión cada 4096 entries para evitar lock excesivo.
        if c % 4096 != 0 {
            return;
        }
        let mut last = match self.last_emit.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if last.elapsed() < std::time::Duration::from_millis(200) {
            return;
        }
        *last = Instant::now();
        let current = self
            .current_path
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        let payload = DiskScanProgressPayload {
            scan_id: self.scan_id.clone(),
            bytes_scanned: self.bytes.load(Ordering::Relaxed),
            files_scanned: self.files.load(Ordering::Relaxed),
            dirs_scanned: self.dirs.load(Ordering::Relaxed),
            current_path: current,
            elapsed_ms: self.started_at.elapsed().as_millis() as u64,
        };
        let _ = self.app.emit("disk:progress", payload);
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    fn record_error(&self, msg: String) {
        if let Ok(mut g) = self.errors.lock() {
            // Limitar para no inflar memoria con discos con miles de errores.
            if g.len() < 500 {
                g.push(msg);
            }
        }
    }

    fn set_current(&self, p: &Path) {
        if let Ok(mut g) = self.current_path.lock() {
            *g = p.to_string_lossy().to_string();
        }
    }
}

// ── walker recursivo ──────────────────────────────────────────────────────

fn walk(path: &Path, ctx: &ScanCtx) -> Aggregate {
    if ctx.cancelled() {
        return placeholder_dir(path, Some("cancelled".into()));
    }

    let meta = match path.symlink_metadata() {
        Ok(m) => m,
        Err(e) => {
            ctx.record_error(format!("{}: {}", path.display(), e));
            return placeholder_dir(path, Some(format!("{e}")));
        }
    };

    let ft = meta.file_type();
    let is_symlink = ft.is_symlink();

    #[cfg(target_os = "windows")]
    let is_reparse = {
        use std::os::windows::fs::MetadataExt;
        // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
        (meta.file_attributes() & 0x400) != 0
    };
    #[cfg(not(target_os = "windows"))]
    let is_reparse = false;

    if is_symlink || (is_reparse && !ctx.follow_reparse_points) {
        let kind = if is_reparse {
            NodeKind::Junction
        } else {
            NodeKind::Symlink
        };
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        ctx.maybe_emit();
        return Aggregate {
            name,
            path: path.to_path_buf(),
            kind,
            extension: None,
            own_bytes: 0,
            bytes: 0,
            file_count: 0,
            dir_count: 0,
            last_modified: 0,
            by_ext: HashMap::new(),
            children: Vec::new(),
            error: None,
        };
    }

    if ft.is_file() {
        let bytes = meta.len();
        ctx.bytes.fetch_add(bytes, Ordering::Relaxed);
        ctx.files.fetch_add(1, Ordering::Relaxed);
        let mtime = system_time_to_secs(meta.modified().ok());
        ctx.maybe_emit();
        return Aggregate::leaf_file(path, bytes, mtime);
    }

    if !ft.is_dir() {
        // Otros tipos (device files, etc.) — los ignoramos como nodo vacío.
        return placeholder_dir(path, None);
    }

    // Es directorio.
    ctx.dirs.fetch_add(1, Ordering::Relaxed);
    ctx.set_current(path);

    let mut children: Vec<Aggregate> = Vec::new();
    let rd = match std::fs::read_dir(path) {
        Ok(r) => r,
        Err(e) => {
            ctx.record_error(format!("{}: {}", path.display(), e));
            return placeholder_dir(path, Some(format!("{e}")));
        }
    };

    for entry in rd {
        if ctx.cancelled() {
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                ctx.record_error(format!("{}: {}", path.display(), e));
                continue;
            }
        };
        let child = walk(&entry.path(), ctx);
        children.push(child);
        ctx.maybe_emit();
    }

    // Agregar métricas de los hijos.
    let mut bytes: u64 = 0;
    let mut file_count: u64 = 0;
    let mut dir_count: u64 = 0;
    let mut last_modified: u64 = 0;
    let mut by_ext: HashMap<String, (u64, u64)> = HashMap::new();

    for c in &children {
        bytes = bytes.saturating_add(c.bytes);
        file_count = file_count.saturating_add(c.file_count);
        dir_count = dir_count.saturating_add(c.dir_count + if matches!(c.kind, NodeKind::Dir) { 1 } else { 0 });
        if c.last_modified > last_modified {
            last_modified = c.last_modified;
        }
        for (ext, (b, f)) in &c.by_ext {
            let entry = by_ext.entry(ext.clone()).or_insert((0, 0));
            entry.0 = entry.0.saturating_add(*b);
            entry.1 = entry.1.saturating_add(*f);
        }
    }

    // Si el dir actual no tiene modified más reciente desde meta, conservamos
    // el máximo de los hijos.
    if last_modified == 0 {
        last_modified = system_time_to_secs(meta.modified().ok());
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    Aggregate {
        name,
        path: path.to_path_buf(),
        kind: NodeKind::Dir,
        extension: None,
        own_bytes: 0,
        bytes,
        file_count,
        dir_count,
        last_modified,
        by_ext,
        children,
        error: None,
    }
}

fn placeholder_dir(path: &Path, error: Option<String>) -> Aggregate {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    Aggregate {
        name,
        path: path.to_path_buf(),
        kind: NodeKind::Dir,
        extension: None,
        own_bytes: 0,
        bytes: 0,
        file_count: 0,
        dir_count: 0,
        last_modified: 0,
        by_ext: HashMap::new(),
        children: Vec::new(),
        error,
    }
}

fn system_time_to_secs(t: Option<SystemTime>) -> u64 {
    t.and_then(|st| st.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── poda + serialización a TreemapNode ────────────────────────────────────

fn epoch_to_iso(secs: u64) -> Option<String> {
    if secs == 0 {
        return None;
    }
    DateTime::<Utc>::from_timestamp(secs as i64, 0)
        .map(|d| d.to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn prune(
    agg: Aggregate,
    parent_bytes: u64,
    root_bytes: u64,
    min_size_bytes: u64,
    max_depth_emit: u32,
    depth: u32,
) -> TreemapNode {
    let percent_of_parent = if parent_bytes == 0 {
        0.0
    } else {
        (agg.bytes as f64 / parent_bytes as f64 * 100.0) as f32
    };
    let percent_of_root = if root_bytes == 0 {
        0.0
    } else {
        (agg.bytes as f64 / root_bytes as f64 * 100.0) as f32
    };

    let path_str = agg.path.to_string_lossy().to_string();
    let last_modified = epoch_to_iso(agg.last_modified);

    if depth >= max_depth_emit {
        return TreemapNode {
            name: agg.name,
            path: path_str,
            size_bytes: agg.bytes,
            kind: agg.kind,
            extension: agg.extension,
            file_count: agg.file_count,
            dir_count: agg.dir_count,
            last_modified,
            percent_of_parent,
            percent_of_root,
            children: Vec::new(),
            truncated: !agg.children.is_empty(),
            error: agg.error,
        };
    }

    let own_bytes = agg.bytes;
    let mut kept_children = Vec::new();
    let mut dropped_bytes: u64 = 0;
    let mut dropped_files: u64 = 0;

    // Ordenar hijos por bytes desc antes de podar para que los más grandes
    // queden arriba (mejora layout del treemap).
    let mut sorted_children = agg.children;
    sorted_children.sort_by_key(|a| std::cmp::Reverse(a.bytes));

    for c in sorted_children {
        if c.bytes < min_size_bytes && matches!(c.kind, NodeKind::File) {
            dropped_bytes = dropped_bytes.saturating_add(c.bytes);
            dropped_files = dropped_files.saturating_add(c.file_count);
            continue;
        }
        if c.bytes < min_size_bytes && matches!(c.kind, NodeKind::Dir) {
            // Carpetas por debajo del threshold también se agregan.
            dropped_bytes = dropped_bytes.saturating_add(c.bytes);
            dropped_files = dropped_files.saturating_add(c.file_count);
            continue;
        }
        kept_children.push(prune(
            c,
            own_bytes,
            root_bytes,
            min_size_bytes,
            max_depth_emit,
            depth + 1,
        ));
    }

    if dropped_bytes > 0 {
        let parent_path = agg.path.clone();
        let synthetic_path = parent_path.join("(otros)").to_string_lossy().to_string();
        kept_children.push(TreemapNode {
            name: "(otros, pequeños)".to_string(),
            path: synthetic_path,
            size_bytes: dropped_bytes,
            kind: NodeKind::Dir,
            extension: None,
            file_count: dropped_files,
            dir_count: 0,
            last_modified: None,
            percent_of_parent: if own_bytes == 0 {
                0.0
            } else {
                (dropped_bytes as f64 / own_bytes as f64 * 100.0) as f32
            },
            percent_of_root: if root_bytes == 0 {
                0.0
            } else {
                (dropped_bytes as f64 / root_bytes as f64 * 100.0) as f32
            },
            children: Vec::new(),
            truncated: false,
            error: None,
        });
    }

    TreemapNode {
        name: agg.name,
        path: path_str,
        size_bytes: agg.bytes,
        kind: agg.kind,
        extension: agg.extension,
        file_count: agg.file_count,
        dir_count: agg.dir_count,
        last_modified,
        percent_of_parent,
        percent_of_root,
        children: kept_children,
        truncated: false,
        error: agg.error,
    }
}

// ── computo de stats globales ─────────────────────────────────────────────

pub(crate) fn categorize_extension(ext: &str) -> ExtCategory {
    match ext.to_lowercase().as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "mp3"
        | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => ExtCategory::Media,
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "svg" | "ico"
        | "heic" | "heif" | "raw" | "cr2" | "nef" => ExtCategory::Image,
        "ts" | "tsx" | "js" | "jsx" | "rs" | "py" | "go" | "java" | "kt" | "cpp" | "cc" | "c"
        | "h" | "hpp" | "cs" | "swift" | "rb" | "php" | "css" | "scss" | "html" | "vue"
        | "svelte" | "lua" | "sh" | "ps1" | "json" | "toml" | "yaml" | "yml" | "xml" | "md"
        | "sql" => ExtCategory::Code,
        "pdf" | "docx" | "doc" | "xlsx" | "xls" | "pptx" | "ppt" | "odt" | "ods" | "odp"
        | "txt" | "rtf" | "epub" | "mobi" => ExtCategory::Docs,
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab" | "lz" | "lzma"
        | "zst" => ExtCategory::Archive,
        "exe" | "dll" | "msi" | "msix" | "appx" | "bat" | "cmd" | "sys" | "drv" => {
            ExtCategory::Executable
        }
        "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "sdf" | "dat" => ExtCategory::Database,
        "ttf" | "otf" | "woff" | "woff2" | "fon" => ExtCategory::Font,
        "obj" | "fbx" | "blend" | "stl" | "gltf" | "glb" | "dae" | "3ds" | "max" => {
            ExtCategory::ThreeD
        }
        _ => ExtCategory::Other,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_report(
    scan_id: &str,
    root_path: &str,
    started_at: Instant,
    bytes_total: u64,
    files_total: u64,
    dirs_total: u64,
    drive_total: u64,
    drive_free: u64,
    by_ext_root: HashMap<String, (u64, u64)>,
    largest_folders: Vec<FolderStat>,
    largest_files: Vec<FileStat>,
    age: AgeDistribution,
    errors: Vec<String>,
) -> DiskAnalysisReport {
    let mut top_extensions: Vec<ExtensionStat> = by_ext_root
        .into_iter()
        .map(|(ext, (bytes, fc))| ExtensionStat {
            extension: ext.clone(),
            category: categorize_extension(&ext),
            bytes,
            file_count: fc,
            percent: if bytes_total == 0 {
                0.0
            } else {
                (bytes as f64 / bytes_total as f64 * 100.0) as f32
            },
        })
        .collect();
    top_extensions.sort_by_key(|e| std::cmp::Reverse(e.bytes));
    top_extensions.truncate(20);

    DiskAnalysisReport {
        scan_id: scan_id.to_string(),
        root: root_path.to_string(),
        scanned_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        duration_ms: started_at.elapsed().as_millis() as u64,
        total_bytes: bytes_total,
        total_files: files_total,
        total_dirs: dirs_total,
        free_bytes: drive_free,
        drive_total_bytes: drive_total,
        top_extensions,
        largest_folders,
        largest_files,
        age_distribution: age,
        errors,
    }
}

fn collect_folder_stats(agg: &Aggregate, out: &mut Vec<FolderStat>, root_bytes: u64, depth: u32) {
    if depth == 0 {
        // Saltar el root para que no domine la lista
    } else if matches!(agg.kind, NodeKind::Dir) && agg.bytes > 0 {
        out.push(FolderStat {
            path: agg.path.to_string_lossy().to_string(),
            name: agg.name.clone(),
            bytes: agg.bytes,
            file_count: agg.file_count,
            percent: if root_bytes == 0 {
                0.0
            } else {
                (agg.bytes as f64 / root_bytes as f64 * 100.0) as f32
            },
        });
    }
    for c in &agg.children {
        collect_folder_stats(c, out, root_bytes, depth + 1);
    }
}

fn collect_file_stats(agg: &Aggregate, out: &mut Vec<FileStat>) {
    if matches!(agg.kind, NodeKind::File) && agg.own_bytes > 0 {
        out.push(FileStat {
            path: agg.path.to_string_lossy().to_string(),
            name: agg.name.clone(),
            bytes: agg.own_bytes,
            extension: agg.extension.clone(),
            last_modified: epoch_to_iso(agg.last_modified),
        });
    }
    for c in &agg.children {
        collect_file_stats(c, out);
    }
}

fn collect_age(agg: &Aggregate, age: &mut AgeDistribution, now_secs: u64) {
    if matches!(agg.kind, NodeKind::File) && agg.last_modified > 0 {
        let age_secs = now_secs.saturating_sub(agg.last_modified);
        let bucket = pick_bucket(age, age_secs);
        bucket.bytes = bucket.bytes.saturating_add(agg.own_bytes);
        bucket.file_count = bucket.file_count.saturating_add(1);
    }
    for c in &agg.children {
        collect_age(c, age, now_secs);
    }
}

fn pick_bucket(age: &mut AgeDistribution, age_secs: u64) -> &mut AgeBucket {
    const D: u64 = 86_400;
    if age_secs < 7 * D {
        &mut age.last_7_days
    } else if age_secs < 30 * D {
        &mut age.last_30_days
    } else if age_secs < 90 * D {
        &mut age.last_90_days
    } else if age_secs < 365 * D {
        &mut age.last_1_year
    } else if age_secs < 5 * 365 * D {
        &mut age.last_5_years
    } else {
        &mut age.older
    }
}

// ── entrypoint ────────────────────────────────────────────────────────────

pub async fn run_analysis(
    app: tauri::AppHandle,
    input: BuildTreemapInput,
) -> AppResult<DiskAnalysisResult> {
    let scan_id = if input.scan_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        input.scan_id.clone()
    };

    let root_path = dunce::canonicalize(Path::new(&input.root))
        .unwrap_or_else(|_| PathBuf::from(&input.root));
    let root_path_str = root_path.to_string_lossy().to_string();

    let cancel = register_scan(&scan_id)?;

    // Datos del drive para el report (capacidad + free).
    #[cfg(target_os = "windows")]
    let (drive_total, drive_free) = {
        crate::platform::filesystem::drive_capacity_and_free(&root_path_str)
            .unwrap_or((0, 0))
    };
    #[cfg(not(target_os = "windows"))]
    let (drive_total, drive_free) = (0u64, 0u64);

    let started_at = Instant::now();
    let min_size_bytes = input.min_size_mb.saturating_mul(1024 * 1024);
    let max_depth_emit = input.max_depth_emit;
    let follow_reparse_points = input.follow_reparse_points;

    let scan_id_for_walk = scan_id.clone();
    let app_clone = app.clone();
    let root_clone = root_path.clone();

    // El walker es síncrono y puede tardar minutos. Lo metemos en
    // `spawn_blocking` para no bloquear el runtime tokio que sirve los demás
    // comandos.
    let aggregate_res = tokio::task::spawn_blocking(move || {
        let ctx = ScanCtx {
            scan_id: scan_id_for_walk,
            cancel,
            app: &app_clone,
            follow_reparse_points,
            started_at: Instant::now(),
            last_emit: Mutex::new(Instant::now()),
            current_path: Mutex::new(root_clone.to_string_lossy().to_string()),
            bytes: Default::default(),
            files: Default::default(),
            dirs: Default::default(),
            counter: Default::default(),
            errors: Mutex::new(Vec::new()),
        };
        let agg = walk(&root_clone, &ctx);
        // emisión final de progreso antes de salir.
        let payload = DiskScanProgressPayload {
            scan_id: ctx.scan_id.clone(),
            bytes_scanned: ctx.bytes.load(Ordering::Relaxed),
            files_scanned: ctx.files.load(Ordering::Relaxed),
            dirs_scanned: ctx.dirs.load(Ordering::Relaxed),
            current_path: ctx
                .current_path
                .lock()
                .map(|g| g.clone())
                .unwrap_or_default(),
            elapsed_ms: ctx.started_at.elapsed().as_millis() as u64,
        };
        let _ = ctx.app.emit("disk:progress", payload);
        let errors = ctx
            .errors
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        (agg, errors, ctx.cancel.load(Ordering::Relaxed))
    })
    .await;

    unregister_scan(&scan_id);

    let (aggregate, errors, was_cancelled) = match aggregate_res {
        Ok(v) => v,
        Err(e) => {
            let _ = app.emit(
                "disk:error",
                serde_json::json!({
                    "scanId": scan_id,
                    "error": format!("scan task panicked: {e}"),
                }),
            );
            return Err(AppError::External(format!("scan task panicked: {e}")));
        }
    };

    if was_cancelled {
        let _ = app.emit(
            "disk:error",
            serde_json::json!({ "scanId": scan_id, "error": "cancelled" }),
        );
        return Err(AppError::Cancelled);
    }

    // Computar stats sobre el aggregate (antes de podar).
    let root_bytes = aggregate.bytes;
    let total_files = aggregate.file_count;
    let total_dirs = aggregate.dir_count;
    let by_ext_root = aggregate.by_ext.clone();

    let mut folders: Vec<FolderStat> = Vec::new();
    collect_folder_stats(&aggregate, &mut folders, root_bytes, 0);
    folders.sort_by_key(|f| std::cmp::Reverse(f.bytes));
    folders.truncate(20);

    let mut files: Vec<FileStat> = Vec::new();
    collect_file_stats(&aggregate, &mut files);
    files.sort_by_key(|f| std::cmp::Reverse(f.bytes));
    files.truncate(20);

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut age = AgeDistribution::default();
    collect_age(&aggregate, &mut age, now_secs);

    let report = build_report(
        &scan_id,
        &root_path_str,
        started_at,
        root_bytes,
        total_files,
        total_dirs,
        drive_total,
        drive_free,
        by_ext_root,
        folders,
        files,
        age,
        errors,
    );

    // Podar y serializar.
    let root_node = prune(aggregate, root_bytes, root_bytes, min_size_bytes, max_depth_emit, 0);

    let result = DiskAnalysisResult {
        report,
        root: root_node,
    };

    let _ = app.emit(
        "disk:complete",
        serde_json::json!({ "scanId": scan_id }),
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorize_known_extensions() {
        assert!(matches!(categorize_extension("mkv"), ExtCategory::Media));
        assert!(matches!(categorize_extension("PNG"), ExtCategory::Image));
        assert!(matches!(categorize_extension("rs"), ExtCategory::Code));
        assert!(matches!(categorize_extension("exe"), ExtCategory::Executable));
        assert!(matches!(categorize_extension("ttf"), ExtCategory::Font));
        assert!(matches!(categorize_extension("unknown-ext"), ExtCategory::Other));
    }

    #[test]
    fn epoch_to_iso_zero_is_none() {
        assert!(epoch_to_iso(0).is_none());
        assert!(epoch_to_iso(1_700_000_000).is_some());
    }

    #[test]
    fn pick_bucket_boundaries() {
        let mut age = AgeDistribution::default();
        const D: u64 = 86_400;
        pick_bucket(&mut age, 0).bytes += 1;
        pick_bucket(&mut age, 7 * D).bytes += 1;
        pick_bucket(&mut age, 30 * D).bytes += 1;
        pick_bucket(&mut age, 90 * D).bytes += 1;
        pick_bucket(&mut age, 365 * D).bytes += 1;
        pick_bucket(&mut age, 5 * 365 * D).bytes += 1;
        assert_eq!(age.last_7_days.bytes, 1);
        assert_eq!(age.last_30_days.bytes, 1);
        assert_eq!(age.last_90_days.bytes, 1);
        assert_eq!(age.last_1_year.bytes, 1);
        assert_eq!(age.last_5_years.bytes, 1);
        assert_eq!(age.older.bytes, 1);
    }
}
