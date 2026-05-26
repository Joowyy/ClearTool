# 01 — Arquitectura

## Visión de alto nivel

```
+-----------------------------------------------+
|              UI: React + Tailwind             |
|  páginas: explorer, cache, debloat, services, |
|  registry, restore, settings                  |
+-----------------------------------------------+
                    |  invoke()  |  listen()
                    v            ^
+-----------------------------------------------+
|     Tauri runtime (Rust 1.78+, tokio)         |
|  commands -> services -> [Win32 / PS / FS]    |
+-----------------------------------------------+
       |             |               |
       v             v               v
   windows-rs      winreg        powershell
   (Win32, WMI,   (registro)    (Appx, Edge,
    Services)                   provisioned)
```

## Capas

### 1. UI (`src/`)

React + TypeScript estricto. Sin lógica de sistema. Solo:

- Render de DTOs recibidos del backend.
- Diálogos de confirmación destructiva.
- Streaming de progresos vía eventos Tauri.
- Persistencia de preferencias UI (tema, último directorio explorado) en `localStorage` Tauri.

### 2. Comandos (`src-tauri/src/commands/`)

Capa fina. Cada función `#[tauri::command]`:

1. Valida inputs.
2. Pide restore point si destructivo.
3. Llama al servicio.
4. Loguea evento.
5. Devuelve `Result<TOut, AppError>`.

### 3. Servicios (`src-tauri/src/services/`)

Lógica de dominio. No conoce Tauri. Recibe datos validados, devuelve resultados o errores. Aquí vive:

- `filesystem` (escaneo y borrado seguros).
- `powershell` (invocación tipada).
- `registry` (lectura/escritura con whitelist).
- `restore_point` (WMI o PS-wrapper).
- `audit_log` (JSONL append-only).

### 4. Modelos (`src-tauri/src/models/`)

DTOs serde + ts-rs. Un único lugar donde se definen los tipos compartidos con TS.

### 5. Errores (`src-tauri/src/error.rs`)

`AppError` enum con `thiserror`. Variantes:

- `Io(#[from] std::io::Error)`
- `Registry(String)`
- `Powershell(String)`
- `Permission(String)` — input rechazado o capability faltante.
- `NotElevated` — operación requería admin y la app no fue elevada.
- `Cancelled` — usuario abortó.
- `RestoreUnavailable(String)` — System Restore no funcional.
- `External(String)` — proceso externo devolvió no-cero.

`Serialize` para que llegue tipado al frontend.

## Decisiones clave

### D1. Tauri 2 sobre Electron / WPF

- **Pros Tauri:** binario 5–10 MB, Rust seguro, IPC tipado, capabilities granulares.
- **Pros WPF:** integración Microsoft profunda.
- **Pros Electron:** ecosistema NPM gigante.
- **Decisión:** Tauri. La lógica peligrosa la queremos en Rust con borrow checker, no en C# con runtime ni en Node con todo en memoria.

### D2. PowerShell para Appx en lugar de COM directo

- Manipular Appx Manager directo via COM/WinRT desde Rust es viable pero verboso y frágil entre builds de Windows.
- Invocar PowerShell con args validados y captura JSON estructurado es más mantenible.
- Trade-off: dependencia de `powershell.exe` (presente por defecto en Windows 11).

### D3. windows-rs vs winapi

- `windows-rs` (oficial Microsoft) está bien mantenido y cubre toda la surface moderna.
- `winapi` queda como alternativa solo donde windows-rs no llegue.

### D4. Restore points obligatorios + tweak de frecuencia

- Microsoft limita System Restore a 1 punto cada 24h.
- ClearTool aplica el tweak `SystemRestorePointCreationFrequency=0` (con consentimiento la primera vez) para que cada sesión destructiva tenga su propio punto.

### D5. ts-rs para bindings

- Genera `.d.ts` desde structs Rust con `#[ts(export)]` y `#[derive(TS)]`.
- Garantiza un único schema entre backend y frontend.
- Alternativa rechazada: tRPC (sería duplicar runtime).

### D6. Capabilities por comando

- Cada comando declara qué `permissions` Tauri necesita.
- `default.json` cubre lecturas; `elevated.json` cubre lo destructivo.
- Reduce blast radius si un comando se compromete.

## Concurrency model

- **Lecturas:** paralelas, sin restricciones.
- **Escrituras (registro/servicios/FS destructivo):** serializadas vía `tokio::sync::Mutex` global + lock por target (no dos comandos tocando el mismo servicio a la vez).
- **Operaciones largas:** `spawn_blocking` para trabajo CPU/IO, eventos para progreso.

## Diagrama de flujo destructivo

```
User clic "Limpiar Windows Update cache"
   |
   v
UI -> ConfirmDestructive
   |  (muestra paths, tamaño, riesgos, dry-run, restore-point id placeholder)
   v
User confirma
   |
   v
invoke('clean_cache_locations', { ids, dryRun: false })
   |
   v
Comando Rust:
  1. Validar ids contra catálogo.
  2. Crear restore point.
  3. Stop wuauserv, bits.
  4. Para cada path: scan -> delete con filtros.
  5. Start wuauserv, bits.
  6. Loguear evento.
  7. Return CleanReport.
   |
   v
UI muestra reporte + opción de "Deshacer" (restore al punto creado).
```

## Empaquetado

```bash
cargo tauri build
# Genera src-tauri/target/release/bundle/msi/cleartool_<ver>_x64_en-US.msi
# Y src-tauri/target/release/bundle/nsis/cleartool_<ver>_x64-setup.exe
```

Manifest UAC con `requireAdministrator` activado en `tauri.conf.json` -> `bundle.windows.signCommand` y `bundle.windows.tsp` para futura firma. Firma de código fuera de scope inicial.

## Hot path performance

- Escaneo de C:\ a profundidad 3 < 5s en SSD.
- Catálogo de bloatware completo (~80 entradas) detectados en < 3s.
- Crear restore point: 5–30s (depende de Microsoft, no controlable).
- IPC overhead Tauri: < 1 ms por invoke promedio.
