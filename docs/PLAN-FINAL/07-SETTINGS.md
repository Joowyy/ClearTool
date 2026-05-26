# 07 — Settings (preferencias persistentes y panel global)

> **Posición:** 7/14.
> **Dependencias:** [02-AUDIT-LOG](02-AUDIT-LOG.md), [01-RESTORE-POINTS](01-RESTORE-POINTS.md).
> **Output:** preferencias persistentes en disco, dry-run global, panel del audit log embebido, link a recursos.

---

## 1. Resumen ejecutivo

Estado actual: solo theme picker en memoria (Zustand). Sin persistencia, sin más opciones.

Cierre: archivo `%APPDATA%\ClearTool\settings.json` cargado al startup. UI reorganizada con secciones. Setting clave: **dry-run global override**, que fuerza todos los módulos a dry-run aunque el usuario desmarque.

---

## 2. Diagnóstico

### 2.1 Categorías de configuración

| Categoría | Settings |
|---|---|
| **Apariencia** | Theme (dark/light/system), idioma (ES/EN), tamaño UI |
| **Seguridad** | Dry-run global, crear restore point automáticamente |
| **Comportamiento** | Confirmación pre-batch obligatoria, auto-update check |
| **Privacidad** | (vacía — no hay telemetría) |
| **Avanzado** | Audit log path (read-only), nivel de logging operativo, abrir carpeta de logs |
| **Acerca de** | Versión, build, links, créditos, licencia |

### 2.2 ¿Dónde persistir?

Opciones:

- `%APPDATA%\ClearTool\settings.json` — JSON simple, fácil de inspeccionar/editar manualmente.
- Registro de Windows — más "nativo" pero requiere instalación y rompe portabilidad.
- Tauri Store plugin — usa archivo JSON internamente, simplifica el API.

**Decisión:** `tauri-plugin-store` (ya viable). El JSON queda en `%APPDATA%\com.cleartool.app\settings.dat` por default; redirigir a `%APPDATA%\ClearTool\settings.json` para coherencia.

### 2.3 Carga al startup

`core::settings::load_or_default()` corre al arrancar la app. Si el JSON no existe, escribe los defaults. Si está corrupto, hace backup `.bak` y restaura defaults.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| `tauri-plugin-store` para persistencia | Battle-tested, atomic writes |
| Path explícito en `%APPDATA%\ClearTool\settings.json` | Inspeccionable manualmente |
| Carga sync al startup en `lib.rs::run` | Settings disponibles antes de la primera ventana |
| Schema versionado (`settingsVersion: 1`) | Migraciones futuras detectables |
| Defaults conservadores | "Crear restore point" ON, "dry-run global" OFF |
| UI Settings con tabs (Apariencia / Seguridad / Avanzado / Acerca) | Navegación clara |

---

## 4. Modelo de datos

```rust
// models/settings.rs (nuevo)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings_version: u32,
    pub appearance: AppearanceSettings,
    pub safety: SafetySettings,
    pub behavior: BehaviorSettings,
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,        // "dark" | "light" | "system"
    pub language: String,     // "es" | "en" | "system"
    pub density: String,      // "compact" | "normal" | "comfortable"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySettings {
    pub dry_run_global: bool,                  // fuerza todos los módulos a dry-run
    pub auto_create_restore_point: bool,
    pub require_confirm_before_batch: bool,
    pub bypass_throttling_restore: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorSettings {
    pub check_updates_on_start: bool,
    pub last_seen_audit_run_id: Option<String>,  // marca para badge "nuevo" en menú
    pub remember_window_size: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSettings {
    pub log_level: String,        // "error" | "warn" | "info" | "debug"
    pub audit_log_max_mb: u32,    // umbral de rotación
    pub diagnostic_mode: bool,    // expone botones "abrir log", "exportar audit"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: 1,
            appearance: AppearanceSettings {
                theme: "dark".into(),
                language: "system".into(),
                density: "normal".into(),
            },
            safety: SafetySettings {
                dry_run_global: false,
                auto_create_restore_point: true,
                require_confirm_before_batch: true,
                bypass_throttling_restore: true,
            },
            behavior: BehaviorSettings {
                check_updates_on_start: true,
                last_seen_audit_run_id: None,
                remember_window_size: true,
            },
            advanced: AdvancedSettings {
                log_level: "info".into(),
                audit_log_max_mb: 10,
                diagnostic_mode: false,
            },
        }
    }
}
```

---

## 5. Plan UX

