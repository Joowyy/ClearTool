# Historial de sesiones — ClearTool

> Bitácora cronológica de cada sesión de trabajo con Claude. Se actualiza al
> final de cada prompt para que cualquier agente futuro pueda reconstruir el
> contexto sin tener que releer toda la conversación.
>
> **Formato:** una entrada por sesión, dentro de cada sesión una sub-entrada por
> bloque de prompts relacionados. Lo más reciente arriba.
>
> **Ubicación:** raíz del proyecto (NO dentro de `.claude/` — esa carpeta queda
> reservada para configuración leída automáticamente por Claude Code y en este
> entorno es de solo lectura para el agente).

---

## Sesión 3 — 2026-05-09 (tarde) — Bootstrap del entorno y estructura backend

### Sub-bloque 3.b — Renombrado y esqueleto de módulos backend

**Hechos**

- Carpeta renombrada de `claude/` a `.claude/` y `specs/` movida a `.claude/specs/` (lo confirmamos al fallar el `Glob` sobre la ruta vieja).
- **Efecto colateral del rename:** `.claude/` quedó como ubicación protegida; el agente solo puede leerla, no escribirla. Por eso este historial vive en el raíz, no dentro. Hay un archivo huérfano `.claude/historial-sesiones.md` (versión previa parcial) que conviene borrar manualmente.
- Renombrado del scaffold `_scaffold` → `cleartool` en:
  - `package.json` → `name: "cleartool"`.
  - `src-tauri/Cargo.toml` → `name = "cleartool"`, `lib.name = "cleartool_lib"`. También se reescribió completo: descripción, autor, `rust-version = "1.78"`, `[profile.release]` con LTO + strip + panic=abort.
  - `src-tauri/tauri.conf.json` → `productName: "ClearTool"`, ventana 1280x800 con minWidth/minHeight según spec UI, `label: "main"`.
  - `src-tauri/src/main.rs` → `cleartool_lib::run()`.
- **Estructura de módulos creada** en `src-tauri/src/` siguiendo `.claude/specs/02-backend-rust.md`:
  - `lib.rs` reescrito con `pub mod` declarations + `invoke_handler` con los 24 comandos cerrados de la spec.
  - `error.rs` con enum `AppError` + `Display` + `Serialize` artesanal (sin `thiserror` todavía) + alias `AppResult<T>`.
  - `elevation.rs` con stub `is_elevated() -> false` (la implementación Win32 vendrá cuando se añada `windows-rs`).
  - `commands/` (8 archivos): `system_info`, `explorer`, `cache`, `debloat`, `services`, `registry`, `restore`, `audit`. Todas las funciones son `pub async fn` con `#[tauri::command]` y devuelven `Err(AppError::NotImplemented)`.
  - `services/` (7 archivos): `filesystem`, `powershell`, `registry`, `service_manager`, `restore_point`, `audit_log`, `catalog`. Por ahora solo comentarios de propósito y referencia a la spec correspondiente.
  - `models/` (7 archivos): `system`, `tree`, `cache`, `debloat`, `service`, `registry`, `restore`. DTOs con `#[serde(rename_all = "camelCase")]` listos para mapear a TS.

**Decisiones tomadas durante este bloque**

- **Dependencias mínimas en `Cargo.toml` de momento.** Se mantienen solo `tauri`, `tauri-plugin-opener`, `serde`, `serde_json`. Las cajas pesadas de la spec (`thiserror`, `anyhow`, `tokio`, `windows`, `winreg`, `windows-service`, `walkdir`, `chrono`, `uuid`, `regex`, `ts-rs` y plugins extra) se incorporan cuando el módulo concreto las necesite. Justificación: primer build más rápido, superficie de auditoría menor desde el principio, evita "código zombi" enlazado a deps que aún no se usan.
- **`tauri-plugin-opener` se mantiene por ahora** (lo añadió el scaffold). Decisión final cuando empecemos a tocar UI: si ningún módulo MVP lo necesita, fuera.
- **`error.rs` sin `thiserror` todavía.** Implementación manual de `Display` + `Serialize` que respeta el contrato del frontend (`{ kind, message }`). Cuando se meta `thiserror` se reescribe sin romper la API pública.
- **`elevation.rs` con stub `false`.** Cualquier comando que dependa de elevación creerá que NO está elevado hasta que se sustituya por la versión Win32 real. No es problema porque todos los comandos están como `NotImplemented` igual.
- **Política de clippy** (a confirmar con el usuario): en CI se mantiene `cargo clippy --all-targets -- -D warnings` como gate; en desarrollo local sin `-D warnings` para no romper la iteración. Pre-commit hook con `cargo fmt + clippy -D warnings`. Razón principal: clippy detecta `unwrap()` y patrones que en una app que toca registro y servicios pueden causar pérdida de datos.

**Pendiente inmediato siguiente**

- [ ] `cd src-tauri && cargo check` para verificar que todo compila. Si falla, iterar sobre los stubs.
- [ ] `npm run tauri dev` para validar que la app sigue arrancando con el nuevo lib.rs.
- [ ] Borrar manualmente el archivo huérfano `.claude/historial-sesiones.md`.
- [ ] Decidir definitivamente la política de clippy.

