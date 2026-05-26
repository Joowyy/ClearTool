# 08 — Auditoría de Seguridad final (GATE pre-release)

> **Posición:** 8/14. **Bloquea el merge a `main` y la generación del instalador.**
> **Dependencias:** archivos 00-07 cerrados.
> **Output:** checklist auditado, parches críticos aplicados, veredicto documentado del agente `security-auditor`.

---

## 1. Resumen ejecutivo

ClearTool toca **registro de sistema**, **servicios**, **borrado masivo de archivos**, **PowerShell elevado** y **System Restore**. Cualquier bug aquí no es un crash — es un sistema operativo roto en producción para un usuario real.

Este archivo es **un gate**: la IA debe ir item por item, marcar OK / FAIL, y aplicar parches antes de pasar a tests. El veredicto final lo da el subagente `security-auditor` (definido en `.claude/agents/`).

---

## 2. Filosofía de la auditoría

Cinco principios contra los que medir cada hallazgo:

1. **Defense in depth.** Allowlist en catálogo + validador regex + chequeo runtime. Triple capa, no doble.
2. **Fail closed.** Si la validación es ambigua, abortar. Nunca asumir "probablemente OK".
3. **Principio de mínimo privilegio.** Pedir admin solo cuando la operación lo necesita.
4. **Auditable.** Cada operación destructiva queda registrada en audit log con detalle.
5. **Reversible.** Cada operación destructiva tiene **al menos uno** de: restore point, audit reverse recipe, o documentación clara de "irreversible".

---

## 3. Checklist por categoría

### 3.1 Validación de input (entrada de datos)

| # | Item | Estado |
|---|---|---|
| 3.1.1 | Todo `id` que llega vía IPC se valida contra el catálogo correspondiente (`is_*_allowed`) antes de actuar | ⏳ |
| 3.1.2 | `PackageFamilyName` se valida con regex `^[A-Za-z0-9_.-]+_[A-Za-z0-9]{13}$` antes de pasar a PowerShell | ⏳ |
| 3.1.3 | `ProvisionedName` se valida con regex `^[A-Za-z0-9_.]+$` | ⏳ |
| 3.1.4 | `sequence_number` de restore point se valida que es u32 antes de inyectar en script PS | ⏳ |
| 3.1.5 | `service_name` se valida contra el SCM, no como string crudo | ⏳ |
| 3.1.6 | `path` del scan de explorer se canonicaliza y se verifica contra reparse-point loops | ⏳ |
| 3.1.7 | Ningún IPC acepta `(hive, key, name)` directos del frontend para operaciones de write | ⏳ |
| 3.1.8 | Cualquier path que se borre debe estar bajo una `cache-locations` entry expandida | ⏳ |
| 3.1.9 | `run_id` del audit revert no se confía: se busca en el archivo, no se ejecuta el JSON entrante | ⏳ |

### 3.2 PowerShell embebido

| # | Item | Estado |
|---|---|---|
| 3.2.1 | Todos los scripts PS son `include_str!` de archivos `.ps1` en `src-tauri/src/platform/ps-scripts/` | ⏳ |
| 3.2.2 | Ningún `run_script` se invoca con string construido por concatenación de input usuario | ⏳ |
| 3.2.3 | Argumentos dinámicos pasan por **env vars** (`$env:CT_*`), nunca como `-Command "& {... $arg ...}"` | ⏳ |
| 3.2.4 | Helper `powershell::run_script` solo acepta `&'static str` para impedir improvisación | ⏳ |
| 3.2.5 | `CREATE_NO_WINDOW` (0x0800_0000) está activo en TODA invocación de PowerShell | ⏳ |
| 3.2.6 | `-NoProfile -NonInteractive -ExecutionPolicy Bypass` siempre presentes | ⏳ |
| 3.2.7 | Output PS se parsea como JSON, no como text scraping | ⏳ |
| 3.2.8 | Timeout configurado (default 20s) en todas las invocaciones | ⏳ |

### 3.3 Registro de Windows

| # | Item | Estado |
|---|---|---|
| 3.3.1 | `hkey_for` rechaza cualquier hive fuera de `{HKLM, HKCU, HKU, HKCR}` | ⏳ |
| 3.3.2 | `is_registry_key_allowed` verifica que la key esté bajo un prefix definido en el catálogo | ⏳ |
| 3.3.3 | `write_value` NO crea keys arbitrarias — solo dentro de paths del catálogo | ⏳ |
| 3.3.4 | `delete_value` también valida contra allowlist | ⏳ |
| 3.3.5 | Cambios en `SystemRestorePointCreationFrequency` se restauran a su valor previo (test verifica) | ⏳ |
| 3.3.6 | Escrituras en HKLM verifican elevación antes de intentar | ⏳ |
| 3.3.7 | `previous_value` capturado y guardado en audit ANTES de modificar | ⏳ |

