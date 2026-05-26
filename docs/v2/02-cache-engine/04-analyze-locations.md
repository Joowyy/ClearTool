# Paso 04 — Pre-flight: `analyze_locations`

**Área**: 02-cache-engine
**Tiempo estimado**: 6-8 horas
**Dependencias**: Paso 01 (who_locks), Paso 03 (CleanPlan)

## Qué hacemos

Implementar la función que recorre las ubicaciones del catálogo, calcula tamaños, detecta bloqueadores, y devuelve un `CleanPlan` clasificado.

## Por qué

Es la pieza central del nuevo flow: "analiza, presenta plan, deja al usuario decidir". Reemplaza el "scan + clean directo" del v0.1.

## Archivos que tocamos

- `src-tauri/src/domain/cache.rs` (función nueva `analyze_locations`)
- `src-tauri/src/platform/filesystem.rs` (puede necesitar helpers nuevos)

## Cómo

### 1. Esqueleto de la función

```rust
// src-tauri/src/domain/cache.rs

use crate::core::{AppError, AppResult};
use crate::models::cache::*;
use crate::models::process_lock::LockingProcess;
use crate::platform::{filesystem, process_lock};
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub fn analyze_locations(ids: &[String]) -> AppResult<CleanPlan> {
    let catalog = super::catalog::load_cache_catalog()?;
    let plan_id = Uuid::new_v4().to_string();
    let generated_at = Utc::now().to_rfc3339();

    let mut ready = Vec::new();
    let mut blocked = Vec::new();
    let mut permission_issues = Vec::new();
    let mut skipped = Vec::new();

    for id in ids {
        let entry = match catalog.iter().find(|e| &e.id == id) {
            Some(e) => e,
            None => {
                skipped.push(SkippedLocation {
                    id: id.clone(),
                    display_name: id.clone(),
                    reason: SkipReason::DisallowedByAllowlist,
                });
                continue;
            }
        };

        // Check allowlist (no tocar Terminal/Store/etc.)
        if is_disallowed(&entry.id, &entry.path) {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::DisallowedByAllowlist,
            });
            continue;
        }

        // Resolve path con %VARS%
        let resolved = match resolve_env_vars(&entry.path) {
            Ok(r) => r,
            Err(_) => {
                skipped.push(SkippedLocation {
                    id: entry.id.clone(),
                    display_name: entry.display_name.clone(),
                    reason: SkipReason::DoesNotExist,
                });
                continue;
            }
        };
        let path = Path::new(&resolved);

        if !path.exists() {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::DoesNotExist,
            });
            continue;
        }

        // Preconditions: ej. "no se aplica si Outlook.exe está corriendo"
        if let Some(unmet) = check_preconditions(&entry.preconditions) {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::Precondition { name: unmet },
            });
            continue;
        }

        // Calcular tamaño + count
        let (bytes, file_count, oldest) = match scan_path_stats(path) {
            Ok(s) => s,
            Err(e) => {
                if matches!(e, AppError::Permission(_)) {
                    permission_issues.push(PermissionLocation {
                        id: entry.id.clone(),
                        display_name: entry.display_name.clone(),
                        resolved_path: resolved,
                        bytes: 0,
                        reason: format!("{}", e),
                    });
                    continue;
                }
                // Otros errores → skip
                skipped.push(SkippedLocation {
                    id: entry.id.clone(),
                    display_name: entry.display_name.clone(),
                    reason: SkipReason::DoesNotExist,
                });
                continue;
            }
        };

        if bytes == 0 {
            skipped.push(SkippedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                reason: SkipReason::Empty,
            });
            continue;
        }

        // ¿Quién bloquea?
        let lockers = process_lock::who_locks(path).unwrap_or_default();

        if !lockers.is_empty() {
            let suggested_action = suggest_action(&lockers, &entry.strategy);
            blocked.push(BlockedLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                resolved_path: resolved,
                bytes,
                locked_by: lockers,
                suggested_action,
            });
        } else {
            ready.push(ReadyLocation {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                resolved_path: resolved,
                bytes,
                file_count,
                strategy: parse_strategy(&entry.strategy, entry),
                age_oldest_file: oldest,
            });
        }
    }

    let total_estimated = ready.iter().map(|r| r.bytes).sum();
    let total_blocked = blocked.iter().map(|b| b.bytes).sum();

    Ok(CleanPlan {
        plan_id,
        generated_at,
        ready,
        blocked,
        permission_issues,
        skipped,
        total_estimated_bytes: total_estimated,
        total_blocked_bytes: total_blocked,
    })
}

// ── Helpers ──

fn is_disallowed(id: &str, path: &str) -> bool {
    // IDs hard-coded que NUNCA se tocan
    const FORBIDDEN_IDS: &[&str] = &[
        "uwp-microsoft-terminal",
        "uwp-microsoft-store",
        "uwp-windows-defender",
        "uwp-app-installer",
    ];
    if FORBIDDEN_IDS.contains(&id) { return true; }

    // Paths que contienen estos sub-paths NUNCA se tocan
    let path_lower = path.to_lowercase();
    const FORBIDDEN_PATTERNS: &[&str] = &[
        r"microsoft.windowsterminal",
        r"microsoft.windowsstore",
        r"microsoft.desktopappinstaller",
        r"microsoft.windowsdefender",
        r"\appx\\",
    ];
    FORBIDDEN_PATTERNS.iter().any(|p| path_lower.contains(p))
}

fn resolve_env_vars(path: &str) -> Result<String, std::env::VarError> {
    // Reemplaza %FOO% con env::var("FOO"). Simple parser.
    let mut out = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut var = String::new();
            while let Some(&next) = chars.peek() {
                if next == '%' { chars.next(); break; }
                var.push(next);
                chars.next();
            }
            out.push_str(&std::env::var(&var)?);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn scan_path_stats(path: &Path) -> AppResult<(u64, u32, Option<String>)> {
    let mut total_bytes: u64 = 0;
    let mut file_count: u32 = 0;
    let mut oldest: Option<SystemTime> = None;

    fn walk(p: &Path, bytes: &mut u64, count: &mut u32, oldest: &mut Option<SystemTime>) -> AppResult<()> {
        let entries = std::fs::read_dir(p).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Permission(format!("{}: {}", p.display(), e))
            } else {
                AppError::Io(format!("{}: {}", p.display(), e))
            }
        })?;
        for entry_r in entries {
            let entry = match entry_r { Ok(e) => e, Err(_) => continue };
            let md = match entry.metadata() { Ok(m) => m, Err(_) => continue };
            if md.is_dir() {
                let _ = walk(&entry.path(), bytes, count, oldest);
            } else {
                *bytes += md.len();
                *count += 1;
                if let Ok(modified) = md.modified() {
                    match *oldest {
                        Some(o) if modified < o => *oldest = Some(modified),
                        None => *oldest = Some(modified),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    walk(path, &mut total_bytes, &mut file_count, &mut oldest)?;
    let oldest_str = oldest.map(|t| DateTime::<Utc>::from(t).to_rfc3339());
    Ok((total_bytes, file_count, oldest_str))
}

fn suggest_action(lockers: &[LockingProcess], strategy: &str) -> BlockedAction {
    // Si hay un proceso main window claro, ofrecer cerrarlo
    if let Some(main) = lockers.iter().find(|p| {
        matches!(p.kind, crate::models::process_lock::LockingKind::MainWindow)
    }) {
        return BlockedAction::CloseProcess {
            pid: main.pid,
            process_name: main.name.clone(),
        };
    }
    // Si es estrategia que tolera reboot, ofrecerlo
    if strategy == "uwp-app-aware" || strategy == "system-restart-required" {
        return BlockedAction::ScheduleReboot;
    }
    // Default: skip
    BlockedAction::SkipOnly {
        reason: "Bloqueado por proceso del sistema sin UI".into(),
    }
}

fn check_preconditions(_conds: &[/* Precondition */]) -> Option<String> {
    // TODO: implementar preconditions del schema v2
    None
}

fn parse_strategy(s: &str, entry: &/* CacheEntry */ ()) -> CleanStrategy {
    // Mock: en el catálogo real se parseará desde JSON
    CleanStrategy::DirectDelete
}
```

