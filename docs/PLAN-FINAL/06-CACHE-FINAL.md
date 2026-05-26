# 06 — Cache Cleaner (cierre y pulido)

> **Posición:** 6/14. Único módulo destructivo ya parcialmente funcional.
> **Dependencias:** [00-CATALOGOS](00-CATALOGOS.md), [01-RESTORE-POINTS](01-RESTORE-POINTS.md), [02-AUDIT-LOG](02-AUDIT-LOG.md).
> **Output:** filtros del catálogo aplicados de verdad, presets, integración con audit, UX final.

---

## 1. Resumen ejecutivo

A diferencia de los módulos anteriores, este YA tiene backend funcional (`domain::cache::scan` y `clean`). El plan documenta el **cierre**: lo que falta para llegar a v1.0.

Estado actual:

| Componente | Estado |
|---|---|
| `domain::cache::list_locations` | ✅ |
| `domain::cache::scan` | ✅ (sin filtros del catálogo) |
| `domain::cache::clean` | ✅ (sin filtros, sin audit, sin restore point automático) |
| `domain::cache::expand_path` | ✅ |
| `platform::filesystem::delete_recursive_robust` | ✅ |
| UI lista + scan + clean | ✅ |
| Filtros del catálogo (exclude-extension, older-than-days) | ❌ |
| Presets (system / browser / dev) | ❌ |
| Integración con audit log | ❌ |
| Integración con restore point automático | ❌ |
| Detección de procesos que bloquean archivos | ❌ |
| Estimación previa de espacio a liberar antes de Clean | 🟡 (existe en scan pero no se muestra agregado) |
| Confirmación pre-clean | ❌ |

---

## 2. Diagnóstico de gaps

### 2.1 Filtros del catálogo se ignoran

El JSON tiene:
```json
"filters": [
  { "kind": "exclude-extension", "value": ".log" },
  { "kind": "older-than-days", "value": "30" }
]
```

El código actual de `scan()` y `clean()` **no aplica estos filtros**. Resulta que el conteo de archivos `matched_after_filters` siempre = `file_count`.

### 2.2 Cleanup procesa archivos en uso de forma cruda

`delete_recursive_robust` ya maneja archivos en uso (los marca para `MoveFileEx` en next reboot), pero la UI no muestra qué procesos los tienen abiertos. Útil para que el usuario decida cerrar Edge antes de limpiar caché de Edge.

### 2.3 No hay restore point antes de Clean

Las cachés son borrados de archivos, no de registro/servicios. Un restore point **no las recupera**. Pero la regla del proyecto es "toda operación destructiva crea restore point". Hay dos opciones:

- **Opcional:** checkbox "Crear punto de restauración" en la UI. Default ON.
- **Skip:** el restore point no aporta para cache. Skippearlo es defendible.

**Decisión:** opcional con default ON. Coherencia con resto de módulos. El usuario puede desmarcarlo conscientemente.

### 2.4 Sin presets

La lista actual son ~30 ubicaciones planas. Útil agrupar:

- **Sistema** (Windows Update, DISM, CBS, Prefetch, WER) — admin.
- **Usuario** (TEMP, INetCache, thumbcache) — sin admin.
- **Navegadores** (Edge cache, Chrome cache si existe) — sin admin, requiere cerrar el browser.
- **Package managers** (npm, pnpm, yarn, pip, cargo, Visual Studio Component Cache) — solo si detecta los toolchains.
- **Media/tools** (Font cache, Spotify, etc.) — opcional.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| Filtros aplicados en `filesystem::directory_stats_filtered` (nuevo) | Cálculo correcto del "matched_after_filters" |
| `clean` respeta los filtros del catálogo | Coherencia con `scan` |
| Audit log: 1 entry por **run** (no por location) | Compatibilidad con UX del log; reverse recipe = Noop con seq de restore |
| Restore point opcional, default ON | Coherencia con principio "transparencia" |
| Detección de procesos via `restart manager API` o `handle.exe` (opcional v1.1) | v1.0: solo reportar "X archivos en uso, próximo reboot" |
| Presets en el frontend (cliente) basados en `category` del catálogo | Backend permanece neutral |

---

## 4. Modelo de datos (extender modelos existentes)

