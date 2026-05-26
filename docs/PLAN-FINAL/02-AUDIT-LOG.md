# 02 — Audit Log y reversa fina por entrada

> **Posición:** 2/14. Prerequisito de 03, 04, 05, 06.
> **Dependencias:** [00-CATALOGOS](00-CATALOGOS.md), [01-RESTORE-POINTS](01-RESTORE-POINTS.md).
> **Output:** `domain::audit` completo, archivo JSONL append-only, UI lista + reverse desde la lista, integración cross-módulo.

---

## 1. Resumen ejecutivo

El **Audit Log** complementa los Restore Points:

- Un Restore Point revierte **todo el sistema**. Cuesta 5-15 min + reboot.
- Una entrada de Audit Log revierte **una sola operación**. Cuesta <1 s.

Cada operación destructiva debe escribir un `AuditEntry` con `reverse_recipe` — instrucciones declarativas (no código) para deshacerla.

Estado actual:

| Componente | Estado |
|---|---|
| `domain::audit::list_log` | ❌ Stub |
| `domain::audit::revert_entry` | ❌ Stub |
| Archivo JSONL `%APPDATA%\ClearTool\audit.jsonl` | ❌ No se crea |
| UI Audit Log | ❌ No existe pantalla |
| Helper `write_audit_entry()` reusable | ❌ No existe |

---

## 2. Diagnóstico

### 2.1 Decisión de formato: JSONL vs JSON array

**JSONL** (JSON Lines, una entrada por línea, append-only):

- **Pros:** append O(1), tolerante a fallos (línea corrupta no rompe el resto), grep-friendly.
- **Contras:** lectura completa requiere parsear N líneas.

**JSON array:** O(N) en append (re-escribir todo). Inaceptable.

**Decisión:** JSONL.

### 2.2 Ubicación: `%APPDATA%` vs `%LOCALAPPDATA%`

- `%APPDATA%\ClearTool\audit.jsonl` — roaming, sigue al usuario en dominios AD.
- `%LOCALAPPDATA%\ClearTool\audit.jsonl` — local.

**Decisión:** `%APPDATA%` (roaming). El log es **identidad del usuario** y debería seguirle.

### 2.3 Tamaño y rotación

Después de 6 meses de uso intenso, el archivo puede llegar a 5-20 MB. No es crítico, pero conviene:

- **Rotación por tamaño:** al alcanzar 10 MB → `audit.jsonl` → `audit.YYYYMMDD.jsonl.gz` y empezar archivo nuevo.
- **Lectura:** la pantalla muestra solo el archivo activo. Los `.gz` archivados son rescatables manualmente.

### 2.4 Concurrencia

ClearTool es single-process, pero `OpenOptions::new().append(true)` con `flock` evita corrupción si por error hay dos instancias. En Windows, NTFS no tiene `flock` nativo — usar `LockFile`/`UnlockFile` de Win32.

**Mitigación práctica:** abrir con `share_mode: 0` exclusivo, retry 3 veces con backoff.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| JSONL append-only en `%APPDATA%\ClearTool\audit.jsonl` | Tolerante, rápido, simple |
| `reverse_recipe` es objeto JSON declarativo (no código) | Versionable, ejecutable por un evaluator de Rust |
| Tipos de `reverse_recipe`: `registry`, `service`, `appx-reinstall`, `cache-restore`, `noop` | Cubre todos los módulos |
| Rotación a 10 MB con compresión gzip | Mantiene tamaño manejable |
| Lock con `LockFileEx` Win32 + retry | Concurrency safe |
| `run_id` (UUID v4) único por operación | Permite cancelar batches a medio camino |
| Cada entrada lleva `restore_point_seq` si el módulo creó uno | Permite también la reversa "atómica" |

---

## 4. Modelo de datos

### 4.1 `AuditEntry`

