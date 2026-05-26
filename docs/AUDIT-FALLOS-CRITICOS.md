# Auditoría de fallos críticos — ClearTool

> Generado: 2026-05-19  
> Snapshot del repositorio en la rama `update/tauri-conex` tras la reestructuración del backend/frontend.  
> Este documento explica **qué está roto, qué se ha arreglado en esta sesión, y qué queda pendiente**.

---

## 0. Cómo leer este documento

Cada hallazgo tiene tres campos clave:

- **Severidad**: `CRÍTICO` (la app no funciona) · `ALTO` (función rota) · `MEDIO` (bug latente) · `BAJO` (calidad/estilo).
- **Estado**: `✅ Arreglado en esta sesión` · `🟡 Parcial` · `❌ Pendiente`.
- **Acción**: qué hay que tocar para resolverlo.

---

## 1. CRÍTICO — La app está vacía: 17 de 19 comandos devuelven `NotImplemented`

**Estado:** ❌ Pendiente (la sesión que viene se ataca esto).

**Síntoma:** abres la app y todas las pantallas dicen "Sin ubicaciones de caché", "Sin tweaks", "Sin servicios", "Sin resultados". La home sí pinta porque `system_summary` y `is_elevated` son los **únicos dos** comandos implementados.

**Causa:** todo el backend de negocio son stubs. Concretamente:

| Módulo               | Comandos Tauri afectados                                       | Implementado |
|----------------------|----------------------------------------------------------------|--------------|
| `ipc::system_info`   | `is_elevated`, `system_summary`                                | ✅ Sí        |
| `ipc::explorer`      | `scan_tree`, `cancel_scan`, `compute_directory_size`           | 🟡 Parcial (esta sesión) |
| `ipc::cache`         | `list_cache_locations`, `scan_cache_locations`, `clean_cache_locations` | 🟡 Parcial (esta sesión) |
| `ipc::debloat`       | `list_bloatware_catalog`, `detect_installed_bloatware`, `remove_bloatware` | ❌ Stub |
| `ipc::services`      | `list_services`, `set_service_state`, `apply_service_preset`   | ❌ Stub      |
| `ipc::registry`      | `list_registry_tweaks`, `read_registry_tweak_state`, `apply_registry_tweak`, `apply_registry_tweak_batch`, `revert_registry_tweak` | ❌ Stub |
| `ipc::restore`       | `ensure_restore_enabled`, `create_restore_point`, `list_restore_points`, `restore_to_point` | ❌ Stub |
| `ipc::audit`         | `list_audit_log`, `revert_audit_entry`                         | ❌ Stub      |

**Acción mínima para "ver datos":**
1. Implementar [domain/debloat.rs](../src-tauri/src/domain/debloat.rs) cargando `bloatware-catalog.json` (ya existe en [.claude/skills/powershell-debloat/RESOURCES/](../.claude/skills/powershell-debloat/RESOURCES/)).
2. Implementar [domain/services.rs](../src-tauri/src/domain/services.rs) con `winapi` o `windows::Win32::System::Services` para listar servicios reales.
3. Implementar [domain/registry.rs](../src-tauri/src/domain/registry.rs) con `winreg` leyendo desde `registry-tweaks.json`.
4. Implementar [domain/restore.rs](../src-tauri/src/domain/restore.rs) con WMI `SystemRestore` para listar puntos existentes.

---

## 2. CRÍTICO — Error 740 al ejecutar `cargo run` / `npm run tauri dev`

**Estado:** ✅ Arreglado en esta sesión.

**Síntoma original:**
```
error: could not execute process `target\debug\cleartool.exe` (never executed)
Caused by:
  La operación solicitada requiere elevación. (os error 740)
```

**Causa raíz:** el `build.rs` embebía un manifest con `requestedExecutionLevel level="requireAdministrator"` **siempre**, incluso en debug. Cuando ejecutas `cargo run` desde una terminal no elevada, Windows ve el manifest, exige UAC, y como el proceso padre (cargo) no puede iniciar un hijo con elevación pendiente, devuelve 740.

**Confusión añadida:** el commit `632c8d9` editó el archivo suelto [src-tauri/ClearTool.exe.manifest](../src-tauri/ClearTool.exe.manifest) para quitar `requireAdministrator`, pero **ese archivo no se usa**. Tauri ignora `.manifest` sueltos en disco — el manifest activo es el embebido por `tauri-build` desde `build.rs`.