```rust
// models/cache.rs — añadir

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheInput {
    pub ids: Vec<String>,
    pub dry_run: bool,
    pub create_restore_point: bool,
    pub force_close_processes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub restore_point_id: Option<u32>,
    pub per_location: Vec<PerLocationResult>,
    pub total_bytes_freed: u64,
    pub total_files_deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerLocationResult {
    pub id: String,
    pub status: String,    // "ok" | "partial" | "failed" | "missing" | "not-in-catalog" | "dry-run"
    pub bytes_freed: u64,
    pub files_deleted: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgressEvent {
    pub run_id: String,
    pub level: String,         // "info" | "warn" | "error"
    pub location_display: String,
    pub message: String,
    pub bytes_delta: u64,
    pub files_delta: u64,
}
```

---

## 5. Plan UX

### 5.1 Pantalla principal (rediseño)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Limpieza de caché                                                             │
│ 12 ubicaciones · 28.4 GB detectados · 13.2 GB liberables                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Presets:  [Sistema (5)]  [Usuario (8)]  [Navegadores (3)]  [Package mgrs (4)]│
│           [Todo (30)]    [Nada]                                                │
│                                                                                │
│ Sistema (5)                                                                    │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ ☑ Windows Update — Download                  HIGH    4.8 GB    [Detalle] │ │
│ │ ☑ DISM / CBS logs                            MED     1.2 GB    [Detalle] │ │
│ │ ☑ Prefetch                                   LOW    180 MB     [Detalle] │ │
│ │ ☐ Windows Error Reporting                    LOW     45 MB     [Detalle] │ │
│ │ ☐ Driver Store Repository                    HIGH    3.4 GB    [Detalle] │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                                │
│ Usuario (8)                                                                    │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ ☑ %TEMP%                                     LOW    2.1 GB     [Detalle] │ │
│ │ ☑ INetCache                                  LOW   850 MB      [Detalle] │ │
│ │ ☑ Thumbcache                                 LOW    320 MB     [Detalle] │ │
│ │ ...                                                                       │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                                │
│ ────────────────────────────────────────────────────────────────────────────  │
│ ☑ Crear punto de restauración antes  ☐ Forzar cerrar procesos que bloquean    │
│ ☐ Dry-run                                                                      │
│                                                                                │
│ [Re-escanear seleccionadas]                              [Limpiar (13.2 GB →)]│
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Modal "Detalle" de ubicación

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Windows Update — Download                                                │
├──────────────────────────────────────────────────────────────────────────┤
│  Path:        %WINDIR%\SoftwareDistribution\Download                      │
│  Resolved:    C:\Windows\SoftwareDistribution\Download                    │
│  Categoría:   System                                                      │
│  Riesgo:      Bajo                                                        │
│  Requiere admin: Sí                                                       │
│  Tamaño:      4.8 GB (12,853 archivos)                                    │
│                                                                           │
│  Filtros aplicados:                                                       │
│   • Excluir extensión .log (preserva logs de troubleshooting)             │
│   • Solo archivos > 7 días                                                │
│                                                                           │
│  Tras filtros: 4.2 GB (11,043 archivos)                                   │
│                                                                           │
│  Precondiciones:                                                          │
│   • Servicio 'wuauserv' debe estar detenido (se hará automáticamente)    │
│                                                                           │
│  Consecuencias:                                                           │
│   • La próxima vez que Update busque actualizaciones, re-descarga.        │
│   • No afecta updates ya instaladas.                                      │
│                                                                           │
│                                                              [Cerrar]     │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Modal pre-clean

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Limpiar caché                                                            │
├──────────────────────────────────────────────────────────────────────────┤
│  Vas a liberar **13.2 GB** de 8 ubicaciones:                              │
│                                                                           │
│   • Windows Update — Download    4.2 GB                                   │
│   • DISM / CBS logs              1.0 GB                                   │
│   • Prefetch                       150 MB                                 │
│   • %TEMP%                       2.1 GB                                   │
│   • ... (4 más)                                                           │
│                                                                           │
│  ☑ Crear punto de restauración                                            │
│  ☐ Forzar cerrar procesos que bloquean archivos                           │
│  ☐ Dry-run                                                                │
│                                                                           │
│  Esta operación no es completamente reversible. Archivos en uso se        │
│  marcarán para borrado en el próximo reboot.                              │
│                                                                           │
│  [Cancelar]                                              [Limpiar →]      │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Plan de implementación por fases

### Fase 1 — Filtros del catálogo

#### Paso 1.1 — Helper `apply_filters` en `domain::cache`

**Archivo:** `src-tauri/src/domain/cache.rs`

