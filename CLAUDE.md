# CLAUDE.md — Memoria del proyecto ClearTool

Este archivo es leído automáticamente por Claude Code al abrir el proyecto. Contiene contexto de alto nivel para que cualquier agente trabaje con coherencia.

## Qué es ClearTool

Aplicación de escritorio Windows 11 que combina exploración de directorios, limpieza profunda de caché del SO y debloat agresivo (incluido el ecosistema Microsoft). Stack: **Tauri 2.x + Rust + React/TypeScript**.

## Principios no negociables

1. **Seguridad antes que velocidad.** Toda operación destructiva crea un restore point Windows previo y queda registrada en un log JSON con instrucciones de reversa.
2. **Transparencia total.** Antes de ejecutar, la UI muestra exactamente qué archivos se borrarán, qué claves de registro se modificarán, qué servicios se detendrán. Sin "magia".
3. **Idempotencia.** Ejecutar la misma acción dos veces no debe romper nada ni dejar el sistema en estado inconsistente.
4. **Privilegios mínimos por operación.** Solo escala a admin cuando la operación lo necesita; las lecturas/escaneos van como usuario normal cuando es posible.
5. **Dry-run obligatorio.** Cada módulo destructivo expone un modo `--dry-run` que reporta qué haría sin tocar nada.

## Convenciones