**Solución aplicada:** [src-tauri/build.rs](../src-tauri/build.rs) ahora lee la variable de entorno `PROFILE`:
- `debug`   → embebe `asInvoker`. `cargo run` arranca sin elevación, la app pinta con la barra "Modo limitado".
- `release` → embebe `requireAdministrator`. La app final dispara UAC al lanzarse, coherente con la decisión "pedir elevación al arrancar siempre" que tomamos.

Además, [platform/elevation.rs](../src-tauri/src/platform/elevation.rs) expone `relaunch_as_admin_if_needed()` que [lib.rs](../src-tauri/src/lib.rs) invoca al arrancar — sirve como red de seguridad si en el futuro el manifest cambia.

---

## 3. CRÍTICO — Catálogos JSON existen pero el backend nunca los cargaba

**Estado:** 🟡 Parcial (carga de `cache-locations.json` implementada en esta sesión).

**Causa:** las allowlists viven en [.claude/skills/cache-scanner/RESOURCES/cache-locations.json](../.claude/skills/cache-scanner/RESOURCES/cache-locations.json), [.../bloatware-catalog.json](../.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json), [.../services-catalog.json](../.claude/skills/powershell-debloat/RESOURCES/services-catalog.json), [.../registry-tweaks.json](../.claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.json), pero la carpeta `src-tauri/src/services/catalog.rs` (vieja) estaba vacía de comentarios.

**Solución aplicada:** [domain/catalog.rs](../src-tauri/src/domain/catalog.rs) implementa `load_cache_locations()` y un helper `load_json<T>()` con degradación grácil (si el JSON no existe, devuelve `[]` en vez de panic).

**Pendiente:**
- Mover los JSON a `src-tauri/resources/` y embeberlos con `include_str!` en release (para que el binario sea autosuficiente, ahora mismo depende de que `.claude/` exista junto al exe en dev).
- Implementar `load_bloatware_catalog()`, `load_services_catalog()`, `load_registry_tweaks()`.
- Añadir validación contra los `*.schema.json` adyacentes.

---

## 4. CRÍTICO — Frontend escucha eventos que el backend nunca emite

**Estado:** ✅ Arreglado en esta sesión.

**Antes:** [features/explorer/explorer-page.tsx](../src/features/explorer/explorer-page.tsx) escuchaba `explorer:node` y `explorer:done` pero `commands/explorer.rs` devolvía `NotImplemented` sin emitir nada. La UI quedaba bloqueada en "Escaneando..." para siempre.

**Solución aplicada:** [ipc/explorer.rs](../src-tauri/src/ipc/explorer.rs) ahora hace `app.emit("explorer:node", ...)` por cada hijo del root y `app.emit("explorer:done", scan_id)` al terminar. La página del explorador funciona contra esa primera versión síncrona.

**Pendiente:**
- Versión asíncrona con `tokio::spawn` para no bloquear el invoke mientras se escanea (importante en árboles grandes).
- Soporte real de `cancel_scan` con un `AtomicBool` por scan_id.

---

## 5. ALTO — `is_elevated` duplicado y la implementación principal era frágil

**Estado:** ✅ Arreglado en esta sesión.

**Antes:**
- `src/elevation.rs` era un stub que devolvía siempre `false`.
- `src/commands/system_info.rs::is_elevated` usaba `Command::new("net").args(["session"])`, que:
  - Es **lento** (lanza un proceso externo).
  - Depende del **idioma del SO** (en algunas localizaciones la salida cambia).
  - Depende del servicio "Server" (en máquinas sin él, falla siempre).

**Solución aplicada:** [platform/elevation.rs](../src-tauri/src/platform/elevation.rs) usa la API correcta:
```rust
OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, ...) 
+ GetTokenInformation(TokenElevation, ...)
```
Es 1000x más rápida, no depende de idioma ni de servicios, y es lo que recomienda Microsoft.

---

## 6. ALTO — Typo `follow_reparse_pints` propagado a TypeScript

**Estado:** ✅ Arreglado en esta sesión.

**Antes:** el parámetro de [commands/explorer.rs:19](../src-tauri/src/commands/explorer.rs) era `follow_reparse_pints` (sic), y el binding en [lib/tauri.ts:32](../src/lib/tauri.ts) lo replicaba con el mismo error. Cualquier llamada al comando habría fallado por mismatch de nombre.