```rust
use crate::models::cache::{CacheFilter, CacheLocation};
use std::path::Path;
use std::time::SystemTime;

pub fn file_passes_filters(path: &Path, metadata: &std::fs::Metadata, filters: &[CacheFilter]) -> bool {
    for f in filters {
        match f.kind.as_str() {
            "exclude-extension" => {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if format!(".{}", ext).eq_ignore_ascii_case(&f.value) {
                    return false;
                }
            }
            "exclude-name" => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.eq_ignore_ascii_case(&f.value) {
                    return false;
                }
            }
            "older-than-days" => {
                let days: u64 = f.value.parse().unwrap_or(0);
                let now = SystemTime::now();
                let modified = metadata.modified().ok();
                if let Some(m) = modified {
                    let age_days = now.duration_since(m).map(|d| d.as_secs() / 86400).unwrap_or(0);
                    if age_days < days {
                        return false;
                    }
                }
            }
            _ => {}  // filtro desconocido: ignorar (tolerante)
        }
    }
    true
}
```

#### Paso 1.2 — `directory_stats_filtered` en `platform::filesystem`

```rust
// platform/filesystem.rs — añadir
use std::path::Path;
use walkdir::WalkDir;

pub fn directory_stats_filtered<F>(
    path: &Path,
    follow_reparse: bool,
    mut keep: F,
) -> AppResult<(u64, u64, u64)>
where
    F: FnMut(&Path, &std::fs::Metadata) -> bool,
{
    let mut total_bytes = 0u64;
    let mut total_files = 0u64;
    let mut total_dirs = 0u64;

    let walker = WalkDir::new(path).follow_links(follow_reparse).into_iter();
    for entry in walker.filter_entry(|_| true) {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let p = entry.path();
        let metadata = match entry.metadata() { Ok(m) => m, Err(_) => continue };
        if metadata.is_dir() {
            total_dirs += 1;
            continue;
        }
        if !keep(p, &metadata) {
            continue;
        }
        total_files += 1;
        total_bytes = total_bytes.saturating_add(metadata.len());
    }
    Ok((total_bytes, total_files, total_dirs))
}
```

#### Paso 1.3 — Refactor `scan` para usar filtros

```rust
// domain/cache.rs — refactor scan()

pub fn scan(ids: &[String]) -> AppResult<Vec<CacheScanReport>> {
    let catalog_items = catalog::load_cache_locations()?;
    let mut reports = Vec::new();

    for id in ids {
        let Some(loc) = catalog_items.iter().find(|l| &l.id == id) else { continue; };
        let resolved = expand_path(&loc.path);
        let path = Path::new(&resolved);
        let exists = path.exists();

        let (raw_bytes, raw_files, _) = if exists {
            filesystem::directory_stats(path, false).unwrap_or((0, 0, 0))
        } else { (0, 0, 0) };

        let (filtered_bytes, filtered_files) = if exists && !loc.filters.is_empty() {
            let (b, f, _) = filesystem::directory_stats_filtered(path, false, |p, m| {
                file_passes_filters(p, m, &loc.filters)
            }).unwrap_or((raw_bytes, raw_files, 0));
            (b, f)
        } else {
            (raw_bytes, raw_files)
        };

        reports.push(CacheScanReport {
            id: id.clone(),
            resolved_path: resolved,
            exists,
            bytes: raw_bytes,
            file_count: raw_files,
            matched_after_filters: filtered_files,
            bytes_after_filters: filtered_bytes,
        });
    }
    Ok(reports)
}
```

#### Paso 1.4 — `delete_recursive_filtered`

```rust
// platform/filesystem.rs

pub fn delete_recursive_filtered<F>(path: &Path, mut keep: F) -> DeleteResult
where
    F: FnMut(&Path, &std::fs::Metadata) -> bool,
{
    let mut result = DeleteResult::default();
    let walker = WalkDir::new(path).contents_first(true);

    for entry in walker {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let p = entry.path().to_path_buf();
        let metadata = match entry.metadata() { Ok(m) => m, Err(_) => continue };

        if metadata.is_dir() {
            // Borrar dir solo si está vacío y matchea
            let _ = std::fs::remove_dir(&p);
            continue;
        }
        if !keep(&p, &metadata) { continue; }

        match std::fs::remove_file(&p) {
            Ok(_) => {
                result.bytes_freed = result.bytes_freed.saturating_add(metadata.len());
                result.files_deleted += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Marcar para reboot
                mark_for_delete_next_reboot(&p);
                result.pending_reboot.push(p.to_string_lossy().into_owned());
            }
            Err(e) => result.errors.push(format!("{}: {}", p.display(), e)),
        }
    }
    result
}
```