```rust
// models/restore.rs (ya esbozado en 01-RESTORE-POINTS)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub run_id: String,
    pub timestamp: String,            // ISO 8601
    pub module: String,               // "cache" | "debloat" | "registry" | "services"
    pub operation: String,            // "clean" | "remove" | "apply_tweak" | "stop_service"
    pub dry_run: bool,
    pub restore_point_seq: Option<u32>,
    pub items_affected: Vec<String>,  // IDs/paths/keys tocadas
    pub reverse_recipe: ReverseRecipe,
    pub status: String,               // "success" | "partial" | "failed"
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReverseRecipe {
    /// Reversa de un tweak de registro: restaurar valores previos.
    #[serde(rename = "registry")]
    Registry {
        operations: Vec<RegistryRevertOp>,
    },
    /// Reversa de un cambio de start type / state de servicio.
    #[serde(rename = "service")]
    Service {
        service_name: String,
        previous_start_type: String,
        previous_state: String,
    },
    /// Reversa de eliminar Appx: reinstall desde Store.
    #[serde(rename = "appxReinstall")]
    AppxReinstall {
        package_family_name: String,
        store_url: Option<String>,
    },
    /// No hay reversa fina; usar restore point.
    #[serde(rename = "noop")]
    Noop {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRevertOp {
    pub hive: String,
    pub key: String,
    pub name: String,
    pub kind: String,
    /// Valor antes de tocar. `None` = la entrada no existía (acción: delete).
    pub previous_value: Option<serde_json::Value>,
}
```

### 4.2 Ejemplos de entries

```jsonl
{"runId":"550e8400-e29b-41d4-a716-446655440000","timestamp":"2026-05-21T14:32:15Z","module":"registry","operation":"applyTweak","dryRun":false,"restorePointSeq":142,"itemsAffected":["taskbar-hide-widgets"],"reverseRecipe":{"kind":"registry","operations":[{"hive":"HKCU","key":"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced","name":"TaskbarDa","kind":"dword","previousValue":1}]},"status":"success","error":null}
{"runId":"a1b2c3d4-...","timestamp":"2026-05-21T14:35:02Z","module":"services","operation":"setState","dryRun":false,"restorePointSeq":143,"itemsAffected":["DiagTrack"],"reverseRecipe":{"kind":"service","serviceName":"DiagTrack","previousStartType":"Automatic","previousState":"Running"},"status":"success","error":null}
{"runId":"f9e8d7c6-...","timestamp":"2026-05-21T14:38:11Z","module":"debloat","operation":"remove","dryRun":false,"restorePointSeq":144,"itemsAffected":["copilot"],"reverseRecipe":{"kind":"appxReinstall","packageFamilyName":"Microsoft.Copilot_8wekyb3d8bbwe","storeUrl":"ms-windows-store://pdp/?productid=9NHT9RB2F4HD"},"status":"success","error":null}
```

---

## 5. Plan de UX

### 5.1 Pantalla "Log de auditoría"

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Log de auditoría                                  [Filtros ▾]  [Exportar]    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Fecha            Módulo     Operación      Items   Estado     Acción         │
│ ────────────────────────────────────────────────────────────────────────────  │
│ 2026-05-21 14:38 Debloat    remove         1       ✓ Success [Revertir]      │
│                  Microsoft.Copilot                                            │
│                  → Reinstall desde Store                                      │
│ ─────────                                                                     │
│ 2026-05-21 14:35 Services   setState       1       ✓ Success [Revertir]      │
│                  DiagTrack: Automatic → Disabled                              │
│                  → Restaurar Automatic + Running                              │
│ ─────────                                                                     │
│ 2026-05-21 14:32 Registry   applyTweak     1       ✓ Success [Revertir]      │
│                  taskbar-hide-widgets                                         │
│                  → Restaurar 3 valores de registro                            │
│ ─────────                                                                     │
│ 2026-05-21 13:01 Cache      clean          18      ⚠ Partial [Sin reversa]   │
│                  18 ubicaciones, 4.2 GB liberados, 3 errores                  │
│                  → Restore point Seq #141                                     │
│                                                                               │
│ Mostrando 4 entradas. Archivo: %APPDATA%\ClearTool\audit.jsonl (124 KB)       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Modal de revert

