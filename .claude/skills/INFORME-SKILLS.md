# Informe — Toolkit de skills para acelerar ClearTool

**Fecha:** 2026-05-27
**Alcance:** auditoría del stack y del directorio `.claude/` + `docs/`, búsqueda en el marketplace oficial, investigación de best-practices web, y propuesta + creación de skills nuevas que cubren todo el roadmap (M1→M5).

---

## 1. El stack, en una línea cada pieza

ClearTool es una app de escritorio **solo-Windows 11** construida sobre tres capas que conviene entender por separado, porque cada una impone sus propias reglas y cada una tiene sus huecos de tooling.

**Tauri 2.x** es el shell de la aplicación: empaqueta un frontend web dentro de una ventana nativa usando el WebView2 del sistema (no embebe Chromium como Electron, de ahí binarios pequeños). Tauri aporta el puente IPC tipado entre Rust y el frontend (`#[tauri::command]` ↔ `invoke`), el sistema de *capabilities* (permisos declarativos por ventana), el bundler de instaladores (NSIS/MSI), el plugin de auto-update y el de logging. Es la pieza que más decisiones de seguridad y distribución concentra.

**Rust** es el backend: todo lo que toca el sistema operativo de verdad — sistema de archivos, registro, servicios, procesos, COM/WMI para restore points — vive aquí. El proyecto ya fijó las convenciones correctas: `thiserror` para errores tipados en librerías, `anyhow` solo en el binario, `cargo fmt` + `cargo clippy -D warnings`, y un backend **híbrido** que llama a PowerShell embebido (vía `include_str!`) para Appx, con validación regex de todos los argumentos antes de invocar. Para Win32 usa `windows-rs`; para procesos `sysinfo`; para bindings TS `ts-rs`.

**React + TypeScript (Vite)** es la UI: TanStack Query para estado de servidor, Zustand para estado de UI, shadcn/ui + Tailwind para componentes, y `sonner` para toasts. La convención dura es `Result<T,E>` como uniones discriminadas exportadas desde los comandos Tauri, nada de `any` salvo en límites de FFI.

La tensión central del producto —y la razón de que el tooling importe tanto— es que **cada operación destructiva debe ser transparente, reversible (restore point + audit log con `reverse_recipe`), idempotente y con dry-run obligatorio**. Eso no se puede improvisar por módulo: necesita patrones codificados. De ahí las skills.

---

## 2. Auditoría del `.claude/` actual

### 2.1 Skills propias del proyecto (6) — sólidas y bien escritas

| Skill | Cubre | Estado |
|---|---|---|
| `tauri-command-builder` | Plantilla de comando Tauri: input/output con `ts-rs`, `Result<_, AppError>`, dry-run, restore point, audit log, capability, wrapper TS, eventos de progreso, test. | Excelente |
| `windows-registry-ops` | Allowlist de ramas, validación anti path-traversal, `ReversibleRegistryEdit`, backup `.reg`, tipado de valores. | Excelente |
| `powershell-debloat` | Invocación segura de PS desde Rust, captura JSON, validación de args, scripts canónicos Appx/Provisioned. | Excelente |
| `cache-scanner` | Catálogo de ubicaciones de caché Win11, escaneo seguro, validación de safe-roots, orden de borrado. | Muy bueno (no cubre aún el rewrite v2: pending-rename / restart-manager). |
| `restore-point-manager` | Crear/listar/restaurar restore points vía WMI/PS, throttling de 24h, audit log JSONL. | Muy bueno |
| `directory-tree-explorer` | Walk con streaming por lotes, reparse points, paths largos, atributos, virtualización. | Muy bueno |

Estas seis están a buen nivel y siguen un estilo consistente (frontmatter YAML con `description` específica, prosa en español, identificadores en inglés, foco en seguridad/idempotencia). No hace falta reescribirlas.

### 2.2 Skills genéricas importadas — ruido para este proyecto

El árbol `.claude/skills/disenio-frontend/**`, `interface-design/`, `r3f-best-practices/` y `three-best-practices/` son skills de **diseño visual y 3D genéricas** (branding, slides, logos, React-Three-Fiber). Salvo que ClearTool mantenga la visualización 3D del árbol de directorios (existe un fix histórico `02-tree-recursive-3d.md`), la mayoría **no codifican ninguna convención del proyecto** y diluyen el contexto que Claude carga. Recomendación: mover lo que no se use a un repo aparte o marcarlas como no prioritarias; conservar a lo sumo `interface-design` y `ui-styling` (shadcn/Tailwind) si ayudan al refactor de UI de M4.

