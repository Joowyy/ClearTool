# v2 · 05 — Módulos nuevos compactos (P2)

Este spec agrupa 6 módulos nuevos que comparten patrón: cada uno es un add-on relativamente autocontenido. Se documentan juntos para evitar fragmentación de specs cortas.

| Módulo | Prioridad | Esfuerzo | Por qué |
|--------|-----------|----------|---------|
| Startup Manager | P1 | M | Esencial. Hoy hay que ir a Task Manager → Startup. |
| Boot-time Cleanup | P1 | S | Necesario para complementar Cache Cleaner v2. |
| Disk Analyzer (Treemap) | P2 | L | Feature "wow". Diferenciación clara vs CCleaner. |
| Network Utilities | P2 | S | Quality-of-life. Solos no justifican download pero suman. |
| Privacy Hardening | P2 | M | Bundle de tweaks de registro para "modo paranoid" en 1 click. |
| App Reset (UWP) | P3 | S | Ya bocetado en `02-cache-engine`. |

Esfuerzo: S = 1-3 días, M = 1 semana, L = 2-3 semanas.

---

## 5.1 Startup Manager

### Problema
El usuario instala 30 apps y todas se autoarrancan. Task Manager → Startup es la única UI pero:
- Sólo muestra entradas Run/RunOnce del usuario actual.
- No muestra scheduled tasks que corren al logon.
- No estima impacto real (todos dicen "alto/medio/bajo" sin medirlo).
- No hay forma de saber qué hace cada entrada.

### Alcance
Cuatro categorías de auto-arranque:

```rust
pub enum StartupOrigin {
    Registry(StartupRegistryLocation),  // HKLM/HKCU Run, RunOnce
    StartupFolder(PathBuf),              // ...\Programs\Startup\*.lnk
    ScheduledTask(String),               // tareas con trigger "at logon"
    Service(String),                     // services con StartType = Automatic
    UwpAutoStart(String),                // AppxManifest Extension > "startupTask"
}
```

### IPC
```rust
#[tauri::command] pub async fn list_startup_entries() -> Result<Vec<StartupEntry>>;
#[tauri::command] pub async fn disable_startup(id: String) -> Result<()>;
#[tauri::command] pub async fn enable_startup(id: String) -> Result<()>;
#[tauri::command] pub async fn measure_startup_impact() -> Result<StartupImpactReport>;
```

### Modelo
```rust
pub struct StartupEntry {
    pub id: String,
    pub origin: StartupOrigin,
    pub display_name: String,
    pub command: String,        // qué se ejecuta
    pub icon_path: Option<String>,
    pub publisher: Option<String>, // de la firma del exe
    pub signature_valid: Option<bool>,
    pub impact: StartupImpact,
    pub last_modified: String,
    pub enabled: bool,
    pub category: StartupCategory, // updater/launcher/widget/cloudsync/...
}

pub enum StartupImpact {
    Unknown,
    Low,    // <100ms estimados al login
    Medium, // 100ms-500ms
    High,   // >500ms o >50MB RAM
}
```

### Medición de impacto
Windows ya tiene esto (Performance Monitor `Microsoft-Windows-Diagnostics-Performance/Operational` event log). Consultar:
```powershell
Get-WinEvent -LogName 'Microsoft-Windows-Diagnostics-Performance/Operational' |
  Where-Object { $_.Id -eq 100 } |
  Select-Object -First 1 -ExpandProperty Properties
```
Los eventos `100` (BootTime) y `200` (LogonTime) llevan breakdown por proceso. Parsear estos eventos en background y cachear en `%APPDATA%\ClearTool\startup-impact.json`.

### UX
```
┌──────────────────────────────────────────────────────────────────┐
│ Arranque automático                                              │
│ 23 entradas · 3 alto impacto · ~4.2s al login                    │
│                                                                  │
│ ⚠ Alto impacto (3)                                               │
│  ⏵ Adobe Creative Cloud         Adobe Inc. · 1.4s · 320 MB       │
│      Updater + tray. [Deshabilitar] [Ver detalles]               │
│  ⏵ Spotify                      Spotify AB · 0.8s · 280 MB       │
│  ⏵ Discord                      Discord Inc. · 0.7s · 240 MB     │
│                                                                  │
│ ▽ Medio impacto (8)                                              │
│ ▽ Bajo impacto (12)                                              │
└──────────────────────────────────────────────────────────────────┘
```

### Reversibilidad
Audit log cada disable/enable. `disable` → mueve la entrada con sufijo `.disabled` (en lugar de borrar) para que el revert sea trivial.

---

## 5.2 Boot-time Cleanup

### Problema
Archivos memory-mapped, drivers, page file fragments → sólo se eliminan en reboot. Hoy ClearTool intenta `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` pero:
- Falla en muchos UWP packages (acceso denegado).
- No hay forma de **ver** qué hay pendiente para reboot.
- Si el usuario no reinicia en 7 días, queda colgado.