```
┌─────────────────────────────────────────────────────┐
│ Revertir entrada                                    │
├─────────────────────────────────────────────────────┤
│ Operación:    Apply tweak                            │
│ Tweak:        taskbar-hide-widgets                   │
│ Realizada:    2026-05-21 14:32:15                    │
│                                                     │
│ La reversa va a:                                    │
│ • Restaurar HKCU\...\TaskbarDa = 1                  │
│                                                     │
│ Esta acción se loguea como nueva entry (revert).    │
│                                                     │
│ [Cancelar]                  [Revertir →]            │
└─────────────────────────────────────────────────────┘
```

### 5.3 Estados de la entrada

| Estado UI | Reverse posible |
|---|---|
| ✓ Success | Sí (siempre que `reverseRecipe.kind != "noop"`) |
| ⚠ Partial | Sí, revierte lo aplicado |
| ✗ Failed | No (nada se aplicó realmente) |
| ↺ Reverted | No (ya está revertida; aparece como "(revert)") |

---

## 6. Plan de implementación por fases

### Fase 1 — Backend infra: archivo, lock, rotación

#### Paso 1.1 — Path utilities en `core::config`

**Archivo:** `src-tauri/src/core/config.rs`

```rust
use std::path::PathBuf;

pub fn audit_log_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(appdata).join("ClearTool");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("audit.jsonl")
}

pub fn audit_log_archive_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(appdata).join("ClearTool").join("audit-archive");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn operational_log_path() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(local).join("ClearTool").join("logs");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("app.log")
}
```

#### Paso 1.2 — Writer con append exclusivo

**Archivo:** `src-tauri/src/domain/audit.rs`

```rust
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;

use crate::core::{AppError, AppResult, config};
use crate::models::restore::{AuditEntry, ReverseRecipe};

const ROTATION_THRESHOLD: u64 = 10 * 1024 * 1024; // 10 MB
const LOCK_RETRY: u32 = 3;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(50);

pub fn write_entry(entry: &AuditEntry) -> AppResult<()> {
    rotate_if_needed()?;
    let path = config::audit_log_path();
    let line = serde_json::to_string(entry)
        .map_err(|e| AppError::Audit(format!("serialize: {}", e)))?;

    let mut last_err = None;
    for _ in 0..LOCK_RETRY {
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                writeln!(f, "{}", line).map_err(|e| AppError::Audit(format!("write: {}", e)))?;
                return Ok(());
            }
            Err(e) => {
                last_err = Some(e);
                std::thread::sleep(LOCK_RETRY_DELAY);
            }
        }
    }
    Err(AppError::Audit(format!("open after {} retries: {:?}", LOCK_RETRY, last_err)))
}

pub fn make_entry(
    module: &str,
    operation: &str,
    dry_run: bool,
    restore_point_seq: Option<u32>,
    items_affected: Vec<String>,
    reverse_recipe: ReverseRecipe,
    status: &str,
    error: Option<String>,
) -> AuditEntry {
    AuditEntry {
        run_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        module: module.to_string(),
        operation: operation.to_string(),
        dry_run,
        restore_point_seq,
        items_affected,
        reverse_recipe,
        status: status.to_string(),
        error,
    }
}

fn rotate_if_needed() -> AppResult<()> {
    let path = config::audit_log_path();
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if size < ROTATION_THRESHOLD {
        return Ok(());
    }
    let archive_name = format!("audit.{}.jsonl", Utc::now().format("%Y%m%d-%H%M%S"));
    let dest = config::audit_log_archive_dir().join(&archive_name);
    std::fs::rename(&path, &dest)
        .map_err(|e| AppError::Audit(format!("rotate: {}", e)))?;
    log::info!("rotated audit log to {:?}", dest);
    Ok(())
}
```

> Compresión gzip de archivados queda como mejora futura — no es crítica para v1.0.