### 2.3 Agentes (6) — bien mapeados, sin skill espejo en varios casos

`windows-systems-expert`, `tauri-rust-backend`, `react-frontend`, `debloat-specialist`, `security-auditor`, `qa-tester`. El mapa de subagentes es bueno, pero **algunos agentes no tienen una skill que codifique sus patrones**: `react-frontend` (no había skill de convenciones de frontend), `qa-tester` (no había skill de testing/CI), `windows-systems-expert` para procesos/Restart Manager (no existía), y no había nada para distribución/firma (territorio de M5).

### 2.4 Huecos detectados frente al roadmap

| Milestone | Necesidad técnica | ¿Cubierta por skill? |
|---|---|---|
| **M1** Error model | AppError Rust ↔ normalizeError/toast/ErrorBoundary | ❌ No (era el bug que "mata la UX") |
| **M2** Process Manager | sysinfo+Win32, Restart Manager `who_locks_path`, kill/suspend/close | ❌ No |
| **M2** Cache rewrite | MoveFileEx/PendingFileRename, retry, CleanPlan | 🟡 Parcial (`cache-scanner` no llega al v2) |
| **M3** Catálogo/Startup/Privacy | schema v2, registry tweaks | 🟡 Parcial (`registry-ops` + `powershell-debloat`) |
| **M4** UI/UX refactor | convenciones React, themes, i18n, virtualización, command palette | ❌ No (las skills de diseño son genéricas) |
| **M5** Distribución | NSIS, Azure Trusted Signing, updater keys, winget, CI release | ❌ No (área más débil) |
| **Transversal** Testing/CI | tests destructivos, GitHub Actions | ❌ No |

---

## 3. Marketplace oficial — qué hay y qué no

Busqué en el marketplace de plugins/skills oficial con términos de Tauri, Rust, React, Windows, code-signing, CI y testing.

**Conclusión: no existe ningún plugin o skill dedicado a Tauri, Rust de sistemas, o desarrollo de apps de escritorio Windows.** Los resultados con "react"/"electron"/"desktop"/"signing" pertenecen a SDKs ajenos (Zoom, CockroachDB) y son ruido.

Lo que **sí** es aprovechable, como utilidades de proceso de ingeniería de propósito general:

- **Plugin `engineering`** — incluye skills `testing-strategy`, `code-review`, `architecture`, `deploy-checklist`, `debug`, `tech-debt`, `system-design`, `documentation`, `incident-response`. Útil como complemento de proceso, no de dominio.
- **Plugin `github`** — flujo de PRs/issues si se gestiona el proyecto en GitHub (encaja con M5: Releases, winget PR).
- **Skills ya disponibles en este entorno**: `security-review` y `review` (revisión de cambios/PR) — directamente aplicables a la regla del proyecto de que `security-auditor` revisa todo cambio a registro/servicios/borrado. `skill-creator` para mantener/optimizar las skills propias.

Recomendación de instalación: el plugin **`engineering`** (por `testing-strategy` y `deploy-checklist`) y **`github`**; y usar `security-review` en cada PR que toque módulos destructivos. Todo lo de **dominio** (Tauri/Win32/distribución) hay que crearlo nosotros — que es justo lo que hicimos.

---

## 4. Best-practices web que fijan el contenido de las skills

Dos puntos se verificaron contra documentación actual (2026) porque eran los de mayor riesgo:

**Auto-update de Tauri 2 y firma.** Hay **dos firmas distintas y ambas obligatorias**: (1) Authenticode/code-signing para que SmartScreen no bloquee — el proyecto eligió Azure Trusted Signing, integrable en CI con secrets; y (2) la firma del **updater** con claves propias `tauri signer` (Ed25519), que no se puede desactivar y cuya clave privada, si se pierde, deja a los usuarios instalados sin poder actualizar. La pública va en `tauri.conf.json`; la privada solo como secret de CI (`TAURI_SIGNING_PRIVATE_KEY[_PASSWORD]`). Confundirlas es el error clásico — la skill `tauri-release-signing` lo deja explícito.

**Restart Manager en `windows-rs`.** El "¿quién bloquea este archivo?" del cache cleaner se resuelve con la secuencia `RmStartSession → RmRegisterResources → RmGetList → RmEndSession`, llamando a `RmGetList` dos veces (descubrir tamaño, luego rellenar el buffer de `RM_PROCESS_INFO`). Requiere la feature `Win32_System_RestartManager`. Está codificado en `win32-process-manager`.

