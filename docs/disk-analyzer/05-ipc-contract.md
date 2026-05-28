# 05 — Contrato IPC

## Comandos Tauri

| Comando | Input | Output | Notas |
|---------|-------|--------|-------|
| `list_drives` | — | `Vec<DriveListing>` | Síncrono y rápido (< 50 ms). |
| `build_treemap_data` | `BuildTreemapInput` | `DiskAnalysisResult` | Asíncrono, larga duración. Emite eventos. |
| `cancel_disk_scan` | `{ scanId: String }` | `()` | Idempotente. |

## Eventos emitidos por backend

| Evento | Payload | Frecuencia |
|--------|---------|------------|
| `disk:progress` | `DiskScanProgressPayload` | ~5 / s |
| `disk:complete` | `{ scanId: String }` | Una vez al terminar |
| `disk:error` | `{ scanId: String; error: String }` | Una vez si falla |

```ts
interface DiskScanProgressPayload {
  scanId: string;
  bytesScanned: number;
  filesScanned: number;
  dirsScanned: number;
  currentPath: string;
  elapsedMs: number;
}
```

## Validación en el comando

```rust
#[tauri::command]
pub async fn build_treemap_data(
    app: tauri::AppHandle,
    input: BuildTreemapInput,
) -> AppResult<DiskAnalysisResult> {
    if input.root.contains("..") {
        return Err(AppError::Permission("Invalid root path".into()));
    }
    if input.scan_id.is_empty() {
        return Err(AppError::Permission("scanId required".into()));
    }
    if !Path::new(&input.root).exists() {
        return Err(AppError::Io(format!("Drive {} not found", input.root)));
    }
    domain::disk::run_analysis(app, input).await
}
```

## Mapping de errores

`AppError` ya cubre los casos típicos:

- `Io(String)` — disco no listo, ruta inválida.
- `Permission(String)` — acceso denegado al root.
- `Cancelled` — usuario canceló.

El frontend mapea `kind` a un toast amigable usando el `error-model`
skill ya existente.

## Singleton de escaneo

Sólo un escaneo activo a la vez. El estado vive en una `OnceLock<Mutex<Option<ScanCtx>>>`
en `domain::disk`. Si `build_treemap_data` se invoca con otro `scanId`
mientras hay uno corriendo:

- Política MVP: rechazar con `AppError::Permission("scan in progress")`.
- Alternativa iter-2: cancelar el anterior automáticamente.

## Wire frontend

```ts
// src/api/client.ts
export const listDrives = () =>
  invoke<DriveListing[]>("list_drives");

export const buildTreemapData = (input: BuildTreemapInput) =>
  invoke<DiskAnalysisResult>("build_treemap_data", { input });

export const cancelDiskScan = (scanId: string) =>
  invoke<void>("cancel_disk_scan", { scanId });
```

```ts
// src/api/events.ts
export const TauriEvents = {
  ...
  DiskProgress: "disk:progress",
  DiskComplete: "disk:complete",
  DiskError: "disk:error",
} as const;

export interface EventPayloads {
  ...
  "disk:progress": DiskScanProgressPayload;
  "disk:complete": { scanId: string };
  "disk:error": { scanId: string; error: string };
}
```