### Fase 2 — Backend: list, revert, lookup

#### Paso 2.1 — `list_log`

```rust
pub fn list_log() -> AppResult<Vec<AuditEntry>> {
    let path = config::audit_log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Audit(format!("read: {}", e)))?;
    let mut entries = Vec::with_capacity(64);
    for (line_no, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<AuditEntry>(line) {
            Ok(e) => entries.push(e),
            Err(err) => {
                log::warn!("audit line {} skipped: {}", line_no, err);
            }
        }
    }
    Ok(entries)
}
```

#### Paso 2.2 — `find_entry`

```rust
pub fn find_entry(run_id: &str) -> AppResult<Option<AuditEntry>> {
    Ok(list_log()?.into_iter().find(|e| e.run_id == run_id))
}
```

#### Paso 2.3 — `revert_entry` (evaluator de `ReverseRecipe`)

```rust
pub fn revert_entry(run_id: &str) -> AppResult<()> {
    let entry = find_entry(run_id)?
        .ok_or_else(|| AppError::Audit(format!("entry {} not found", run_id)))?;

    if entry.dry_run {
        return Err(AppError::Audit("dry-run entries cannot be reverted".into()));
    }

    match &entry.reverse_recipe {
        ReverseRecipe::Registry { operations } => {
            for op in operations {
                crate::platform::registry::write_value_or_delete(
                    &op.hive, &op.key, &op.name, op.previous_value.as_ref(),
                )?;
            }
        }
        ReverseRecipe::Service { service_name, previous_start_type, previous_state } => {
            crate::platform::services::set_start_type(service_name, previous_start_type)?;
            if previous_state == "Running" {
                let _ = crate::platform::services::start(service_name);
            }
        }
        ReverseRecipe::AppxReinstall { package_family_name, store_url } => {
            // No reinstalamos automáticamente. Abrimos Store en la URL para que el usuario decida.
            if let Some(url) = store_url {
                let _ = open::that(url);
            } else {
                let url = format!("ms-windows-store://search?query={}", package_family_name);
                let _ = open::that(url);
            }
        }
        ReverseRecipe::Noop { reason } => {
            return Err(AppError::Audit(format!("entry no reversible: {}", reason)));
        }
    }

    // Loggear el revert como nueva entry
    let revert_entry = make_entry(
        &entry.module,
        &format!("revert:{}", entry.operation),
        false,
        None,
        entry.items_affected.clone(),
        ReverseRecipe::Noop { reason: format!("revert of {}", entry.run_id) },
        "success",
        None,
    );
    write_entry(&revert_entry)?;
    Ok(())
}
```

### Fase 3 — Integración con módulos destructivos

#### Paso 3.1 — Helper único de ejecución de operación con audit

**Archivo:** `src-tauri/src/domain/audit.rs` — añadir:

```rust
/// Ejecuta `op` con la red de seguridad completa:
/// 1. Crea restore point (si no dry-run).
/// 2. Ejecuta `op`, recibiendo el SequenceNumber para que la operación pueda
///    asociar su audit entry.
/// 3. Escribe el AuditEntry final.
///
/// `op` debe construir y devolver el AuditEntry final.
pub fn with_audit<F>(module: &str, operation: &str, dry_run: bool, op: F) -> AppResult<AuditEntry>
where
    F: FnOnce(Option<u32>) -> AppResult<AuditEntry>,
{
    let seq = if dry_run {
        None
    } else {
        let desc = format!("ClearTool — {} {}", module, operation);
        crate::domain::restore::ensure_or_create(&desc).ok().flatten()
    };

    let result = op(seq);

    match &result {
        Ok(entry) => {
            let _ = write_entry(entry);
        }
        Err(e) => {
            let failed = make_entry(
                module, operation, dry_run, seq,
                Vec::new(),
                ReverseRecipe::Noop { reason: "failed before applying changes".into() },
                "failed",
                Some(format!("{}", e)),
            );
            let _ = write_entry(&failed);
        }
    }

    result
}
```