### 5.1 Pantalla Settings con tabs

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Ajustes                                                                   │
│  [Apariencia]  [Seguridad]  [Comportamiento]  [Avanzado]  [Acerca de]    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Apariencia                                                               │
│                                                                           │
│  Tema visual                                                              │
│   ● Oscuro     ○ Claro     ○ Seguir sistema                              │
│                                                                           │
│  Idioma                                                                   │
│   ● Español    ○ English   ○ Seguir sistema                              │
│                                                                           │
│  Densidad de UI                                                           │
│   ○ Compacta   ● Normal    ○ Cómoda                                      │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Tab Seguridad

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Seguridad                                                                │
│                                                                           │
│  ☐ Modo Dry-Run global                                                    │
│      Fuerza todos los módulos destructivos a simular sin tocar nada.     │
│      Recomendado para sesiones de exploración. Visible como banner       │
│      naranja en la app cuando está activo.                                │
│                                                                           │
│  ☑ Crear restore point automáticamente antes de operaciones destructivas │
│      Recomendado. Es la red de seguridad principal.                       │
│                                                                           │
│  ☑ Pedir confirmación antes de aplicar un batch                           │
│      Muestra modal con preview detallado.                                 │
│                                                                           │
│  ☑ Bypass del throttling 24h de System Restore                            │
│      Permite crear múltiples puntos en una sesión. Restaura el valor     │
│      previo del SO al terminar.                                           │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Tab Comportamiento

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Comportamiento                                                           │
│                                                                           │
│  ☑ Buscar actualizaciones al arrancar                                     │
│      ClearTool consulta updates.cleartool.app/manifest.json. Sin datos   │
│      personales enviados.                                                 │
│                                                                           │
│  ☑ Recordar tamaño y posición de la ventana                               │
│                                                                           │
│  ─────────────────────────────────────────────                            │
│                                                                           │
│  Audit log                                                                │
│   • Path:    %APPDATA%\ClearTool\audit.jsonl                              │
│   • Tamaño:  124 KB                                                       │
│   • Entries: 42                                                           │
│                                                                           │
│  [Abrir audit log]  [Ver pantalla audit]  [Exportar log]                  │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.4 Tab Avanzado

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Avanzado                                                                 │
│                                                                           │
│  Nivel de log operativo                                                   │
│   ○ Error  ○ Warn  ● Info  ○ Debug                                       │
│   El log técnico vive en %LOCALAPPDATA%\ClearTool\logs\app.log            │
│                                                                           │
│  Tamaño máximo del audit antes de rotar (MB)                              │
│   [10] MB                                                                 │
│                                                                           │
│  ☐ Modo diagnóstico                                                       │
│      Habilita botones técnicos: "abrir log técnico", "exportar audit",   │
│      "vaciar audit log" (¡destructivo!).                                  │
│                                                                           │
│  ─────────────────────────────────────────────                            │
│                                                                           │
│  [Restaurar valores por defecto]                                          │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.5 Tab Acerca de

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ClearTool                                                                │
│                                                                           │
│  Versión:        1.0.0                                                    │
│  Build:          2026-05-21 1f3a4b5                                       │
│  Stack:          Tauri 2.x + Rust + React + TypeScript                    │
│                                                                           │
│  Licencia:       MIT — ver LICENSE                                        │
│                                                                           │
│  Recursos:                                                                │
│   • Repo en GitHub                                                        │
│   • Reporte de issues                                                     │
│   • Documentación del proyecto                                            │
│                                                                           │
│  Créditos: comunidad open-source — winreg, windows-rs, walkdir,           │
│  tauri, react, tanstack, framer-motion.                                   │
│                                                                           │
│  Hecho con cuidado, sin telemetría, sin tracking, sin anuncios.           │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.6 Banner de Dry-Run global activo

Cuando `dryRunGlobal=true`, todas las páginas muestran arriba:

```
┌────────────────────────────────────────────────────────────────────────┐
│ ⚠  Modo Dry-Run activo — ninguna operación toca el sistema realmente.  │
│    Desactivá en Ajustes → Seguridad para aplicar cambios.              │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Plan de implementación por fases

### Fase 1 — Backend: módulo `core::settings`

#### Paso 1.1 — Implementación con archivo JSON propio (sin plugin)

> Para evitar otra dependencia y mantener control, no usamos `tauri-plugin-store`. Implementamos read/write atómico de un JSON.

**Archivo:** `src-tauri/src/core/settings.rs` (nuevo)

```rust
use crate::core::{AppError, AppResult, config};
use crate::models::settings::Settings;
use std::sync::RwLock;
use std::sync::OnceLock;

static SETTINGS: OnceLock<RwLock<Settings>> = OnceLock::new();

pub fn settings_path() -> std::path::PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(appdata).join("ClearTool").join("settings.json")
}