### 2. Caveats

- `scan_path_stats` puede ser **lento** en paths grandes. Considera streaming con events `cache:scan-progress` (lo dejamos para `08-ipc-y-frontend`).
- `who_locks` puede fallar para paths que son directorios sin ningún archivo abierto directamente. En ese caso, retornará `Ok(vec![])` y la location va a `ready`. Si después `execute_plan` falla por archivos internos bloqueados, se reclasificará en runtime.

### 3. Test manual

Crear un test rápido en `tests/cache_analyze.rs` (integration test):

```rust
#[test]
fn test_analyze_skips_disallowed() {
    let ids = vec!["uwp-microsoft-terminal".to_string()];
    let plan = cleartool::domain::cache::analyze_locations(&ids).unwrap();
    assert_eq!(plan.ready.len(), 0);
    assert_eq!(plan.skipped.len(), 1);
}

#[test]
fn test_analyze_temp_dir() {
    let ids = vec!["windows-temp".to_string()]; // asumiendo está en catálogo
    let plan = cleartool::domain::cache::analyze_locations(&ids).unwrap();
    // Casi siempre habrá algo en TEMP, no debe ser vacío
    assert!(plan.ready.len() + plan.skipped.len() >= 1);
}
```

## Criterio de done

- [ ] `analyze_locations` devuelve un `CleanPlan` válido.
- [ ] Paths no existentes van a `skipped` con `DoesNotExist`.
- [ ] Paths con permission error van a `permission_issues`.
- [ ] IDs/paths en la allowlist (Terminal, Store) van a `skipped` con `DisallowedByAllowlist`.
- [ ] `who_locks` se llama para cada path no vacío.
- [ ] `total_estimated_bytes` = suma de `ready[i].bytes`.
- [ ] `total_blocked_bytes` = suma de `blocked[i].bytes`.
- [ ] Test integration pasa: caso disallowed + caso normal.

## Próximo paso

`05-execute-plan-retry.md` — ejecutar el plan.