### 3.4 Servicios

| # | Item | Estado |
|---|---|---|
| 3.4.1 | `set_start_type` valida `service_name` contra catálogo | ⏳ |
| 3.4.2 | Servicios protegidos del SO (`Spooler`, `WSearch` ALTOS) marcados `risk: high` en catálogo | ⏳ |
| 3.4.3 | Apertura de `OpenServiceW` cierra siempre el handle con `CloseServiceHandle` (incluso en error) | ⏳ |
| 3.4.4 | `ControlService(STOP)` no espera infinitamente — timeout 30s | ⏳ |
| 3.4.5 | Cambio de start_type no afecta el estado runtime — `stop_now` es flag separado y opcional | ⏳ |
| 3.4.6 | Dependent services se listan en UI antes de stop | ⏳ |
| 3.4.7 | Servicios marcados `protected` en el report no abortan batch | ⏳ |

### 3.5 Borrado de archivos

| # | Item | Estado |
|---|---|---|
| 3.5.1 | `delete_recursive_robust` NUNCA recibe un path arbitrario — solo paths del catálogo expandidos | ⏳ |
| 3.5.2 | Test de path traversal: `%APPDATA%/../../Windows` se rechaza (no se acepta el catálogo) | ⏳ |
| 3.5.3 | Filtros del catálogo se aplican (no se borran archivos excluidos por extensión) | ⏳ |
| 3.5.4 | Reparse points no se siguen por default (evita loops + cross-volume) | ⏳ |
| 3.5.5 | Archivos en uso se marcan para reboot (`MoveFileEx`), no abortan operación | ⏳ |
| 3.5.6 | Cancelación del scan/clean detiene el walk a tiempo razonable (<2s) | ⏳ |

### 3.6 System Restore

| # | Item | Estado |
|---|---|---|
| 3.6.1 | `SRSetRestorePointW` se llama con `BEGIN_NESTED_SYSTEM_CHANGE` y se cierra con `END_NESTED` | ⏳ |
| 3.6.2 | Bypass del throttling restaura el valor previo de `SystemRestorePointCreationFrequency` | ⏳ |
| 3.6.3 | `restore_to_point` valida que el `sequence_number` exista antes de invocar | ⏳ |
| 3.6.4 | UI muestra disclaimer claro: "reinicia el sistema, irreversible una vez iniciada" | ⏳ |
| 3.6.5 | Si System Protection está OFF, no se intenta crear punto silenciosamente — se reporta y abort opcional | ⏳ |

### 3.7 Audit log

| # | Item | Estado |
|---|---|---|
| 3.7.1 | JSONL escrito con `OpenOptions::new().append(true)`, retry 3x si lock | ⏳ |
| 3.7.2 | Línea corrupta no rompe lectura del resto | ⏳ |
| 3.7.3 | `revert_entry` rechaza entries con `dry_run: true` | ⏳ |
| 3.7.4 | `ReverseRecipe::Noop` se rechaza para revert (con error claro) | ⏳ |
| 3.7.5 | El audit log no se borra desde la app salvo en "Modo diagnóstico" + doble confirmación | ⏳ |
| 3.7.6 | Path del audit log: `%APPDATA%\ClearTool\audit.jsonl` — no en `%TEMP%`, no en `C:\Logs` | ⏳ |
| 3.7.7 | Rotación a 10 MB se ejecuta atómicamente (rename, no truncate) | ⏳ |

### 3.8 Elevación

| # | Item | Estado |
|---|---|---|
| 3.8.1 | `is_elevated` usa `OpenProcessToken` + `GetTokenInformation`, no `net session` | ⏳ |
| 3.8.2 | Manifest debug: `asInvoker`. Release: `requireAdministrator`. | ⏳ |
| 3.8.3 | `relaunch_as_admin_if_needed` se invoca al startup en release | ⏳ |
| 3.8.4 | Operaciones que requieren admin verifican `is_elevated` antes de intentar y devuelven error claro | ⏳ |
| 3.8.5 | UI muestra banner "Modo limitado" si no elevado en release | ⏳ |

### 3.9 IPC y permisos Tauri

| # | Item | Estado |
|---|---|---|
| 3.9.1 | `capabilities/default.json` lista solo plugins realmente usados | ⏳ |
| 3.9.2 | No hay `tauri-plugin-shell` con `execute` permitido — el shell solo se usa para `open(url)` | ⏳ |
| 3.9.3 | `tauri-plugin-fs` con scope mínimo (no `**/*`) | ⏳ |
| 3.9.4 | Eventos solo escuchan en frontend si están emitidos por el backend (auditar `listen` calls) | ⏳ |
| 3.9.5 | Comandos async no bloquean el main thread con operaciones síncronas largas | ⏳ |