pub fn load_or_default() -> Settings {
    let path = settings_path();
    if !path.exists() {
        let def = Settings::default();
        let _ = write_atomic(&def);
        return def;
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str::<Settings>(&s) {
            Ok(parsed) => {
                if parsed.settings_version != 1 {
                    log::warn!("settings versión desconocida: {}, usando defaults", parsed.settings_version);
                    let def = Settings::default();
                    let _ = write_atomic(&def);
                    def
                } else { parsed }
            }
            Err(e) => {
                log::warn!("settings corruptos: {}, restaurando defaults", e);
                let bak = path.with_extension("json.bak");
                let _ = std::fs::rename(&path, &bak);
                let def = Settings::default();
                let _ = write_atomic(&def);
                def
            }
        },
        Err(_) => Settings::default(),
    }
}

pub fn init() {
    let loaded = load_or_default();
    let _ = SETTINGS.set(RwLock::new(loaded));
}

pub fn get() -> Settings {
    SETTINGS.get()
        .expect("settings::init no llamado")
        .read()
        .map(|s| s.clone())
        .unwrap_or_default()
}

pub fn update(new_settings: Settings) -> AppResult<()> {
    write_atomic(&new_settings)?;
    if let Some(lock) = SETTINGS.get() {
        *lock.write().map_err(|_| AppError::Validation("settings lock poisoned".into()))? = new_settings;
    }
    Ok(())
}

fn write_atomic(s: &Settings) -> AppResult<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("json.tmp");
    let payload = serde_json::to_string_pretty(s)
        .map_err(|e| AppError::Audit(format!("serialize settings: {}", e)))?;
    std::fs::write(&tmp, payload)
        .map_err(|e| AppError::Audit(format!("write tmp: {}", e)))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| AppError::Audit(format!("rename atomic: {}", e)))?;
    Ok(())
}

pub fn reset_to_defaults() -> AppResult<()> {
    update(Settings::default())
}

/// Helper consultable desde otros módulos para honrar dry-run global.
pub fn is_dry_run_global() -> bool {
    get().safety.dry_run_global
}
```

#### Paso 1.2 — Llamar `settings::init()` en `lib.rs::run`

**Archivo:** `src-tauri/src/lib.rs`

```rust
// dentro de run() antes de construir tauri::Builder
crate::core::settings::init();
crate::domain::catalog::validate_all_at_startup();
```

#### Paso 1.3 — Honrar `dry_run_global` en los módulos destructivos

En cada `domain::*::apply/remove/clean`:

```rust
let effective_dry_run = input.dry_run || crate::core::settings::is_dry_run_global();
```

Pasar `effective_dry_run` al resto del código en lugar de `input.dry_run`. Si la setting global está ON, **nunca** se aplica nada real, aunque el caller pida.

### Fase 2 — IPC commands

#### Paso 2.1 — `ipc::settings`

**Archivo:** `src-tauri/src/ipc/settings.rs` (nuevo)

```rust
use crate::core::{AppResult, settings};
use crate::models::settings::Settings;

#[tauri::command]
pub async fn get_settings() -> AppResult<Settings> {
    Ok(settings::get())
}

#[tauri::command]
pub async fn update_settings(updated: Settings) -> AppResult<()> {
    settings::update(updated)
}

#[tauri::command]
pub async fn reset_settings_to_defaults() -> AppResult<Settings> {
    settings::reset_to_defaults()?;
    Ok(settings::get())
}