#### Paso 1.5 — Refactor `clean` con filtros + audit + restore

```rust
// domain/cache.rs

use crate::domain::audit;
use crate::models::restore::ReverseRecipe;

pub fn clean<F>(input: &CleanCacheInput, mut emit: F) -> AppResult<CleanReport>
where
    F: FnMut(&str, &str, &str, u64, u64),
{
    let started_at = Utc::now().to_rfc3339();
    let run_id = Uuid::new_v4().to_string();

    let restore_seq = if input.create_restore_point && !input.dry_run {
        crate::platform::restore_point::create(
            &format!("ClearTool — cache clean ({} ubicaciones)", input.ids.len()),
            12, true,
        ).ok()
    } else { None };

    let catalog_items = catalog::load_cache_locations()?;
    let mut per_location: Vec<PerLocationResult> = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut total_files: u64 = 0;
    let mut items_affected: Vec<String> = Vec::new();

    for id in &input.ids {
        let Some(loc) = catalog_items.iter().find(|l| &l.id == id) else {
            per_location.push(PerLocationResult {
                id: id.clone(), status: "not-in-catalog".into(),
                bytes_freed: 0, files_deleted: 0, errors: vec!["allowlist".into()],
            });
            continue;
        };
        let resolved = expand_path(&loc.path);
        let path = Path::new(&resolved);
        emit("info", &loc.display_name, &format!("Procesando: {}", resolved), 0, 0);

        if !path.exists() {
            emit("warn", &loc.display_name, "Path no existe", 0, 0);
            per_location.push(PerLocationResult {
                id: id.clone(), status: "missing".into(),
                bytes_freed: 0, files_deleted: 0, errors: vec![],
            });
            continue;
        }

        // Precondiciones (servicio detenido, etc.) — para v1.0, solo loguear.
        for pre in &loc.preconditions {
            log::info!("[{}] precondition: kind={} value={}", id, pre.kind, pre.value);
        }

        if input.dry_run {
            let (bytes, files, _) = if !loc.filters.is_empty() {
                crate::platform::filesystem::directory_stats_filtered(path, false, |p, m| {
                    file_passes_filters(p, m, &loc.filters)
                }).unwrap_or((0, 0, 0))
            } else {
                crate::platform::filesystem::directory_stats(path, false).unwrap_or((0, 0, 0))
            };
            emit("info", &loc.display_name,
                &format!("[dry-run] {} archivos, {}", files, format_bytes(bytes)),
                0, 0);
            per_location.push(PerLocationResult {
                id: id.clone(), status: "dry-run".into(),
                bytes_freed: bytes, files_deleted: files, errors: vec![],
            });
            continue;
        }

        // Borrado real con filtros
        let result = if !loc.filters.is_empty() {
            crate::platform::filesystem::delete_recursive_filtered(path, |p, m| {
                file_passes_filters(p, m, &loc.filters)
            })
        } else {
            crate::platform::filesystem::delete_recursive_robust(path)
        };

        total_bytes = total_bytes.saturating_add(result.bytes_freed);
        total_files = total_files.saturating_add(result.files_deleted);

        emit("info", &loc.display_name,
            &format!("OK: {} archivos eliminados, {}",
                result.files_deleted, format_bytes(result.bytes_freed)),
            result.bytes_freed, result.files_deleted);

        for pending in &result.pending_reboot {
            emit("warn", &loc.display_name,
                &format!("En uso, próximo reboot: {}", pending), 0, 0);
        }
        for err in &result.errors {
            emit("error", &loc.display_name, err, 0, 0);
        }

        let status = if result.errors.is_empty() && result.pending_reboot.is_empty() {
            "ok"
        } else if result.bytes_freed > 0 { "partial" } else { "failed" };

        let mut errors = result.errors.clone();
        if !result.pending_reboot.is_empty() {
            errors.push(format!("{} archivos pendientes reboot", result.pending_reboot.len()));
        }

        items_affected.push(id.clone());
        per_location.push(PerLocationResult {
            id: id.clone(), status: status.into(),
            bytes_freed: result.bytes_freed, files_deleted: result.files_deleted, errors,
        });
    }

    let finished_at = Utc::now().to_rfc3339();

    // Audit entry — uno por run
    if !input.dry_run {
        let entry = audit::make_entry(
            "cache", "clean", false, restore_seq,
            items_affected.clone(),
            ReverseRecipe::Noop {
                reason: format!("borrado de archivos; usar restore point seq #{} si aplica",
                    restore_seq.map(|s| s.to_string()).unwrap_or_else(|| "<sin punto>".into())),
            },
            "success", None,
        );
        let _ = audit::write_entry(&entry);
    }

    Ok(CleanReport {
        run_id, started_at, finished_at,
        restore_point_id: restore_seq, per_location,
        total_bytes_freed: total_bytes, total_files_deleted: total_files,
    })
}
```