---

### Sub-bloque 3.a — Bootstrap del entorno

#### Contexto de entrada

- Specs ya redactadas en sesiones anteriores (00-overview a 07-testing).
- Subagentes y skills definidos en `claude/agents/` y `claude/skills/`.
- Scaffold del proyecto: pendiente.
- `package.json`, `Cargo.toml`, `src-tauri/`, `src/`: no existían.

#### Decisiones y hechos

1. **Toolchain a instalar manualmente, en este orden:**
   1. Visual Studio Build Tools 2022/2026 con workload "Desarrollo para escritorio con C++" (incluye MSVC v143 + Windows 11 SDK + CMake).
   2. Rust vía `rustup` (toolchain `stable-x86_64-pc-windows-msvc`).
   3. Node.js LTS (24.x funcionó).
   4. WebView2 Runtime (ya viene preinstalado en Windows 11).
   5. Tauri CLI (se instala como dev-dependency local del proyecto, no global).
   6. Git.
   7. VS Code + extensiones rust-analyzer, Tauri, ESLint, Prettier, Even Better TOML.

2. **Scaffolding del proyecto:**
   - Se usó `npm create tauri-app@latest _scaffold` desde la carpeta raíz.
   - Subdirectorio temporal `_scaffold` para no chocar con `CLAUDE.md`, `claude/`, `specs/` ya existentes.
   - Contenido movido al raíz con `Move-Item _scaffold\* . -Force` y `Move-Item _scaffold\.* . -Force`.
   - Respuestas del wizard:
     - **Project name:** `_scaffold`
     - **Identifier:** `com.cleartool.app`
     - **Frontend language:** `TypeScript / JavaScript` (NO Rust — fue un error inicial elegir `Rust - (cargo)` que ofrece Sycamore/Yew/Leptos/Dioxus).
     - **Package manager:** `npm`
     - **UI template:** `React`
     - **UI flavor:** `TypeScript`

3. **Problemas resueltos durante el bootstrap:**
   - Comando `npm install -D @tauri-apps/cli@latest` se pegó duplicado y npm interpretó la versión como `latestnpm` (inexistente). Lección: revisar el pegado antes de Enter.
   - Comando `npm install` falló con `EJSONPARSE` porque existía un `package.json` de 0 bytes. Solución: borrarlo antes de relanzar el scaffold.
   - `npx tauri info` reportó `Couldn't detect any Visual Studio or VS Build Tools instance`. Causa: el instalador estaba puesto pero faltaba marcar el workload "Desarrollo para escritorio con C++". Solución: VS Installer → Modificar → marcar workload + componentes individuales (MSVC v143, SDK Windows 11 10.0.26100.7705, CMake, vcpkg). Tras reinstalar y abrir nueva PowerShell, MSVC pasó a ✔.

4. **Estado final del entorno (`npx tauri info`):**
   - ✔ WebView2 147.0.3912.98
   - ✔ MSVC: Visual Studio Build Tools 2026
   - ✔ rustc 1.95.0, cargo 1.95.0, rustup 1.29.0
   - ✔ Rust toolchain `stable-x86_64-pc-windows-msvc`
   - Node 24.13.0, npm 11.6.2
   - tauri 2, @tauri-apps/api 2.11.0, @tauri-apps/cli 2.11.1
   - tauri-plugin-opener 2 (vendrá del scaffold por defecto, evaluar si se mantiene o se quita)
   - `tauri-build`, `wry`, `tao` aparecen como "No version detected" → es **normal antes del primer build**. Se resuelven al ejecutar `npm run tauri dev` por primera vez (cargo descarga las cajas).

#### Convención de actualización del historial

- Tras cada prompt del usuario que produzca decisiones, comandos ejecutados o cambios en el repo, añadir una entrada bajo la sesión en curso.
- Si la conversación retoma el día siguiente o tras una pausa larga, abrir nueva sub-sesión con timestamp.
- Mantener este archivo y `CLAUDE.md` sincronizados: las decisiones arquitectónicas duras se reflejan en `CLAUDE.md` (sección "Decisiones arquitectónicas vivas"); los detalles operativos del día a día viven aquí.

---

## Sesiones previas (resumen reconstruido)

### Sesión 1 — 2026-05-09 (mañana) — Definición del producto

- Concepto: app Windows 11 que combina exploración de directorios, limpieza de caché y debloat agresivo.
- Stack elegido: Tauri 2.x + Rust + React/TypeScript.
- Principios no negociables fijados (ver `CLAUDE.md`).
- Modelo de subagentes y skills definido.

### Sesión 2 — 2026-05-09 (mediodía) — Diseño técnico

- Specs 00-07 redactadas y aprobadas.
- Decisiones arquitectónicas duras (backend híbrido Rust+PowerShell, postura "Total" por defecto en debloat, reversibilidad doble vía restore points + audit log, allowlists obligatorias por módulo). Detalle en `CLAUDE.md`.
- Catálogos `bloatware-catalog.json` y `cache-locations.json` creados (versiones iniciales).
- Catálogos `services-catalog.json`, `registry-tweaks.json` y los 4 schemas JSON: pendientes.