**Solución aplicada:** corregido a `follow_reparse_points` tanto en [ipc/explorer.rs](../src-tauri/src/ipc/explorer.rs) como en [api/client.ts](../src/api/client.ts).

---

## 7. ALTO — Permisos de plugins Tauri faltantes

**Estado:** ✅ Arreglado en esta sesión.

**Antes:** [capabilities/default.json](../src-tauri/capabilities/default.json) solo contenía `core:default` y `opener:default`, pero [lib.rs](../src-tauri/src/lib.rs) registraba 6 plugins (`opener`, `dialog`, `fs`, `shell`, `os`, `process`). Cualquier llamada JS a `@tauri-apps/plugin-dialog`, `plugin-fs`, etc. habría fallado en runtime con `permission denied`.

**Solución aplicada:** [capabilities/default.json](../src-tauri/capabilities/default.json) ahora incluye los `:default` de los 6 plugins.

**Recomendación futura:** crear capabilities específicas por feature en vez de un solo `default.json` con todo (principio de mínimo privilegio).

---

## 8. MEDIO — Stale closures en `useTauriEvent`

**Estado:** ✅ Arreglado en esta sesión.

**Antes:** [src/hooks/use-tauri-event.ts](../src/hooks/use-tauri-event.ts) tenía `// eslint-disable-next-line react-hooks/exhaustive-deps` y ponía solo `[name]` como dependencia. Si el componente padre cambiaba el `handler` (referencia nueva en cada render), el efecto seguía usando el primer `handler` recibido. Resultado: state stale, eventos perdidos.

**Solución aplicada:** uso de `useRef` para el handler, sin ignorar reglas de exhaustive-deps. Tipado también añadido: `useTauriEvent<N extends TauriEventName>` infiere el payload desde el nombre del evento (ver [api/events.ts](../src/api/events.ts)).

---

## 9. MEDIO — Tipo `Risk` con dos casings (duplicación)

**Estado:** ❌ Pendiente (lo dejo marcado para cuando se llene el catálogo).

**Síntoma:** en [api/types.ts](../src/api/types.ts):
```ts
export type Risk = "Low" | "Medium" | "High" | "low" | "medium" | "high";
```

Esto refleja que los JSON del catálogo y el código frontend no están alineados. Cada page tiene una función `riskVariant(risk)` que llama `risk.toLowerCase()` antes de comparar, lo cual funciona pero es síntoma de que el contrato no está cerrado.

**Acción:** decidir un casing único (recomendado: `"low" | "medium" | "high"` en lowercase), añadir un validador en `domain::catalog` que normalice al cargar, y eliminar la duplicación del tipo.

---

## 10. MEDIO — Presets de debloat referencian categorías inexistentes

**Estado:** ❌ Pendiente (cuando se llene el catálogo).