### Fase 2 — Eventos de progreso unificados

#### Paso 2.1 — Emit del comando IPC

**Archivo:** `src-tauri/src/ipc/cache.rs`

```rust
#[tauri::command]
pub async fn clean_cache_locations(
    app: AppHandle,
    input: CleanCacheInput,
) -> AppResult<CleanReport> {
    let app_clone = app.clone();
    let run_id_holder = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let h = run_id_holder.clone();

    let report = domain::cache::clean(&input, move |level, location_display, message, bytes, files| {
        let _ = app_clone.emit("cache:progress", CleanProgressEvent {
            run_id: h.lock().unwrap().clone(),
            level: level.to_string(),
            location_display: location_display.to_string(),
            message: message.to_string(),
            bytes_delta: bytes,
            files_delta: files,
        });
    })?;

    *run_id_holder.lock().unwrap() = report.run_id.clone();
    Ok(report)
}
```

### Fase 3 — Frontend: presets, modales, log streaming

#### Paso 3.1 — Hook `useCacheLocations` ampliado

**Archivo:** `src/features/cache-cleaner/use-cache-cleaner.ts` (nuevo)

```ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  listCacheLocations, scanCacheLocations, cleanCacheLocations,
  type CacheLocation, type CacheScanReport, type CleanCacheInput,
} from "../../api";

export function useCacheLocations() {
  return useQuery({ queryKey: ["cache", "locations"], queryFn: listCacheLocations });
}

export function useScanCache() {
  return useMutation({ mutationFn: (ids: string[]) => scanCacheLocations(ids) });
}

export function useCleanCache() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CleanCacheInput) => cleanCacheLocations(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["cache"] });
      qc.invalidateQueries({ queryKey: ["audit"] });
    },
  });
}

export interface CacheProgressLog {
  level: "info" | "warn" | "error";
  locationDisplay: string;
  message: string;
  timestamp: string;
}

export function useCacheProgress() {
  const [log, setLog] = useState<CacheProgressLog[]>([]);
  useEffect(() => {
    let un: UnlistenFn | null = null;
    (async () => {
      un = await listen<any>("cache:progress", (e) => {
        setLog((prev) => [...prev, {
          level: e.payload.level,
          locationDisplay: e.payload.locationDisplay,
          message: e.payload.message,
          timestamp: new Date().toISOString(),
        }].slice(-500));  // máximo 500 entries para evitar memory bloat
      });
    })();
    return () => { un?.(); };
  }, []);
  const clear = () => setLog([]);
  return { log, clear };
}
```

#### Paso 3.2 — Categorización en cliente

```ts
// src/features/cache-cleaner/lib/grouping.ts
import type { CacheLocation } from "../../../api";

export const CATEGORIES = ["system", "user", "browser", "package-manager", "media", "tools"] as const;
export type Category = (typeof CATEGORIES)[number];

export const CATEGORY_LABELS: Record<Category, string> = {
  "system": "Sistema",
  "user": "Usuario",
  "browser": "Navegadores",
  "package-manager": "Package managers",
  "media": "Media y tools",
  "tools": "Herramientas",
};

export function groupByCategory(locations: CacheLocation[]): Map<Category, CacheLocation[]> {
  const map = new Map<Category, CacheLocation[]>();
  for (const c of CATEGORIES) map.set(c, []);
  for (const l of locations) {
    const cat = (CATEGORIES as readonly string[]).includes(l.category)
      ? (l.category as Category)
      : "tools";
    map.get(cat)!.push(l);
  }
  return map;
}
```

#### Paso 3.3 — Componentes a crear o ajustar