### Alcance
- Listar pending file renames de `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\PendingFileRenameOperations`.
- Permitir cancelar pendientes (sacar del registry).
- Programar nuevos pendientes (alternativa a MoveFileEx con mejor logging).
- Detectar si algo está pendiente desde hace mucho tiempo y avisar.

### IPC
```rust
#[tauri::command] pub async fn list_pending_renames() -> Result<Vec<PendingRename>>;
#[tauri::command] pub async fn cancel_pending_rename(source_path: String) -> Result<()>;
#[tauri::command] pub async fn schedule_delete_on_reboot(path: String) -> Result<()>;
#[tauri::command] pub async fn pending_count() -> Result<u32>;
```

### UI
No tiene página dedicada (no lo merece). En su lugar:
- En **Settings → Avanzado**: card "Operaciones pendientes para próximo reinicio: 14 archivos (1.2 GB)".
- En **Caché** tras un clean: card "Programados para reboot: 4 archivos".
- Badge en titlebar (mini) si hay pendientes > 50 archivos (señal de "reinicia ya").

---

## 5.3 Disk Analyzer (Treemap)

### Problema
El Explorer Tree actual lista carpetas con sus tamaños, pero pesado a la vista. WinDirStat lo resuelve con treemap visual: rectángulos proporcionales al tamaño, colores por extensión.

### Tech stack
Tres opciones evaluadas:

**A. Canvas 2D con react-d3-treemap** — recomendada.
- Pro: <50KB extra, performance OK hasta ~10k nodos.
- Con: re-render todo en cada zoom.

**B. WebGL via R3F / @react-three/drei** — overkill para v1.
- Pro: GPU-accelerated, escala a 100k nodos.
- Con: 300KB extra, complejidad alta.

**C. SVG nativo** — descartado (mal performance para >2k rectángulos).

**Decisión**: A (Canvas 2D con d3-hierarchy + react). Migrar a R3F si en testing se ve necesario.

### Backend
Reutiliza `compute_directory_size` que ya existe. Nuevo comando para devolver el árbol entero serializable:

```rust
#[tauri::command]
pub async fn build_treemap_data(
    root: String,
    max_depth: u32,           // default 4 (raíz + 3 niveles)
    min_size_mb: u64,         // default 10 (filtra ruido)
    follow_reparse_points: bool,
) -> Result<TreemapNode>;
```

```rust
pub struct TreemapNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: NodeKind,
    pub extension: Option<String>,  // para colorear hojas-file
    pub children: Vec<TreemapNode>,
}
```

### Frontend
```
┌─────────────────────────────────────────────────────────┐
│ Disk Analyzer · C:\                                     │
│ 488 GB total · 380 GB usados                            │
│                                                         │
│  ┌───────────┬───┬─────┬───┐                            │
│  │ Users     │   │     │   │                            │
│  │ 120 GB    │   │     │   │                            │
│  │           │   │     │   │                            │
│  │  ┌──┬──┐  │   │     │   │                            │
│  │  │  │  │  │   │     │   │                            │
│  │  └──┴──┘  │   │     │   │                            │
│  ├───────────┤   │     │   │                            │
│  │ Program   │   │     │   │                            │
│  │ Files     │   └─────┘   │                            │
│  │ 80 GB     │             │                            │
│  └───────────┴─────────────┘                            │
│                                                         │
│  Click para zoom · Doble click para "Eliminar carpeta"  │
└─────────────────────────────────────────────────────────┘
```

Color por extensión: `.mp4` rojo, `.iso` púrpura, `.exe` cyan, etc.

### Acciones desde el treemap
- Hover: tooltip con path completo + tamaño.
- Click: zoom in (esa carpeta llena el viewport).
- Doble click: "Abrir en explorador" / "Eliminar carpeta" (con confirmación + restore point si > 1 GB).

---

## 5.4 Network Utilities

### Alcance
Comandos sin UI propia, accesibles desde un menú **Settings → Herramientas de red**:

```
[ DNS Flush                  ] → ipconfig /flushdns
[ Renovar IP                 ] → ipconfig /release && ipconfig /renew
[ Resetear Winsock           ] → netsh winsock reset
[ Resetear TCP/IP            ] → netsh int ip reset
[ Limpiar caché de proxy     ] → netsh winhttp reset proxy
[ Restaurar host file        ] → reescribir C:\Windows\System32\drivers\etc\hosts a defaults
```

Cada uno requiere admin. Cada uno se confirma antes (modal). Audit log obligatorio. Restore point automático (los resets de tcpip pueden fastidiar la conectividad si hay configuración custom).

