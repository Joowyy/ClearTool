# Qwen 3.6 Plus — Fixes menores y limpiezas (M1 → M4)

> **Para Qwen 3.6 Plus.** Tareas mecánicas, de pocas líneas cada una, sin
> decisiones arquitectónicas. Hacerlas en commits separados por bloque para
> que el reviewer las repase fácil. **Antes de cada commit:** `cargo check`
> y `npx tsc --noEmit`.
>
> Estilo: comentarios en español, identificadores en inglés. No añadir
> features ni refactorizar de más — solo lo que pide cada punto.

---

## 1. Ctrl+K abre el palette dos veces (no se puede cerrar con Ctrl+K)

**Archivos:**
- [src/components/command-palette.tsx:170-179](../../src/components/command-palette.tsx#L170-L179) — listener interno con `toggle`.
- [src/hooks/use-keyboard-shortcuts.ts:22-26](../../src/hooks/use-keyboard-shortcuts.ts#L22-L26) — listener global con `openCommandPalette()` (que solo abre).

**Síntoma.** Ctrl+K para cerrar: el palette se cierra (toggle) e inmediatamente se vuelve a abrir (global). Imposible cerrar con teclado.

**Fix.** Eliminar el listener de `command-palette.tsx:170-179` entero (el `useEffect` con `document.addEventListener("keydown", down)`). Dejar que el atajo lo maneje **solo** `use-keyboard-shortcuts`. Cambiar `openCommandPalette()` en [command-palette.tsx:155-157](../../src/components/command-palette.tsx#L155-L157) a `toggleCommandPalette()`:

```ts
let globalToggle: (() => void) | null = null;

export function toggleCommandPalette() {
  globalToggle?.();
}

// dentro de CommandPalette():
useEffect(() => {
  globalToggle = () => setOpen(o => !o);
  return () => { globalToggle = null; };
}, []);
```

Y en `use-keyboard-shortcuts.ts:25` cambiar `openCommandPalette()` por `toggleCommandPalette()` (renombrar el import).

**Acceptance.** Ctrl+K abre, Ctrl+K cierra. Esc también cierra.

---

## 2. `run_script_with_timeout` ignora el parámetro `timeout`

**Archivo:** `src-tauri/src/platform/powershell.rs:76`

**Síntoma.** `cargo check` warning: `unused variable: timeout`. El timeout no se aplica — cualquier script PS lento cuelga la app indefinidamente.

**Fix.** Implementar el timeout con `tokio::time::timeout` envolviendo el `Child::wait()` (si la función es async) o con un thread de watchdog que llame a `child.kill()` tras `timeout`. Usar el patrón de `tokio`:

```rust
pub async fn run_script_with_timeout(script: &'static str, timeout: Duration) -> AppResult<Output> {
    let mut child = build_command(script).spawn()
        .map_err(|e| AppError::Powershell(format!("spawn: {}", e)))?;
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(out)) => Ok(out),
        Ok(Err(e)) => Err(AppError::Powershell(format!("wait: {}", e))),
        Err(_) => {
            let _ = child.kill().await;
            Err(AppError::Powershell(format!("timeout {}s", timeout.as_secs())))
        }
    }
}
```

Si la función actual es sync (no async), convertirla a async y actualizar los call sites (deberían ser pocos — `grep` por `run_script_with_timeout`).

**Acceptance.** El warning desaparece. Llamar con `Duration::from_millis(100)` y un script que duerma 5s devuelve `AppError::Powershell("timeout 0s")` en ~100ms.

---

## 3. Audit log de network filtra el script PowerShell crudo

**Archivo:** [src-tauri/src/ipc/network.rs:36](../../src-tauri/src/ipc/network.rs#L36)

**Síntoma.** Las entries quedan con `items_affected: ["ipconfig /release; ipconfig /renew"]`. Inútil para auditoría y un dolor para mostrar en la UI.

**Fix.** Cambiar la firma de `run_network_op` para que reciba `items_affected: Vec<String>` y pasarle algo semántico desde cada comando:

```rust
fn run_network_op(script: &str, action: &str, items_affected: Vec<String>, dry_run: bool) -> AppResult<()> {
    // ...
    let entry = AuditEntry {
        // ...
        items_affected,  // en vez de vec![script.to_string()]
        // ...
    };
}

// y en cada caller:
pub async fn flush_dns(dry_run: bool) -> AppResult<()> {
    run_network_op("ipconfig /flushdns", "flush_dns", vec!["DNS resolver cache".into()], dry_run)
}
```

**Acceptance.** Tras flush_dns + abrir Audit log: aparece "DNS resolver cache" en items_affected, no el script.

---

## 4. `restore_hosts_file` sobrescribe sin backup

**Archivo:** [src-tauri/src/ipc/network.rs:74-125](../../src-tauri/src/ipc/network.rs#L74-L125)

**Síntoma.** `Set-Content` sin backup. Si el usuario tenía entradas custom, se pierden.

**Fix.** Antes del `Set-Content`, copiar el hosts actual a `%TEMP%\cleartool-hosts-backup-<timestamp>.txt`. Incluir la ruta del backup en el `AuditEntry.items_affected` y en un `reverse_recipe` nuevo tipo `RestoreFromBackup { backup_path: PathBuf, target_path: PathBuf }` (añadir variante al enum `ReverseRecipe` en `models/restore.rs`).

Si la variante `RestoreFromBackup` es demasiado para Qwen, hacer un compromiso: dejar `reverse_recipe: ReverseRecipe::Noop { reason: format!("hosts backup: {}", backup_path) }` y que la reversa manual la haga el usuario copiando el archivo a mano. Documentar en el toast: "Backup guardado en %TEMP%\cleartool-hosts-backup-...".

**Acceptance.** Modificar `C:\Windows\System32\drivers\etc\hosts` añadiendo una línea custom, ejecutar `restore_hosts_file(false)`, verificar que existe el backup en `%TEMP%\cleartool-hosts-backup-*.txt` con la línea custom.

---

## 5. `enable_startup` busca la entry dos veces

**Archivo:** [src-tauri/src/domain/startup.rs:74-99](../../src-tauri/src/domain/startup.rs#L74-L99)

**Síntoma.** Hace `entries.iter().find(...).ok_or(...)?` y luego dentro del match hace `entries.iter().find(...).unwrap().origin` — duplicado y con `.unwrap()` innecesario.

**Fix.** Capturar el origin en la primera búsqueda:

```rust
pub fn enable_startup(id: &str) -> AppResult<()> {
    let entries = list_all()?;
    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::Validation(format!("startup entry no encontrada: {}", id)))?;

    match &entry.origin {
        StartupOrigin::Registry { hive, key, name } => enable_registry_entry(hive, key, name)?,
        StartupOrigin::StartupFolder { lnk_path } => enable_lnk_entry(std::path::Path::new(lnk_path))?,
        StartupOrigin::ScheduledTask { task_path } => enable_scheduled_task(task_path)?,
        StartupOrigin::Service { service_name } => crate::platform::services::set_start_type(service_name, "Automatic")?,
        StartupOrigin::UwpAutoStart { .. } => return Err(AppError::Validation("UWP autostart enable aún no soportado".into())),
    }
    // ...
}
```

**Acceptance.** El `.unwrap()` desaparece. La función queda idéntica en comportamiento.

---

## 6. Warnings de `cargo check`

Tres warnings actuales que ensucian el output. Limpiar.

**A.** [src-tauri/src/platform/processes.rs:82-83](../../src-tauri/src/platform/processes.rs#L82-L83) — imports `CreateToolhelp32Snapshot`, `Process32FirstW`, `Process32NextW`, `PROCESSENTRY32W`, `TH32CS_SNAPPROCESS` no usados. Eliminar líneas 82-83 del `use windows::Win32::System::Diagnostics::ToolHelp::{...}` (esos símbolos sí se usan en `get_thread_count_via_snapshot` y `find_children`, pero los imports superiores duplican). Verificar con `cargo check` que sigue compilando.

**B.** `src-tauri/src/domain/registry.rs:118` — `let mut skipped = 0;` no se muta. Quitar `mut`.

**C.** El `unused variable: timeout` queda resuelto con la tarea **#2** de este archivo.

**Acceptance.** `cd src-tauri && cargo check 2>&1 | grep warning` no devuelve nada.

---

## 7. Variante `NotImplemented` del error usa argumentos que no acepta

**Archivo:** [src-tauri/src/domain/startup.rs:178, 183](../../src-tauri/src/domain/startup.rs#L178)

**Síntoma.** Los stubs `#[cfg(not(windows))]` hacen `AppError::NotImplemented("Windows only".into())`. Pero el enum en [core/error.rs:39-40](../../src-tauri/src/core/error.rs#L39-L40) define `NotImplemented` sin argumentos. En Windows pasa porque ese código no se compila; en Linux no compilaría.

**Fix.** Cambiar las dos líneas a `AppError::Validation("Windows only".into())` (validation acepta `String`). O alternativamente convertir el enum a `NotImplemented(String)` y actualizar **todos** los call sites + el `match` de `Serialize`. La primera opción es 2 líneas; la segunda son ~15. **Hacer la primera.**

**Acceptance.** Si `cargo check --target x86_64-unknown-linux-gnu` está disponible, pasa. Si no, basta con que `cargo check` siga limpio en Windows.

---

## 8. Comentario muerto en `domain/cache.rs`

**Archivo:** [src-tauri/src/domain/cache.rs:944-946](../../src-tauri/src/domain/cache.rs#L944-L946)

```rust
// ── VerifyReport (añadido a models/cache.rs) ──
// Se define aquí temporalmente hasta que se mueva al modelo.
```

`VerifyReport` ya está en `models/cache.rs:243`. Borrar las 3 líneas. No queda nada después de ellas.

**Acceptance.** El archivo termina en línea 943.

---

## 9. `glob_match` es un mock (solo entiende `*` y `*.ext`)

**Archivo:** [src-tauri/src/domain/cache.rs:301-310](../../src-tauri/src/domain/cache.rs#L301-L310)

**Síntoma.** Cualquier filtro `include`/`exclude` con patrones como `**/*.tmp`, `cache_*`, `foo?` se ignora silenciosamente.

**Fix.** Reemplazar la función entera por el crate `glob` (ya está en el ecosistema Rust, ligero). Añadir a `Cargo.toml`:

```toml
glob = "0.3"
```

Y sustituir:

```rust
fn glob_match(pattern: &str, text: &str) -> bool {
    match glob::Pattern::new(pattern) {
        Ok(p) => p.matches(text),
        Err(_) => text == pattern,  // fallback: comparación literal
    }
}
```

Si el coste de la nueva dep importa, alternativa es escribir un parser glob mínimo
(no recomendado — Qwen, usa la dep).

**Acceptance.** Añadir un test unit:
```rust
#[test] fn glob_matches_double_star() {
    assert!(glob_match("**/*.tmp", "foo/bar/baz.tmp"));
    assert!(!glob_match("**/*.tmp", "foo/bar/baz.log"));
}
```

---

## 10. Versión hardcoded "0.5.0" en el About

**Archivo:** [src/features/settings/settings-page.tsx:358](../../src/features/settings/settings-page.tsx#L358)

**Síntoma.** Hardcoded `<Badge>0.5.0</Badge>`. Si bump de versión, queda desincronizado.

**Fix.** Crear un comando IPC nuevo `app_version() -> AppResult<AppVersionInfo>`:

```rust
// src-tauri/src/ipc/system_info.rs
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppVersionInfo {
    pub version: String,       // env!("CARGO_PKG_VERSION")
    pub build_date: String,    // built en compile-time via build.rs
    pub git_commit: Option<String>,
}

#[tauri::command]
pub async fn app_version() -> AppResult<AppVersionInfo> {
    Ok(AppVersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: env!("BUILD_DATE").to_string(),  // ver más abajo
        git_commit: option_env!("GIT_COMMIT").map(String::from),
    })
}
```

`build.rs` (en `src-tauri/`):
```rust
fn main() {
    println!("cargo:rustc-env=BUILD_DATE={}", chrono::Utc::now().format("%Y-%m-%d"));
    let git = std::process::Command::new("git").args(["rev-parse", "--short", "HEAD"]).output();
    if let Ok(out) = git {
        if out.status.success() {
            println!("cargo:rustc-env=GIT_COMMIT={}", String::from_utf8_lossy(&out.stdout).trim());
        }
    }
    tauri_build::build()
}
```

Registrar el comando en `lib.rs::invoke_handler`. Exponer `appVersion()` en `client.ts`.

Settings page: `useQuery(["app-version"], appVersion)` y sustituir el badge hardcoded.

**Acceptance.** El About muestra siempre la versión real de `Cargo.toml`, la fecha de build y (si disponible) el commit corto.