- **Idioma:** comentarios y docs en español, identificadores de código en inglés (estándar Rust/JS).
- **Commits:** Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`).
- **Errores Rust:** `thiserror` para tipos custom, `anyhow` solo en el binario, nunca en libs.
- **Errores TS:** discriminated unions `Result<T, E>` exportadas desde Tauri commands.
- **Estilo Rust:** `cargo fmt` + `cargo clippy -- -D warnings`.
- **Estilo TS:** `eslint` + `prettier` con config estricta, no `any` salvo en límites de FFI.

## Mapa de subagentes

Cuando el usuario pida algo concreto, delega al subagente correcto:

- **windows-systems-expert** — Win32 API, registro, WMI, servicios, COM.
- **tauri-rust-backend** — comandos Tauri, IPC, plugins, scaffolding Rust.
- **react-frontend** — UI, componentes, estado, comunicación con backend.
- **debloat-specialist** — Appx packages, capabilities, scripts PS, listas de bloatware.
- **security-auditor** — revisa cualquier cambio que toque registro, servicios o borrado masivo.
- **qa-tester** — diseña casos de prueba, ejecuta en VM Windows 11, valida reversibilidad.

## Mapa de skills

Skills reutilizables en `.claude/skills/`:

- `windows-registry-ops` — patrones seguros para leer/escribir HKLM/HKCU.
- `powershell-debloat` — invocación segura de PS desde Rust con captura estructurada.
- `restore-point-manager` — crear/listar/restaurar System Restore Points vía WMI.
- `cache-scanner` — catálogo de ubicaciones de caché conocidas en Windows 11.
- `tauri-command-builder` — plantilla para crear nuevos comandos Tauri con validación.
- `directory-tree-explorer` — recorrido eficiente del sistema de archivos con tamaños.

## Decisiones arquitectónicas vivas

- **2026-05-09:** Stack elegido = Tauri + Rust + React. Justificación: binarios pequeños, acceso nativo Win32, IPC tipado.
- **2026-05-09:** Privilegios: la app pide elevación al iniciar (manifest), pero módulos de solo-lectura quedan habilitados aunque el usuario cancele UAC (modo "limitado").
- **2026-05-09:** Punto de restauración antes de cualquier operación destructiva = obligatorio, no configurable.
- **2026-05-09 (sesión 2):** Backend **híbrido Rust + PowerShell**. Rust para FS, registro, servicios, restore points, COM, audit log. PowerShell **embebido** vía `include_str!` para Appx (paquetes y provisionados). Sin construcción dinámica de PS desde input — los args pasan por validadores regex en Rust antes de invocar. Detallado en `.claude/specs/04-modules/debloat-engine.md` y `.claude/specs/05-security.md`.
- **2026-05-09 (sesión 2):** Postura por defecto del módulo Debloat = **Total** (agresiva). La UI marca todo el catálogo al entrar; los presets `Mínimo` y `Recomendado` son alternativas que el usuario elige. Disclaimers obligatorios para Edge, Store, OneDrive, Cortana. Detalle en `.claude/specs/04-modules/debloat-engine.md` y flujo en `.claude/specs/06-ui-ux.md`.
- **2026-05-09 (sesión 2):** Reversibilidad doble — cada operación destructiva crea **un restore point del sistema** (vía `SRSetRestorePointW`) **y** escribe un **`AuditEntry` con `reverse_recipe`** en `%APPDATA%\ClearTool\audit.jsonl`. La reversa fina (un solo tweak, un solo servicio) usa el audit log; la reversa total usa el restore point. Detallado en `.claude/specs/04-modules/restore-point-system.md`.
- **2026-05-09 (sesión 2):** Allowlists obligatorias por módulo destructivo: `cache-locations` (paths), `bloatware-catalog` (Appx + uninstaller IDs), `services-catalog` (services), `registry-tweaks` (hives + path prefixes). Cualquier intento fuera de allowlist → `AppError::Permission` y abort. Validación al cargar el catálogo y en cada comando.
- **2026-05-09 (sesión 3):** Scaffold inicial generado con `npm create tauri-app@latest` (React + TypeScript + Vite + npm). Identifier `com.cleartool.app`. CLI Tauri instalada como dev-dependency local del proyecto, no global. El plugin `tauri-plugin-opener` lo añade el scaffold por defecto — pendiente decidir si se conserva.
- **2026-05-09 (sesión 3):** **Bitácora operativa en `HISTORIAL-SESIONES.md`** (raíz del proyecto, cronológica, una entrada por sesión). Las decisiones arquitectónicas duras siguen viniendo aquí, en esta sección de `CLAUDE.md`; el día a día (comandos ejecutados, errores resueltos, pasos pendientes) vive en el historial. Convención: cualquier prompt que produzca decisiones, comandos o cambios en el repo añade entrada al historial al final del turno. (Nota: vive en raíz porque `.claude/` es de solo lectura para el agente en este entorno.)
- **2026-05-09 (sesión 3):** Esqueleto de módulos backend creado en `src-tauri/src/` con stubs `NotImplemented`. Dependencias del `Cargo.toml` mantenidas al mínimo (solo `tauri`, `tauri-plugin-opener`, `serde`, `serde_json`); las cajas pesadas (`thiserror`, `windows-rs`, `winreg`, `tokio`, etc.) se incorporan cuando cada módulo las necesite, no antes. Justificación: build inicial rápido + superficie de auditoría mínima desde el día uno.

## Estado del diseño (snapshot)

| Spec / artefacto | Estado |
|---|---|
| `.claude/specs/00-overview.md` | ✅ |
| `.claude/specs/01-architecture.md` | ✅ |
| `.claude/specs/02-backend-rust.md` | ✅ |
| `.claude/specs/03-frontend-react.md` | ✅ |
| `.claude/specs/04-modules/directory-explorer.md` | ✅ |
| `.claude/specs/04-modules/cache-cleaner.md` | ✅ |
| `.claude/specs/04-modules/debloat-engine.md` | ✅ |
| `.claude/specs/04-modules/service-manager.md` | ✅ |
| `.claude/specs/04-modules/registry-tweaks.md` | ✅ |
| `.claude/specs/04-modules/restore-point-system.md` | ✅ |
| `.claude/specs/05-security.md` | ✅ |
| `.claude/specs/06-ui-ux.md` | ✅ |
| `.claude/specs/07-testing.md` | ✅ |
| Catálogo `bloatware-catalog.json` | ✅ inicial |
| Catálogo `cache-locations.json` | ✅ inicial |
| Catálogo `services-catalog.json` | ❌ pendiente |
| Catálogo `registry-tweaks.json` | ❌ pendiente |
| Schemas JSON (`*.schema.json`) para los 4 catálogos | ❌ pendientes |
| Scaffold Tauri+Rust+React inicial (`src-tauri/`, `src/`, `package.json`, `Cargo.toml`) | ✅ generado (sesión 3) |
| Validación `npm run tauri dev` (primer build, descarga cajas Rust) | ❌ pendiente |
| Renombrar `claude/` → `.claude/` y mover `specs/` a `.claude/specs/` | ❌ pendiente |

## Lo que NO está decidido

- Sistema de actualizaciones (probablemente Tauri Updater con servidor propio).
- Firma de código (requiere certificado, postergado).
- Telemetría propia (probablemente nula por coherencia con la propuesta del producto).

## Cómo empezar a trabajar

1. Lee `.claude/specs/00-overview.md` para el panorama.

> Estructura: todo el material que usa Claude Code para guiarse vive bajo `.claude/`:
> agents en `.claude/agents/`, skills en `.claude/skills/`, specs en `.claude/specs/`.
2. Lee la spec del módulo concreto que vas a tocar.
3. Identifica qué subagente debe liderar y qué skills aplican.
4. Escribe tests antes que código en módulos destructivos.

## Recursos externos clave

- Win32 docs: https://learn.microsoft.com/en-us/windows/win32/
- windows-rs: https://github.com/microsoft/windows-rs
- Tauri 2 docs: https://v2.tauri.app/
- Appx cmdlets: https://learn.microsoft.com/en-us/powershell/module/appx/