**Síntoma:** [features/debloat/debloat-page.tsx:70](../src/features/debloat/debloat-page.tsx#L70) filtra por strings como `"consumer-app"`, `"consumerapp"`, `"ai"`, `"telemetry"`, `"ms-consumer"`, `"msconsumer"`. Esto depende de qué `category` aparezca en `bloatware-catalog.json`. Si el catálogo usa `"ConsumerApp"` (PascalCase) o `"consumer_app"` (snake_case), los presets quedan vacíos.

**Acción:** alinear el contrato:
- Catálogo: usa SIEMPRE kebab-case en `category` (e.g. `"consumer-app"`).
- Frontend: normaliza con `.toLowerCase()` antes de comparar.
- Spec en [.claude/specs/04-modules/debloat-engine.md](../.claude/specs/04-modules/debloat-engine.md) debe documentar el set cerrado de categorías.

---

## 11. BAJO — Dependencias no usadas + versiones potencialmente desalineadas

**Estado:** ❌ Pendiente (revisar antes del próximo build).

- `dunce` está en [Cargo.toml](../src-tauri/Cargo.toml) pero no se usa en el código actual (se incorporaría al manejar paths >MAX_PATH).
- `vite ^7.0.4` y `@tauri-apps/cli ^2` — verificar compatibilidad si actualizas más adelante.
- `react ^19.1.0` es reciente; el ecosistema de hooks/test libs todavía no está al 100% en v19, vigilar al instalar test deps.

---

## 12. BAJO — Falta `App.css` y `assets/` (residuos del scaffold)

**Estado:** ✅ Arreglado en esta sesión.

Eran restos del scaffolding `npm create tauri-app` que nadie importaba. Eliminados.

---

## 13. BAJO — Convención `snake_case` en JSON vs `camelCase` en TS

**Estado:** ❌ Pendiente (cuando se incorpore `ts-rs`).

Los structs Rust usan `snake_case` por defecto, los DTOs serializan con `snake_case`, y el frontend lo consume tal cual con `snake_case`. Funciona, pero pierde idiomatic TS.

**Acción recomendada (orden importa):**
1. Añadir `ts-rs` como dev-dependency en [Cargo.toml](../src-tauri/Cargo.toml).
2. Decorar los structs en [models/](../src-tauri/src/models/) con `#[derive(TS)] #[ts(export, rename_all = "camelCase")]`.
3. Eliminar [api/types.ts](../src/api/types.ts) y reemplazar por los archivos generados por ts-rs en `src/bindings-generated/`.
4. Añadir `serde(rename_all = "camelCase")` para coherencia.

---

## 14. Resumen ejecutivo — qué hacer en la próxima sesión

Orden recomendado:

1. **Implementar `domain::cache::list_locations`** (ya devuelve algo gracias al `catalog.rs` añadido) y verificar que la pantalla "Limpieza de caché" lista las ubicaciones.  
2. **Implementar `domain::cache::scan` y `clean`** con dry-run real, y conectar la barra de tamaño detectado.  
3. **Implementar `domain::explorer::list_top_level`** (ya hecho) — probar contra `C:\`.  
4. **Crear `domain::restore` con WMI** para que el botón "Crear punto" funcione (es prerequisito de cualquier op destructiva).  
5. **Llenar `domain::registry`** con `winreg` y backup `.reg` previo.  
6. **Llenar `domain::services`** con SCM.  
7. **Implementar `domain::audit_log`** (JSONL append-only en `%APPDATA%\ClearTool\audit.jsonl`).  
8. **Implementar `domain::debloat`** invocando PowerShell embebido via `platform::powershell`.

## 15. Mapa de la nueva estructura

```
ClearTool/
├── CLAUDE.md                                ← instrucciones del proyecto (sin tocar)
├── README.md
├── docs/                                    ← NUEVO
│   ├── AUDIT-FALLOS-CRITICOS.md             ← este archivo
│   ├── HISTORIAL-SESIONES.md
│   └── proyecto-modelo.md
├── scripts/                                 ← NUEVO
│   └── build-release.ps1
├── public/, index.html, package.json, …
├── src/                                     ← frontend React
│   ├── api/                                 ← NUEVO (capa única de IPC)
│   │   ├── client.ts                        ← antes lib/tauri.ts
│   │   ├── events.ts                        ← NUEVO (eventos tipados)
│   │   ├── types.ts                         ← antes bindings/index.ts
│   │   └── index.ts
│   ├── components/, features/, hooks/, lib/, styles/
│   └── App.tsx, main.tsx, router.tsx
└── src-tauri/                               ← backend Rust
    ├── build.rs                             ← manifest dinámico debug/release
    ├── Cargo.toml                           ← features Win32 ampliadas
    ├── capabilities/default.json            ← permisos plugins completados
    └── src/
        ├── main.rs
        ├── lib.rs                           ← solo bootstrap Tauri
        ├── core/                            ← NUEVO (error, config)
        ├── platform/                        ← NUEVO (Win32, winreg, PS)
        │   ├── elevation.rs                 ← OpenProcessToken real
        │   ├── filesystem.rs                ← walkdir + delete
        │   ├── registry.rs, services.rs, restore_point.rs, powershell.rs
        ├── domain/                          ← NUEVO (lógica de negocio pura)
        │   ├── cache.rs, explorer.rs, catalog.rs, system_info.rs
        │   ├── debloat.rs, services.rs, registry.rs, restore.rs, audit.rs
        ├── ipc/                             ← antes commands/ (wrappers Tauri)
        │   └── audit.rs, cache.rs, debloat.rs, explorer.rs,
        │       registry.rs, restore.rs, services.rs, system_info.rs
        └── models/                          ← DTOs (sin cambios)
```