### 3.10 Telemetría y red

| # | Item | Estado |
|---|---|---|
| 3.10.1 | No hay `reqwest::Client::new()` que llame a endpoints fuera de `updates.cleartool.app/manifest.json` | ⏳ |
| 3.10.2 | Update check es opt-in (setting puede desactivarlo) | ⏳ |
| 3.10.3 | User-Agent del update check no incluye datos personales (solo `ClearTool/1.0.0`) | ⏳ |
| 3.10.4 | DNS resolution: no hay calls a Cloudflare DoH / Google DoH desde la app | ⏳ |
| 3.10.5 | No hay analytics, posthog, sentry, etc. en frontend | ⏳ |

### 3.11 Concurrencia y handles

| # | Item | Estado |
|---|---|---|
| 3.11.1 | Cancellation tokens del explorer scan se honran con check cada ~256 entries | ⏳ |
| 3.11.2 | SCM handles cerrados en TODOS los paths de error (no leaks) | ⏳ |
| 3.11.3 | Mutex/RwLock no se mantienen across await — uso de scope explícito | ⏳ |
| 3.11.4 | `std::process::Command` con stdout pipe se consume — no se queda pending | ⏳ |

### 3.12 Logging y output

| # | Item | Estado |
|---|---|---|
| 3.12.1 | Logs no contienen API keys, tokens, paths personales completos sin redactar | ⏳ |
| 3.12.2 | `log::debug!` no expone payloads de eventos completos en producción | ⏳ |
| 3.12.3 | `panic!` solo en `validate_all_at_startup` (catalog malformado = bug de build) | ⏳ |
| 3.12.4 | Mensajes de error visibles al usuario no exponen paths internos del SO | ⏳ |

---

## 4. Procedimiento de auditoría

### 4.1 Workflow recomendado

1. Crear branch `chore/security-audit`.
2. La IA recorre el checklist item por item, marcando ✓/✗ con commit por sección.
3. Para cada ✗, crear issue en `docs/PLAN-FINAL/issues-seguridad.md` con:
   - Item #
   - Archivo + línea
   - Severidad: Crítica / Alta / Media / Baja
   - Fix recomendado
4. Aplicar fixes en commits separados.
5. Re-correr checklist. Solo se acepta `✗` si tiene justificación explícita (issue documentado + decisión consciente).
6. Invocar `security-auditor` subagent para revisión cruzada del PR.

### 4.2 Comando de auditoría sugerido

```bash
# Buscar strings sospechosas
grep -rn "powershell.exe" src-tauri/src/ --include="*.rs"
grep -rn "unwrap" src-tauri/src/ --include="*.rs" | grep -v "test"
grep -rn "Command::new" src-tauri/src/ --include="*.rs"
grep -rn "include_str" src-tauri/src/ --include="*.rs"
grep -rn "delete" src-tauri/src/ --include="*.rs"

# Verificar que no hay endpoints sospechosos
grep -rn "https://" src-tauri/src/ --include="*.rs"
grep -rn "http://" src-tauri/src/ --include="*.rs"
```

### 4.3 Subagent invocation

> Cuando el checklist esté ≥95% en verde, invocar al subagente:
>
> ```
> Agent({
>   subagent_type: "security-auditor",
>   description: "Audit cruzado pre-release ClearTool 1.0",
>   prompt: "Auditá la rama refactor/ui-cosmic-redesign contra docs/PLAN-FINAL/08-SEGURIDAD-FINAL.md. Reportá hallazgos por severidad. Tu veredicto bloquea el merge."
> })
> ```

---

## 5. Test de pentesting básico

### 5.1 Casos de prueba adversarial

| # | Caso | Esperado |
|---|---|---|
| 5.1.1 | IPC `apply_registry_tweak({"id": "../../../../etc/passwd"})` | Error: tweak no en allowlist |
| 5.1.2 | IPC `set_service_state({"name": "RpcSs", "start_type": "Disabled"})` (RpcSs es crítico, NO está en catálogo) | Error: servicio no en catálogo |
| 5.1.3 | IPC `remove_bloatware({"entryIds": ["foo'; Remove-Item C:\\ -Recurse;#"]})` | Error: id no en catálogo |
| 5.1.4 | IPC `clean_cache_locations({"ids": ["windows-system32"]})` (no existe en catálogo) | Status `not-in-catalog` |
| 5.1.5 | Crear un `audit.jsonl` con entry forjada apuntando a `HKLM\SYSTEM` arbitrario y llamar `revert_entry` | Error: la key no está en allowlist del catálogo de tweaks |
| 5.1.6 | Llamar `restore_to_point(sequence_number=999999)` | Error: punto no existe |
| 5.1.7 | Variable de entorno hostil `%APPDATA%=C:\Windows` antes de lanzar | Las paths expanden a C:\Windows, pero todas las operaciones validan contra catálogo — abortan |
| 5.1.8 | Frontend envía JSON con campos extra `__proto__` | Serde ignora campos no declarados |