---

## 5. Skills nuevas creadas (entregables)

Cinco skills, una por hueco crítico, cubriendo de M1 a M5. Cada una sigue el estilo del proyecto (frontmatter con `description` específica para que Claude la cargue solo cuando aplica, prosa en español, foco seguridad/idempotencia/reversibilidad) y referencia las skills existentes en lugar de duplicarlas.

| Skill | Milestone | Por qué es de alto impacto |
|---|---|---|
| **`error-model`** | M1 (base transversal) | Codifica el contrato AppError(Rust)↔normalizeError/toast/ErrorBoundary(React). Es el bug que "mata la UX"; cada módulo nuevo hereda este patrón, así que es la skill con mayor efecto multiplicador. |
| **`win32-process-manager`** | M2 (P0) | sysinfo+Win32, blacklist de procesos protegidos, kill/suspend/resume, cierre grácil WM_CLOSE→Terminate, y `who_locks_path` con Restart Manager. Desbloquea el cache cleaner v2. |
| **`react-frontend-cleartool`** | M4 + transversal | Convenciones reales del frontend (IPC tipado en `lib/tauri.ts`, TanStack Query vs Zustand, ConfirmDialog+dry-run, virtualización, themes CSS-vars, i18n, lazy routes). Llena el hueco que las skills de diseño genéricas no cubren. |
| **`tauri-release-signing`** | M5 | NSIS/MSI, Azure Trusted Signing en CI, claves del updater + latest.json + canales, winget, `release.yml`. El área más débil del proyecto, con la distinción de las dos firmas bien marcada. |
| **`rust-testing-ci`** | Transversal (arranca M1/M2) | Los 4 tests no negociables de todo módulo destructivo (allowlist, dry-run inerte, idempotencia, revert), los 3 niveles de test, testing en VM, y `ci.yml` con fmt/clippy/test/lint/build. |

Las cinco están en `skills-propuestas/<nombre>/SKILL.md`. Como `.claude/` es de solo lectura en este entorno, se entregan ahí para que las copies a `.claude/skills/`.

### Cómo instalarlas

```powershell
# desde la raíz del repo
Copy-Item -Recurse skills-propuestas\error-model              .claude\skills\
Copy-Item -Recurse skills-propuestas\win32-process-manager    .claude\skills\
Copy-Item -Recurse skills-propuestas\react-frontend-cleartool .claude\skills\
Copy-Item -Recurse skills-propuestas\tauri-release-signing    .claude\skills\
Copy-Item -Recurse skills-propuestas\rust-testing-ci          .claude\skills\
```

Después, añade las filas correspondientes a la tabla de `.claude/skills/README.md` y, si quieres, una entrada en `HISTORIAL-SESIONES.md`.

---

## 6. Recomendaciones de orden (atado al "orden de batalla")

1. **Ahora (M1):** instalar `error-model` y `rust-testing-ci`, y arrancar `ci.yml` (lint+build) — es lo que el propio roadmap pone primero y lo que evita arrastrar deuda a todos los módulos siguientes.
2. **M2:** `win32-process-manager` antes del rewrite de cache (el cache depende de `who_locks_path`).
3. **M3:** empezar **ya** el trámite de Azure Trusted Signing (tarda 1-3 días de validación de identidad) aunque la skill `tauri-release-signing` se use a fondo en M5.
4. **M4:** `react-frontend-cleartool` como guía del refactor de UI; revisar si las skills de diseño 3D/branding siguen aportando o se archivan.
5. **M5:** `tauri-release-signing` para todo el pipeline de release.
6. **Continuo:** usar `security-review` (ya disponible) en cada PR que toque registro/servicios/borrado, y el plugin `engineering` para `testing-strategy`/`deploy-checklist`.

### Posibles skills futuras (no creadas aún)

- `cache-engine-v2` — extender `cache-scanner` con MoveFileEx/PendingFileRenameOperations, retry con backoff y el modelo `CleanPlan` (ready/blocked/permission/skipped). Encaja en M2.
- `debloat-catalog-v2` — schema v2, detección multi-método y curación por tiers. Encaja en M3.
- `treemap-disk-analyzer` — d3-hierarchy + canvas para el Disk Analyzer de M4.

Estas tres son candidatas claras, pero dependen de decisiones de diseño que aún se están cerrando en sus specs; conviene crearlas cuando entres a cada milestone para que reflejen el diseño final y no especulación.