```
src/features/cache-cleaner/
  cache-page.tsx                  ← rediseñar §5.1
  use-cache-cleaner.ts            ← nuevo
  lib/grouping.ts                 ← nuevo
  components/
    category-section.tsx
    location-row.tsx
    location-detail-modal.tsx     ← §5.2
    clean-confirm-modal.tsx       ← §5.3
    progress-log-panel.tsx        ← streaming en vivo
    preset-bar.tsx
    risk-badge.tsx
```

### Fase 4 — Mejora opcional: detección de procesos bloqueantes

Para v1.0, **skipped**. Demasiado complejo: requiere `Restart Manager API` (`RmStartSession`, `RmRegisterResources`, `RmGetList`). Documentar como mejora v1.1.

---

## 7. Tests

### 7.1 Test del filter helper

```rust
// src-tauri/tests/cache_filters.rs

use cleartool::models::cache::CacheFilter;
use cleartool::domain::cache::file_passes_filters;
use std::path::Path;

fn dummy_metadata(modified_ago_days: u64) -> std::fs::Metadata {
    // Helper: crear archivo en tmp + leer metadata real.
    let tmp = std::env::temp_dir().join("cache-test.txt");
    std::fs::write(&tmp, b"x").unwrap();
    let m = std::fs::metadata(&tmp).unwrap();
    let _ = std::fs::remove_file(&tmp);
    m
}

#[test]
fn exclude_extension_funciona() {
    let filters = vec![CacheFilter { kind: "exclude-extension".into(), value: ".log".into() }];
    let m = dummy_metadata(0);
    assert!(!file_passes_filters(Path::new("a.log"), &m, &filters));
    assert!(file_passes_filters(Path::new("a.tmp"), &m, &filters));
}

#[test]
fn filtros_vacios_dejan_pasar_todo() {
    let m = dummy_metadata(0);
    assert!(file_passes_filters(Path::new("anything.zzz"), &m, &[]));
}
```

### 7.2 Test manual VM

1. Win11 limpia + admin.
2. `/cache` → ver lista categorizada (presets visibles).
3. Click "Preset Usuario" → 8 marcados.
4. Click "Re-escanear seleccionadas" → cifras se actualizan.
5. Click "Limpiar" → modal preview.
6. Check "Crear punto de restauración" + uncheck dry-run.
7. Aceptar → progress log streaming filas en vivo.
8. Tras completar: cifras en sidebar actualizadas, audit log con 1 entry.

---

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Borrar caché de Edge mientras está abierto | UI sugiere cerrar; pendientes van a reboot |
| Filtro `older-than-days` no se aplica si metadata.modified() falla | `unwrap_or(0)` → archivo se considera nuevo, no se borra |
| `directory_stats_filtered` con WalkDir es lento en árboles profundos (>1M archivos) | Cancellation token: abortable si el usuario cancela |
| Cache de Windows Update bloquea borrado | Precondition "service-stopped" implementada en v1.1; v1.0 reporta `partial` |
| Path traversal via env var maliciosa (`%APPDATA%=C:\Windows`) | `expand_path` solo expande nombres conocidos; env var del SO se confía |
| Streaming log llena la UI con 100k+ entries | Cap a 500, descartar viejas |

---

## 9. Definition of Done

- [ ] `file_passes_filters` implementado en `domain::cache`.
- [ ] `platform::filesystem::directory_stats_filtered` y `delete_recursive_filtered`.
- [ ] `domain::cache::scan` aplica filtros: `matched_after_filters` distinto a `file_count` cuando hay filtros.
- [ ] `domain::cache::clean` aplica filtros + crea restore point + escribe audit entry.
- [ ] IPC: evento `cache:progress` con `runId`.
- [ ] Hook `useCacheProgress` con streaming log y cap a 500 entries.
- [ ] UI categorizada por `category` con 6 grupos.
- [ ] Preset buttons (Sistema/Usuario/Navegadores/Package mgrs/Todo/Nada).
- [ ] Modal detalle de ubicación con consequences, filters, preconditions.
- [ ] Modal pre-clean con preview de bytes/files agregados.
- [ ] Tests `cache_filters.rs` pasan.
- [ ] Test manual VM: preset Usuario + clean + audit + log streaming funciona.
- [ ] Commit `feat(cache): filtros, presets, audit y progress streaming`.

---

## 10. Próximo archivo

→ [07-SETTINGS.md](07-SETTINGS.md) — settings persistentes globales + panel del audit log + dry-run global + theme + idioma.