### IPC
```rust
#[tauri::command] pub async fn flush_dns() -> Result<()>;
#[tauri::command] pub async fn renew_ip() -> Result<()>;
#[tauri::command] pub async fn reset_winsock() -> Result<()>;
#[tauri::command] pub async fn reset_tcpip() -> Result<()>;
#[tauri::command] pub async fn reset_proxy() -> Result<()>;
#[tauri::command] pub async fn restore_hosts_file() -> Result<()>;
```

### UX
No es una página propia. Drawer/popover desde Settings con 6 botones. Cada uno tiene un tooltip explicando qué hace.

---

## 5.5 Privacy Hardening

### Alcance
Bundle de tweaks de registro y servicios para "modo privacidad agresivo". No es un módulo nuevo — es un **preset bundle** que reutiliza Registry Tweaks + Services + Debloat.

### El preset "Modo Paranoid"
Un toggle en Settings o un botón en la Home que aplica simultáneamente:

**Registry tweaks (~25 keys):**
- Advertising ID off
- Cortana off
- WebSearch off
- ActivityHistory off
- Telemetría = 0 (security only)
- Location off
- Lock screen tips off
- App suggestions off
- Tailored experiences off

**Services (~10 servicios):**
- DiagTrack → Disabled
- dmwappushsvc → Disabled
- WerSvc (Windows Error Reporting) → Disabled
- RetailDemo → Disabled

**Scheduled tasks (~15 tareas):**
- Customer Experience Improvement Program
- Application Experience Service Initialization
- ...

**Apps (debloat):**
- Cortana, Copilot, ConnectedUserExperiencesAndTelemetry.

### IPC
```rust
#[tauri::command] pub async fn apply_privacy_preset(level: PrivacyLevel, dry_run: bool) -> Result<PrivacyReport>;

pub enum PrivacyLevel {
    Balanced,    // tweaks "evidentes", reversibles, sin romper UX (Cortana off, ads off)
    Strict,      // los anteriores + más servicios off, telemetría min
    Paranoid,    // todo lo anterior + Defender SmartScreen reducido (advertencia gorda)
}
```

### UI
Página dedicada o sección dentro de Settings:

```
Privacidad
○ Sin cambios — Windows por defecto
● Equilibrado — Quita publicidad y telemetría no esencial (recomendado)
○ Estricto — Privacidad fuerte. Algunas búsquedas web menos integradas
○ Paranoid — Privacidad máxima. Lee los disclaimers, esto rompe cosas

[Ver cambios] [Aplicar (con restore point)]
```

"Ver cambios" abre un modal con la lista exacta de:
- Registry keys
- Servicios
- Scheduled tasks
- Apps

Cada uno enlaza a su página dedicada (Registry / Servicios / Debloat) por si el usuario quiere ajustar.

---

## 5.6 App Reset (UWP)

Ya descrito en `02-cache-engine-rewrite.md` como escape hatch. Aquí formalizado como módulo propio dentro de Debloat:

### IPC
```rust
#[tauri::command] pub async fn reset_uwp_app(package_family_name: String) -> Result<()>;
#[tauri::command] pub async fn list_resettable_apps() -> Result<Vec<ResettableApp>>;
```

### UX
Dentro de la página Debloat, junto a cada entry UWP, botón secundario "Reset" (con icono trash + refresh). Confirmar con modal que avisa "perderás sesión y data local de esta app".

```rust
async fn reset_uwp_app(pfn: String) -> AppResult<()> {
    let script = format!(
        r#"Get-AppxPackage -Name "{}" | Reset-AppxPackage"#,
        pfn.replace('"', "")  // basic injection guard, validado por regex antes
    );
    let out = run_script_owned(&script)?;
    if !out.status.success() {
        return Err(AppError::Powershell(/* ... */));
    }
    Ok(())
}
```

---

## Dependencias entre estos módulos

```
Startup Manager  ───┐
Boot Cleanup     ───┤── usan platform/processes + platform/registry
Network Utils    ───┘
Disk Analyzer        ── usa domain/explorer
Privacy Hardening    ── reusa Registry Tweaks + Services + Debloat
App Reset            ── usa platform/debloat
```

Todos comparten:
- Restore point obligatorio antes de aplicar.
- Audit log entry.
- Toast con resultado.
- Soporte dry-run.

---

## Criterio de "done" agregado

Cada módulo: ver su sección. A nivel agregado para v1.0:

- [ ] Startup Manager listado + disable/enable funcional para 4 tipos de origen.
- [ ] Boot Cleanup permite ver y cancelar pendientes.
- [ ] Disk Analyzer con treemap navegable de mínimo 5k nodos.
- [ ] Network Utilities con 6 botones todos funcionales.
- [ ] Privacy Hardening con 3 niveles y dry-run.
- [ ] App Reset operativo para top 10 packages UWP.