#[tauri::command]
pub async fn settings_file_path() -> AppResult<String> {
    Ok(settings::settings_path().to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn open_settings_file() -> AppResult<()> {
    let p = settings::settings_path();
    if let Some(parent) = p.parent() {
        let _ = open::that(parent);
    }
    Ok(())
}
```

#### Paso 2.2 — Registrar en `lib.rs`

Añadir 5 comandos al `tauri::generate_handler![...]`.

#### Paso 2.3 — Crear `mod.rs` `ipc::settings`

Actualizar `src-tauri/src/ipc/mod.rs`:

```rust
pub mod audit;
pub mod cache;
pub mod debloat;
pub mod explorer;
pub mod registry;
pub mod restore;
pub mod services;
pub mod settings;
pub mod system_info;
pub mod telemetry;
```

### Fase 3 — Frontend

#### Paso 3.1 — API client

```ts
// src/api/client.ts
import type { Settings } from "./types";

export const getSettings = () => invoke<Settings>("get_settings");
export const updateSettings = (settings: Settings) =>
  invoke<void>("update_settings", { updated: settings });
export const resetSettingsToDefaults = () => invoke<Settings>("reset_settings_to_defaults");
export const settingsFilePath = () => invoke<string>("settings_file_path");
export const openSettingsFile = () => invoke<void>("open_settings_file");
```

#### Paso 3.2 — Tipos TS

```ts
// src/api/types.ts
export interface Settings {
  settingsVersion: number;
  appearance: AppearanceSettings;
  safety: SafetySettings;
  behavior: BehaviorSettings;
  advanced: AdvancedSettings;
}
export interface AppearanceSettings {
  theme: "dark" | "light" | "system";
  language: "es" | "en" | "system";
  density: "compact" | "normal" | "comfortable";
}
export interface SafetySettings {
  dryRunGlobal: boolean;
  autoCreateRestorePoint: boolean;
  requireConfirmBeforeBatch: boolean;
  bypassThrottlingRestore: boolean;
}
export interface BehaviorSettings {
  checkUpdatesOnStart: boolean;
  lastSeenAuditRunId: string | null;
  rememberWindowSize: boolean;
}
export interface AdvancedSettings {
  logLevel: "error" | "warn" | "info" | "debug";
  auditLogMaxMb: number;
  diagnosticMode: boolean;
}
```

#### Paso 3.3 — Hook + store global

**Archivo:** `src/features/settings/use-settings.ts` (nuevo)

```ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getSettings, updateSettings, resetSettingsToDefaults, type Settings } from "../../api";

export function useSettings() {
  return useQuery({ queryKey: ["settings"], queryFn: getSettings });
}

export function useUpdateSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (s: Settings) => updateSettings(s),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["settings"] }),
  });
}

export function useResetSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: resetSettingsToDefaults,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["settings"] }),
  });
}
```

#### Paso 3.4 — Banner global de Dry-Run

**Archivo:** `src/components/layout/dry-run-banner.tsx` (nuevo)

```tsx
import { useSettings } from "../../features/settings/use-settings";

export function DryRunBanner() {
  const { data: settings } = useSettings();
  if (!settings?.safety.dryRunGlobal) return null;
  return (
    <div className="bg-amber-500/20 border-b border-amber-500/40 text-amber-200 px-4 py-2 text-sm flex items-center gap-2">
      <span>⚠</span>
      <span>Modo Dry-Run activo — ninguna operación toca el sistema realmente.</span>
      <a href="/settings" className="underline ml-auto">Desactivar</a>
    </div>
  );
}
```

Montar en `app-shell.tsx` arriba del contenido.

#### Paso 3.5 — Reescritura de `settings-page.tsx`

**Archivo:** `src/features/settings/settings-page.tsx`

```tsx
import { useState } from "react";
import { useSettings, useUpdateSettings, useResetSettings } from "./use-settings";
import { AppearanceTab } from "./components/appearance-tab";
import { SafetyTab } from "./components/safety-tab";
import { BehaviorTab } from "./components/behavior-tab";
import { AdvancedTab } from "./components/advanced-tab";
import { AboutTab } from "./components/about-tab";
import type { Settings } from "../../api";

const TABS = [
  { id: "appearance", label: "Apariencia" },
  { id: "safety", label: "Seguridad" },
  { id: "behavior", label: "Comportamiento" },
  { id: "advanced", label: "Avanzado" },
  { id: "about", label: "Acerca de" },
] as const;

type TabId = (typeof TABS)[number]["id"];