#### Paso 3.2 — Cómo los módulos lo usan

```rust
// Ejemplo en domain::registry::apply
pub fn apply(input: &ApplyTweakInput) -> AppResult<()> {
    crate::domain::audit::with_audit("registry", "applyTweak", input.dry_run, |restore_seq| {
        let tweak = lookup_tweak(&input.id)?;
        let previous = capture_previous_values(&tweak)?;
        if !input.dry_run {
            execute_operations(&tweak, input.enable)?;
        }
        Ok(crate::domain::audit::make_entry(
            "registry",
            "applyTweak",
            input.dry_run,
            restore_seq,
            vec![input.id.clone()],
            ReverseRecipe::Registry { operations: previous },
            if input.dry_run { "dry-run" } else { "success" },
            None,
        ))
    }).map(|_| ())
}
```

### Fase 4 — IPC y eventos

#### Paso 4.1 — `ipc::audit`

**Archivo:** `src-tauri/src/ipc/audit.rs`

```rust
use crate::core::AppResult;
use crate::domain;
use crate::models::restore::AuditEntry;

#[tauri::command]
pub async fn list_audit_log() -> AppResult<Vec<AuditEntry>> {
    domain::audit::list_log()
}

#[tauri::command]
pub async fn revert_audit_entry(run_id: String) -> AppResult<()> {
    domain::audit::revert_entry(&run_id)
}

#[tauri::command]
pub async fn audit_log_path() -> AppResult<String> {
    Ok(crate::core::config::audit_log_path().to_string_lossy().into_owned())
}
```

#### Paso 4.2 — Emitir evento al añadir entry

Para que la UI de Audit se refresque sin polling, emitir desde `write_entry`:

```rust
// Variante con app handle:
pub fn write_entry_with_emit(entry: &AuditEntry, app: &tauri::AppHandle) -> AppResult<()> {
    write_entry(entry)?;
    let _ = app.emit("audit:new-entry", entry);
    Ok(())
}
```

> En la práctica, `with_audit` no tiene acceso al `AppHandle`. **Decisión:** mantenemos `write_entry` puro, y la UI re-fetcha con `staleTime: 2_000` tras cada mutation. Simpler.

### Fase 5 — Frontend

#### Paso 5.1 — Cliente API

**Archivo:** `src/api/client.ts` — añadir:

```ts
import type { AuditEntry } from "./types";

export const listAuditLog = () =>
  invoke<AuditEntry[]>("list_audit_log");

export const revertAuditEntry = (runId: string) =>
  invoke<void>("revert_audit_entry", { runId });

export const getAuditLogPath = () =>
  invoke<string>("audit_log_path");
```

#### Paso 5.2 — Tipos TS

**Archivo:** `src/api/types.ts`

```ts
export type ReverseRecipe =
  | { kind: "registry"; operations: RegistryRevertOp[] }
  | { kind: "service"; serviceName: string; previousStartType: string; previousState: string }
  | { kind: "appxReinstall"; packageFamilyName: string; storeUrl?: string }
  | { kind: "noop"; reason: string };

export interface RegistryRevertOp {
  hive: string;
  key: string;
  name: string;
  kind: string;
  previousValue: unknown | null;
}

export interface AuditEntry {
  runId: string;
  timestamp: string;
  module: string;
  operation: string;
  dryRun: boolean;
  restorePointSeq: number | null;
  itemsAffected: string[];
  reverseRecipe: ReverseRecipe;
  status: "success" | "partial" | "failed" | "dry-run";
  error: string | null;
}
```

#### Paso 5.3 — Pantalla `audit`

**Archivo:** `src/features/audit/audit-page.tsx` (nuevo)