### 5.2 Cómo ejecutar (test manual)

Cada caso = mini script en `src-tauri/tests/adversarial.rs`. Ejecutar con `cargo test --test adversarial`.

```rust
// src-tauri/tests/adversarial.rs

use cleartool::ipc;

#[tokio::test]
async fn tweak_id_path_traversal_rechazado() {
    let r = cleartool::domain::registry::apply(&cleartool::models::registry::ApplyTweakInput {
        id: "../../../etc/passwd".to_string(),
        enable: true,
        dry_run: true,
    });
    assert!(r.is_err());
}

#[tokio::test]
async fn servicio_critico_no_en_catalogo_rechazado() {
    let r = cleartool::domain::services::set_state(&cleartool::models::service::SetServiceStateInput {
        name: "RpcSs".into(),
        start_type: "Disabled".into(),
        stop_now: false,
        dry_run: true,
    });
    assert!(r.is_err(), "RpcSs no debe ser tocable — no en catálogo");
}

#[tokio::test]
async fn bloatware_id_inyeccion_rechazado() {
    let r = cleartool::domain::debloat::remove(
        &cleartool::models::debloat::RemoveBloatwareInput {
            entry_ids: vec!["foo'; Remove-Item C:\\ -Recurse;#".into()],
            dry_run: true,
            create_restore_point: false,
            apply_policies: false,
            disable_services: false,
        },
        |_, _, _, _| {},
    ).unwrap();
    assert_eq!(r.skipped, 1, "id maliciosa debe skipearse, no procesarse");
}
```

---

## 6. Issues conocidos y aceptados

> Los issues que **no se resuelven** en v1.0 pero se aceptan documentadamente.

| # | Issue | Justificación | Plan v1.1 |
|---|---|---|---|
| OPEN-1 | SmartScreen mostrará warning al primer lanzamiento (sin code-signing) | Decisión: lanzar sin firma. Documentado en [10-DISTRIBUCION](10-DISTRIBUCION.md) | Comprar certificado OV cuando haya >5k descargas |
| OPEN-2 | `AutomaticDelayed` no se detecta correctamente (reportado como `Automatic`) | Requiere query `SERVICE_CONFIG_DELAYED_AUTO_START_INFO` separada | Implementar en v1.1 |
| OPEN-3 | No hay detección de procesos que bloquean archivos del cache | Restart Manager API es complejo | v1.1 con `handle.exe` opcional |
| OPEN-4 | Compresión gzip de audit log archivado no implementada | Tamaño manejable sin compresión a corto plazo | v1.1 |
| OPEN-5 | `.reg` backup automático antes de tweaks no se genera | Audit log + restore point son suficientes para v1.0 | v1.1 si feedback de usuarios lo pide |

---

## 7. Veredicto del agente `security-auditor`

> Esta sección la rellena el subagente, no la IA principal.

```yaml
veredicto:
  fecha: ""
  release_blocked: false
  hallazgos_criticos: []
  hallazgos_altos: []
  hallazgos_medios: []
  hallazgos_bajos: []
  recomendaciones_v1_1: []
  firma: security-auditor@cleartool
```

---

## 8. Definition of Done

- [ ] Los 87 items del checklist marcados ✓.
- [ ] `docs/PLAN-FINAL/issues-seguridad.md` creado con issues abiertos resueltos.
- [ ] Test suite `adversarial.rs` pasa 8/8.
- [ ] Comando `grep -rn "Command::new(\"powershell\")" src-tauri/src/` solo devuelve `platform/powershell.rs` y `platform/debloat.rs` (los wrappers controlados).
- [ ] Comando `grep -rn "https://" src-tauri/src/` solo devuelve `updates.cleartool.app`.
- [ ] Subagente `security-auditor` invocado y veredicto = `release_blocked: false`.
- [ ] Commit `chore(security): cierre auditoría pre-release ClearTool 1.0`.

---

## 9. Próximo archivo

→ [09-TESTS.md](09-TESTS.md) — solo se pasa cuando este archivo está completamente en verde. Tests + QA en VM Windows 11 limpia.