export function SettingsPage() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const reset = useResetSettings();
  const [tab, setTab] = useState<TabId>("appearance");

  if (!settings) return <div className="p-6">Cargando ajustes...</div>;

  const onPatch = (patch: Partial<Settings>) => {
    update.mutate({ ...settings, ...patch });
  };

  return (
    <div className="p-6 space-y-4 max-w-3xl">
      <h2 className="text-2xl font-bold">Ajustes</h2>
      <nav className="flex gap-2 border-b border-border">
        {TABS.map((t) => (
          <button
            key={t.id}
            className={`px-3 py-2 text-sm border-b-2 ${tab === t.id ? "border-primary" : "border-transparent text-muted-foreground"}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </nav>
      {tab === "appearance" && <AppearanceTab settings={settings} onPatch={onPatch} />}
      {tab === "safety" && <SafetyTab settings={settings} onPatch={onPatch} />}
      {tab === "behavior" && <BehaviorTab settings={settings} onPatch={onPatch} />}
      {tab === "advanced" && (
        <AdvancedTab
          settings={settings}
          onPatch={onPatch}
          onReset={() => {
            if (confirm("¿Restaurar valores por defecto? Tus preferencias actuales se perderán.")) {
              reset.mutate();
            }
          }}
        />
      )}
      {tab === "about" && <AboutTab />}
    </div>
  );
}
```

Cada `*-tab.tsx` es ~30 líneas mostrando los inputs según mockups §5.

### Fase 4 — Integración con módulos destructivos

#### Paso 4.1 — Honrar `dry_run_global`

En `domain::registry::apply`:

```rust
pub fn apply(input: &ApplyTweakInput) -> AppResult<()> {
    let dry_run = input.dry_run || crate::core::settings::is_dry_run_global();
    // resto usa `dry_run` en lugar de `input.dry_run`
}
```

Replicar en `domain::services::set_state`, `apply_preset`, `domain::debloat::remove`, `domain::cache::clean`.

#### Paso 4.2 — Honrar `auto_create_restore_point`

En cada batch:

```rust
let should_restore = input.create_restore_point
    && crate::core::settings::get().safety.auto_create_restore_point;
```

#### Paso 4.3 — Honrar `require_confirm_before_batch` (frontend)

Los modales pre-batch (`BatchConfirmModal`) siempre se muestran. Esta setting permite **skippearlos** en sesiones avanzadas:

```tsx
const settings = useSettings().data;
const skipConfirm = !settings?.safety.requireConfirmBeforeBatch;

const handleApply = () => {
  if (skipConfirm) {
    applyBatch(payload);
  } else {
    setConfirmOpen(true);
  }
};
```

---

## 7. Tests

### 7.1 Test de roundtrip settings

```rust
#[test]
fn settings_roundtrip() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::env::set_var("APPDATA", tmp.path());

    let mut s = cleartool::models::settings::Settings::default();
    s.safety.dry_run_global = true;
    s.appearance.theme = "light".into();

    cleartool::core::settings::init();
    cleartool::core::settings::update(s.clone()).unwrap();

    // Re-init para forzar reload del disco
    cleartool::core::settings::init();
    let loaded = cleartool::core::settings::get();
    assert!(loaded.safety.dry_run_global);
    assert_eq!(loaded.appearance.theme, "light");
}

#[test]
fn settings_corruptos_restauran_defaults() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::env::set_var("APPDATA", tmp.path());
    let path = cleartool::core::settings::settings_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"esto-no-es-json").unwrap();

    let loaded = cleartool::core::settings::load_or_default();
    assert_eq!(loaded.appearance.theme, "dark");  // default
    assert!(path.with_extension("json.bak").exists());
}
```

### 7.2 Test manual VM

1. `/settings` → tab Seguridad → activar "Modo Dry-Run global".
2. Banner amarillo aparece en todas las pantallas.
3. `/registry` → seleccionar tweak → aplicar.
4. Verificar que **no escribió** al registro (audit log entry con `dry_run: true`).
5. Desactivar dry-run global.
6. Tab Comportamiento → ver path del audit log + tamaño actualizado.
7. Tab Avanzado → activar "Modo diagnóstico" → aparece botón "Vaciar audit log".

---

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Settings.json corrupto rompe app | `load_or_default` con backup `.bak` |
| Race condition: dos writes simultáneos | `RwLock` global + write atómico vía rename |
| Usuario desactiva auto-restore y luego rompe sistema | Banner de advertencia al desactivar; doble confirmación |
| Path appdata no existe (raro) | `create_dir_all` antes de cualquier write |
| Idioma `system` no resuelve | Fallback a `es` para v1.0 (mercado primario) |

---

## 9. Definition of Done

- [ ] `models::settings` con 4 sub-structs + defaults conservadores.
- [ ] `core::settings::{init, get, update, reset_to_defaults, is_dry_run_global}` implementados.
- [ ] Write atómico via tmp + rename.
- [ ] Backup `.bak` si JSON corrupto.
- [ ] IPC: 5 comandos (`get`, `update`, `reset`, `path`, `open_file`).
- [ ] `settings::init` llamado en `lib.rs::run`.
- [ ] Frontend: 5 tabs implementados.
- [ ] Banner `DryRunBanner` global en app-shell.
- [ ] Módulos destructivos honran `is_dry_run_global` (mín. cache, registry, services, debloat).
- [ ] Tab "Acerca de" muestra versión correcta (leída de `Cargo.toml`).
- [ ] Tests roundtrip + corruptos pasan.
- [ ] Test manual VM verifica dry-run global aplica realmente.
- [ ] Commit `feat(settings): preferencias persistentes + dry-run global + panel completo`.

---

## 10. Próximo archivo

→ [08-SEGURIDAD-FINAL.md](08-SEGURIDAD-FINAL.md) — auditoría cruzada pre-release. Es un **gate**: nada pasa a tests/distribución hasta que el security checklist esté cerrado.