```tsx
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listAuditLog, revertAuditEntry, type AuditEntry } from "../../api";
import { EmptyState } from "../../components/empty-state";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { useState } from "react";
import { ClipboardList } from "lucide-react";

export function AuditPage() {
  const qc = useQueryClient();
  const { data: entries = [], isLoading } = useQuery({
    queryKey: ["audit"],
    queryFn: listAuditLog,
    staleTime: 2_000,
  });
  const revert = useMutation({
    mutationFn: revertAuditEntry,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["audit"] }),
  });
  const [confirmId, setConfirmId] = useState<string | null>(null);

  if (isLoading) return <div className="p-6">Cargando log...</div>;
  if (entries.length === 0) {
    return (
      <EmptyState
        icon={ClipboardList}
        title="Sin operaciones registradas"
        description="Aquí aparecerán todas las operaciones destructivas con opción de revertirlas."
      />
    );
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <h2 className="text-2xl font-bold">Log de auditoría</h2>
      <p className="text-muted-foreground text-sm">
        {entries.length} entradas. Cada operación destructiva queda registrada con opción de reversa.
      </p>
      <div className="flex-1 overflow-auto border border-border rounded-lg">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-card border-b border-border z-10">
            <tr>
              <th className="text-left p-3">Fecha</th>
              <th className="text-left p-3">Módulo</th>
              <th className="text-left p-3">Operación</th>
              <th className="text-left p-3">Items</th>
              <th className="text-left p-3">Estado</th>
              <th className="text-right p-3">Acción</th>
            </tr>
          </thead>
          <tbody>
            {entries.slice().reverse().map((e) => (
              <tr key={e.runId} className="border-b border-border/40 hover:bg-card/40">
                <td className="p-3 font-mono text-xs">{new Date(e.timestamp).toLocaleString()}</td>
                <td className="p-3"><Badge variant="secondary">{e.module}</Badge></td>
                <td className="p-3">{e.operation}</td>
                <td className="p-3 text-xs">{e.itemsAffected.slice(0, 2).join(", ")}{e.itemsAffected.length > 2 ? ` +${e.itemsAffected.length - 2}` : ""}</td>
                <td className="p-3">
                  <Badge variant={e.status === "success" ? "success" : e.status === "partial" ? "warning" : "destructive"}>
                    {e.status}
                  </Badge>
                </td>
                <td className="p-3 text-right">
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={e.reverseRecipe.kind === "noop" || e.dryRun}
                    onClick={() => setConfirmId(e.runId)}
                  >
                    Revertir
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {confirmId && (
        <ConfirmRevertModal
          runId={confirmId}
          onClose={() => setConfirmId(null)}
          onConfirm={() => { revert.mutate(confirmId); setConfirmId(null); }}
        />
      )}
    </div>
  );
}

function ConfirmRevertModal(props: { runId: string; onClose: () => void; onConfirm: () => void }) {
  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={props.onClose}>
      <div className="bg-card border border-border rounded-lg p-6 max-w-md" onClick={(e) => e.stopPropagation()}>
        <h3 className="font-bold mb-2">Revertir operación</h3>
        <p className="text-sm text-muted-foreground mb-4">
          La reversa se loguea como nueva entrada y no se puede deshacer fácilmente.
        </p>
        <div className="flex gap-2 justify-end">
          <Button variant="ghost" onClick={props.onClose}>Cancelar</Button>
          <Button onClick={props.onConfirm}>Revertir</Button>
        </div>
      </div>
    </div>
  );
}
```

#### Paso 5.4 — Añadir ruta

**Archivo:** `src/router.tsx` (o equivalente) — añadir ruta `/audit` apuntando a `AuditPage`.

**Archivo:** `src/lib/routes.ts` — añadir `AUDIT: "/audit"`.

**Archivo:** `src/components/layout/app-shell.tsx` — añadir entrada de menú "Auditoría" con icono `ClipboardList`.

---

## 7. Tests

### 7.1 Unit tests

**Archivo:** `src-tauri/tests/audit.rs`

