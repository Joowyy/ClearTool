# Historial de sesiones — ClearTool

> Bitácora cronológica de cada sesión de trabajo con Claude. Se actualiza al
> final de cada prompt para que cualquier agente futuro pueda reconstruir el
> contexto sin tener que releer toda la conversación.
>
> **Formato:** una entrada por sesión, dentro de cada sesión una sub-entrada por
> bloque de prompts relacionados. Lo más reciente arriba.

---

## Sesión 3 — 2026-05-09 (tarde) — Bootstrap del entorno de desarrollo

### Contexto de entrada

- Specs ya redactadas en sesiones anteriores (00-overview a 07-testing).
- Subagentes y skills definidos en `claude/agents/` y `claude/skills/`.
- Scaffold del proyecto: pendiente.
- `package.json`, `Cargo.toml`, `src-tauri/`, `src/`: no existían.

### Decisiones y hechos

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

### Pendientes inmediatos

- [ ] **Renombrar `claude/` a `.claude/`** y mover `specs/` dentro como `.claude/specs/`. La spec lo exige y los subagentes/skills están escritos asumiendo esa ruta.
  ```powershell
  Rename-Item claude .claude
  Move-Item specs .claude\specs
  ```
- [ ] **Primer `npm run tauri dev`** para validar que la app de scaffold compila y abre. La primera vez tarda 3-10 min descargando cajas Rust (`tauri-build`, `wry`, `tao`, `webview2-com`, `windows-rs`, etc.).
- [ ] Decidir si se conserva `tauri-plugin-opener` (lo añade el scaffold por defecto). Para ClearTool no lo usaremos en MVP, candidato a quitar.
- [ ] Empezar a poblar los catálogos pendientes (`services-catalog.json`, `registry-tweaks.json`) y sus schemas JSON.
- [ ] Crear la primera estructura de módulos en `src-tauri/src/` siguiendo `02-backend-rust.md` (módulos `commands/`, `core/`, `audit/`, `errors.rs`).

### Convención de actualización del historial

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