```rust
use cleartool::domain::audit;
use cleartool::models::restore::ReverseRecipe;
use tempfile::TempDir;

#[test]
fn make_entry_genera_uuid() {
    let e1 = audit::make_entry("cache", "clean", false, None, vec![], ReverseRecipe::Noop { reason: "test".into() }, "success", None);
    let e2 = audit::make_entry("cache", "clean", false, None, vec![], ReverseRecipe::Noop { reason: "test".into() }, "success", None);
    assert_ne!(e1.run_id, e2.run_id);
}

#[test]
fn write_and_list_roundtrip() {
    // Redirigir el path del audit log a un temp dir
    let tmp = TempDir::new().unwrap();
    std::env::set_var("APPDATA", tmp.path());

    let entry = audit::make_entry("cache", "clean", false, Some(42),
        vec!["loc-1".into(), "loc-2".into()],
        ReverseRecipe::Noop { reason: "irreversible".into() },
        "success", None);
    audit::write_entry(&entry).expect("write ok");
    audit::write_entry(&entry).expect("write 2");

    let read = audit::list_log().expect("list ok");
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].module, "cache");
    assert_eq!(read[0].items_affected, vec!["loc-1", "loc-2"]);
}

#[test]
fn line_corrupta_no_rompe_lectura() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("APPDATA", tmp.path());

    let valid = audit::make_entry("registry", "applyTweak", false, None, vec![], ReverseRecipe::Noop { reason: "x".into() }, "success", None);
    audit::write_entry(&valid).unwrap();

    // Inyectar línea corrupta directamente
    let path = cleartool::core::config::audit_log_path();
    let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
    use std::io::Write;
    writeln!(f, "{{ esto-no-es-json").unwrap();

    audit::write_entry(&valid).unwrap();
    let read = audit::list_log().unwrap();
    assert_eq!(read.len(), 2, "líneas válidas se leen, corrupta se ignora");
}
```

### 7.2 Test integration (mock revert)

Necesita mock de `platform::registry::write_value_or_delete`. Para v1.0, se cubre con test manual en VM:

1. Aplicar tweak `taskbar-hide-widgets`.
2. Verificar entry en audit-page.
3. Click "Revertir".
4. Verificar que `HKCU\...\TaskbarDa = 1` se restaura.
5. Verificar nueva entry `revert:applyTweak` aparece.

---

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Audit log crece sin control | Rotación a 10 MB |
| Dos instancias de ClearTool corrompen el archivo | Retry 3x con backoff en open exclusivo |
| Reverse recipe queda obsoleto si la estructura del registro cambió | El revert solo escribe valores específicos; si las claves padre fueron borradas, falla con error claro |
| Usuario revierte sin entender qué hace | Modal de confirmación + lista explícita de acciones |
| `previousValue` no se captura correctamente | Tests por módulo verifican que la entry contiene los previos antes de modificar |

---

## 9. Definition of Done

- [ ] `domain::audit::write_entry` escribe en `%APPDATA%\ClearTool\audit.jsonl`.
- [ ] `domain::audit::list_log` parsea JSONL, ignora líneas corruptas.
- [ ] `domain::audit::revert_entry` ejecuta `ReverseRecipe` y loguea nuevo entry `revert:*`.
- [ ] `domain::audit::with_audit` helper usable por los módulos siguientes.
- [ ] Rotación a 10 MB funcional (test escribiendo 11 MB de entries genera `audit-archive/audit.<timestamp>.jsonl`).
- [ ] IPC: 3 comandos (`list_audit_log`, `revert_audit_entry`, `audit_log_path`).
- [ ] Pantalla `/audit` en frontend, con tabla + modal confirmación + botón revertir.
- [ ] Entrada de menú "Auditoría" en sidebar.
- [ ] Tests `audit.rs` pasan (3 unit tests mínimo).
- [ ] Test manual VM: applyTweak + revert funciona.
- [ ] Commit `feat(audit): log JSONL + reverse recipes + UI completa`.

---

## 10. Próximo archivo

→ [03-REGISTRY.md](03-REGISTRY.md) — primer módulo destructivo que aprovecha tanto el restore point como el audit log. Sirve como **referencia de patrón** para los siguientes (services, debloat).
