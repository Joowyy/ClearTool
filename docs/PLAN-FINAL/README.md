# PLAN-FINAL — Hoja de ruta para llevar ClearTool a versión pública 1.0

> **Propósito.** Este directorio contiene el plan operativo completo, fragmentado en bloques atómicos, para cerrar ClearTool en una versión publicable. Está escrito para que una IA (Claude Code o similar) pueda recorrerlo en orden, ejecutar cada paso, y dejar el repositorio en estado listo para release.
>
> **Generado:** 2026-05-21
> **Rama base esperada:** `refactor/ui-cosmic-redesign`
> **Target release:** ClearTool 1.0.0 — Open-source MIT, ES/EN, sin telemetría, firma de código diferida.

---

## 0. Cómo leer este plan

Cada archivo de este directorio sigue la **misma estructura interna** para que la IA pueda procesarlos de forma homogénea:

```
1. Resumen ejecutivo
2. Diagnóstico (qué hay hoy, qué falta, qué está roto)
3. Investigación (mercado / técnica) — cuando aporta
4. Decisiones arquitectónicas
5. Modelo de datos / contratos
6. Plan de UX (mockups ASCII cuando aplique)
7. Plan de implementación por fases
   - Fase N
     - Paso N.x — Título descriptivo
       - Archivo concreto
       - Snippet de código si procede
       - Criterio de aceptación
8. Tests
9. Riesgos y mitigaciones
10. Definition of Done (DoD)
```

**Reglas de oro al ejecutar:**

1. **Una fase a la vez.** Cada fase deja la app funcionando. No saltar.
2. **Cada paso es ejecutable de forma aislada.** Si la IA queda a medias, otra IA debe poder continuar leyendo solo el paso siguiente.
3. **Antes de marcar una fase como completa**, verificar el DoD del archivo correspondiente.
4. **Toda operación destructiva pasa por [01-RESTORE-POINTS](01-RESTORE-POINTS.md) + [02-AUDIT-LOG](02-AUDIT-LOG.md)**. No hay excepciones.
5. **Toda lectura/escritura del catálogo pasa por [00-CATALOGOS](00-CATALOGOS.md)**. No hay paths hard-coded.

---

## 1. Estado del proyecto (snapshot al 2026-05-21)

### 1.1 Módulos del backend (`src-tauri/src/domain/`)

| Módulo | Estado | Plan asociado |
|---|---|---|
| `cache.rs` | ✅ Funcional (escaneo + clean) | [06-CACHE-FINAL](06-CACHE-FINAL.md) — pulido + filtros |
| `catalog.rs` | 🟡 Solo cache-locations | [00-CATALOGOS](00-CATALOGOS.md) |
| `explorer.rs` | 🟡 list_top_level OK, árbol roto | [../PLAN-EXPLORER-REDISENIO.md](../PLAN-EXPLORER-REDISENIO.md) (ya existe) |
| `system_info.rs` | ✅ Funcional | — |
| `telemetry.rs` | ✅ Funcional | — |
| `debloat.rs` | ❌ `NotImplemented` | [05-DEBLOAT](05-DEBLOAT.md) |
| `services.rs` | ❌ `NotImplemented` | [04-SERVICES](04-SERVICES.md) |
| `registry.rs` | ❌ `NotImplemented` | [03-REGISTRY](03-REGISTRY.md) |
| `restore.rs` | ❌ `NotImplemented` | [01-RESTORE-POINTS](01-RESTORE-POINTS.md) |
| `audit.rs` | ❌ `NotImplemented` | [02-AUDIT-LOG](02-AUDIT-LOG.md) |

### 1.2 Módulos del frontend (`src/features/`)

| Pantalla | Estado | Plan asociado |
|---|---|---|
| `home` (Dashboard 3D) | ✅ Funcional | [../PLAN-DASHBOARD-3D.md](../PLAN-DASHBOARD-3D.md) |
| `explorer` | 🟡 Funcional pero rota la expansión | [../PLAN-EXPLORER-REDISENIO.md](../PLAN-EXPLORER-REDISENIO.md) |
| `cache-cleaner` | 🟡 Listado + scan/clean OK; falta filtros y presets | [06-CACHE-FINAL](06-CACHE-FINAL.md) |
| `debloat` | 🟡 UI esquemática contra stub | [05-DEBLOAT](05-DEBLOAT.md) |
| `services` | 🟡 UI esquemática contra stub | [04-SERVICES](04-SERVICES.md) |
| `registry-tweaks` | 🟡 UI esquemática contra stub | [03-REGISTRY](03-REGISTRY.md) |
| `restore-points` | 🟡 UI esquemática contra stub | [01-RESTORE-POINTS](01-RESTORE-POINTS.md) |
| `settings` | ❌ Solo theme picker | [07-SETTINGS](07-SETTINGS.md) |
| `audit` (UI) | ❌ No existe pantalla | [02-AUDIT-LOG](02-AUDIT-LOG.md) |
| `onboarding` (first-run) | ❌ No existe | [12-I18N-ONBOARDING](12-I18N-ONBOARDING.md) |

### 1.3 Catálogos JSON (allowlists)

| Catálogo | Ubicación | Estado |
|---|---|---|
| `cache-locations.json` | `.claude/skills/cache-scanner/RESOURCES/` | ✅ Curado |
| `bloatware-catalog.json` | `.claude/skills/powershell-debloat/RESOURCES/` | 🟡 Inicial, ampliar |
| `services-catalog.json` | (pendiente crear) | ❌ |
| `registry-tweaks.json` | `.claude/skills/windows-registry-ops/RESOURCES/` | ❌ Vacío |
| Schemas `*.schema.json` | adyacentes a cada catálogo | ❌ Pendientes |

### 1.4 Cross-cutting

| Área | Estado | Plan asociado |
|---|---|---|
| Tests unitarios Rust | ❌ Inexistentes | [09-TESTS](09-TESTS.md) |
| Tests frontend (Vitest) | ❌ Inexistentes | [09-TESTS](09-TESTS.md) |
| Validación en VM limpia | ❌ Sin procedimiento | [09-TESTS](09-TESTS.md) |
| Instalador NSIS / MSI | ❌ Solo el debug build | [10-DISTRIBUCION](10-DISTRIBUCION.md) |
| Firma de código | ❌ No aplica para v1.0 (warning SmartScreen aceptable) | [10-DISTRIBUCION](10-DISTRIBUCION.md) |
| Auto-update (Tauri Updater) | ❌ Inexistente | [11-AUTO-UPDATE](11-AUTO-UPDATE.md) |
| Internacionalización (ES/EN) | ❌ Strings hardcoded en español | [12-I18N-ONBOARDING](12-I18N-ONBOARDING.md) |
| Onboarding / first-run | ❌ Inexistente | [12-I18N-ONBOARDING](12-I18N-ONBOARDING.md) |
| Auditoría de seguridad cruzada | ❌ Pendiente | [08-SEGURIDAD-FINAL](08-SEGURIDAD-FINAL.md) |
| Checklist de lanzamiento | ❌ Inexistente | [13-LANZAMIENTO](13-LANZAMIENTO.md) |

---

## 2. Mapa de dependencias entre archivos

Importante: el **orden de los archivos no es arbitrario**. Hay dependencias duras: si saltas a Registry sin tener Restore Points + Audit Log, vas a estar construyendo sobre arena.

```
                ┌──────────────────────────────┐
                │  00-CATALOGOS (schemas + JSON)│  ← Base de TODOS los módulos destructivos
                └──────────────┬───────────────┘
                               │
        ┌──────────────────────┴──────────────────────┐
        │                                              │
┌───────▼──────────┐                          ┌────────▼─────────┐
│ 01-RESTORE-POINTS│ ────────┐                │   02-AUDIT-LOG    │
└──────────────────┘         │                └────────┬─────────┘
                             │                         │
                             └──────────┬──────────────┘
                                        │
        ┌───────────────────────────────┼───────────────────────────────┐
        │                               │                               │
┌───────▼──────────┐          ┌─────────▼────────┐           ┌─────────▼────────┐
│   03-REGISTRY    │          │   04-SERVICES    │           │    05-DEBLOAT    │
└──────────────────┘          └──────────────────┘           └──────────────────┘
        │                               │                               │
        └───────────────────────────────┼───────────────────────────────┘
                                        │
                              ┌─────────▼────────┐
                              │  06-CACHE-FINAL  │ (pulido del único módulo ya en marcha)
                              └─────────┬────────┘
                                        │
                              ┌─────────▼────────┐
                              │   07-SETTINGS    │ (necesita audit + restore para el panel "log")
                              └─────────┬────────┘
                                        │
                              ┌─────────▼─────────────┐
                              │ 08-SEGURIDAD-FINAL    │ (auditoría cruzada — bloquea release)
                              └─────────┬─────────────┘
                                        │
                              ┌─────────▼────────┐
                              │    09-TESTS      │
                              └─────────┬────────┘
                                        │
                              ┌─────────▼────────────┐
                              │  10-DISTRIBUCION     │
                              └─────────┬────────────┘
                                        │
                              ┌─────────▼────────────┐
                              │   11-AUTO-UPDATE     │
                              └─────────┬────────────┘
                                        │
                              ┌─────────▼────────────┐
                              │ 12-I18N-ONBOARDING   │
                              └─────────┬────────────┘
                                        │
                              ┌─────────▼────────────┐
                              │   13-LANZAMIENTO     │
                              └──────────────────────┘
```

**Regla de oro de dependencias:**

- `00` antes que cualquier módulo destructivo.
- `01` y `02` antes que `03`, `04`, `05`, `06`.
- `08` (seguridad) **es un gate**: no se pasa a `09` sin auditoría aprobada.
- `10`, `11`, `12`, `13` son secuenciales — instalador antes que updater, updater antes que onboarding, onboarding antes que lanzamiento.

---

## 3. Orden de ejecución recomendado (modo "una sesión de IA por archivo")

Si tu sesión de Claude puede atacar **un archivo por turno** sin perder contexto, este es el orden óptimo:

| Sesión | Archivo | Output esperado |
|---|---|---|
| 1 | [00-CATALOGOS](00-CATALOGOS.md) | 3 JSON catálogos + 3 schemas + helpers Rust de validación |
| 2 | [01-RESTORE-POINTS](01-RESTORE-POINTS.md) | `domain::restore` y `platform::restore_point` completos + UI |
| 3 | [02-AUDIT-LOG](02-AUDIT-LOG.md) | `domain::audit` + UI básica + reverse recipes |
| 4 | [03-REGISTRY](03-REGISTRY.md) | `domain::registry` completo + UI con diff viewer |
| 5 | [04-SERVICES](04-SERVICES.md) | `domain::services` (SCM) + UI con presets |
| 6 | [05-DEBLOAT](05-DEBLOAT.md) | `domain::debloat` (Appx + uninstallers) + UI con presets |
| 7 | [06-CACHE-FINAL](06-CACHE-FINAL.md) | Filtros + presets + integración con restore/audit |
| 8 | [07-SETTINGS](07-SETTINGS.md) | Settings persistentes + panel audit log + dry-run global |
| 9 | [08-SEGURIDAD-FINAL](08-SEGURIDAD-FINAL.md) | Auditoría documentada + parches críticos aplicados |
| 10 | [09-TESTS](09-TESTS.md) | Test suite mínima viable + procedimiento QA en VM |
| 11 | [10-DISTRIBUCION](10-DISTRIBUCION.md) | NSIS .exe + MSI funcionales + iconos + metadatos |
| 12 | [11-AUTO-UPDATE](11-AUTO-UPDATE.md) | Tauri Updater + manifest server + canales |
| 13 | [12-I18N-ONBOARDING](12-I18N-ONBOARDING.md) | i18next ES/EN + first-run wizard + accesibilidad |
| 14 | [13-LANZAMIENTO](13-LANZAMIENTO.md) | Checklist + comunicación + post-mortem template |

Estimación bruta: **14 sesiones de 2-3 h de trabajo dirigido**. No es lineal — algunos archivos exigen ejecutar paralelo en VM.

---

## 4. Convenciones aplicables a TODOS los archivos

### 4.1 Idioma y casing

- Comentarios y docs en **español**.
- Identificadores Rust en `snake_case`.
- Identificadores TS en `camelCase`.
- JSON (catálogos) en `camelCase` — los structs Rust llevan `#[serde(rename_all = "camelCase")]`.
- Strings de UI: extraer a `i18n/es.json` e `i18n/en.json` (ver [12-I18N-ONBOARDING](12-I18N-ONBOARDING.md)).

### 4.2 Errores

- Backend Rust: tipos `thiserror` en `core::error::AppError`. Nunca `unwrap()` en lib.
- Frontend TS: `Result<T, E>` o `try { invoke }` con boundary del componente.

### 4.3 Eventos Tauri

- Naming: `<modulo>:<evento>` (kebab-case módulo, kebab-case evento).
- Todo evento que pertenece a una operación lleva `runId`/`scanId` para descartar zombies.

### 4.4 Comandos Tauri

- Async siempre (`#[tauri::command] async fn`).
- Validación de input en la primera línea contra allowlist.
- Dry-run obligatorio en cualquier comando destructivo.

### 4.5 Logs operativos vs Audit log

- **Audit log** (`%APPDATA%\ClearTool\audit.jsonl`): operaciones del usuario, con `reverse_recipe`. JSONL append-only.
- **Operational log** (`%LOCALAPPDATA%\ClearTool\logs\app.log`): trazas técnicas (errores, warnings). Rotativo por tamaño.
- No mezclar los dos.

### 4.6 Telemetría

- **No hay**. Decisión firme. Cero llamadas salientes salvo:
  - Auto-update check (Tauri Updater contra manifest server propio).
  - Apertura de URLs en navegador externo cuando el usuario clickea explícitamente.
- Cualquier excepción se documenta en `08-SEGURIDAD-FINAL.md`.

---

## 5. Estado de finalización del plan

| # | Archivo | Generado | Output esperado |
|---|---|---|---|
| — | README.md | ✅ | Este archivo (índice + dependencias) |
| 00 | [00-CATALOGOS.md](00-CATALOGOS.md) | ✅ | 4 JSON catálogos curados + 4 schemas + helpers Rust |
| 01 | [01-RESTORE-POINTS.md](01-RESTORE-POINTS.md) | ✅ | `SRSetRestorePointW` + WMI + UI + helper `ensure_or_create` |
| 02 | [02-AUDIT-LOG.md](02-AUDIT-LOG.md) | ✅ | JSONL append-only + reverse recipes + UI revert |
| 03 | [03-REGISTRY.md](03-REGISTRY.md) | ✅ | `winreg` + diff viewer + batch con audit |
| 04 | [04-SERVICES.md](04-SERVICES.md) | ✅ | SCM directo + presets + UI virtualizada |
| 05 | [05-DEBLOAT.md](05-DEBLOAT.md) | ✅ | Appx + uninstallers + PS embebido + disclaimers HIGH |
| 06 | [06-CACHE-FINAL.md](06-CACHE-FINAL.md) | ✅ | Filtros + presets + audit + progress streaming |
| 07 | [07-SETTINGS.md](07-SETTINGS.md) | ✅ | Settings persistentes + dry-run global + 5 tabs |
| 08 | [08-SEGURIDAD-FINAL.md](08-SEGURIDAD-FINAL.md) | ✅ | Auditoría cruzada con 87 items + tests adversarial — **GATE** |
| 09 | [09-TESTS.md](09-TESTS.md) | ✅ | Test suite + CI GitHub Actions + QA manual en VM |
| 10 | [10-DISTRIBUCION.md](10-DISTRIBUCION.md) | ✅ | NSIS + MSI + manifest + iconos + estrategia SmartScreen |
| 11 | [11-AUTO-UPDATE.md](11-AUTO-UPDATE.md) | ✅ | Tauri Updater + manifest server + canales stable/beta |
| 12 | [12-I18N-ONBOARDING.md](12-I18N-ONBOARDING.md) | ✅ | i18next ES/EN + first-run wizard + accesibilidad WCAG AA |
| 13 | [13-LANZAMIENTO.md](13-LANZAMIENTO.md) | ✅ | Pre-flight checklist + secuencia D-day + comunicación + post-mortem |

**Plan completo: 14 documentos, ~6000 líneas, plan ejecutable bloque-a-bloque por IA.**

---

## 6. Cómo comunicar el avance

Cada vez que se cierra un archivo:

1. Cambiar el `⏳` por `✅` en la tabla de la sección 5.
2. Añadir una entrada en `docs/HISTORIAL-SESIONES.md` con:
   - Fecha
   - Archivo cerrado
   - Commits asociados
   - Issues / blockers detectados
3. Cuando todos los archivos están `✅`, el repo está en estado RC1 (Release Candidate 1).

---

## 7. Glosario

- **Allowlist:** lista cerrada de IDs/paths/claves que el catálogo permite tocar. Cualquier input fuera → abort.
- **Audit entry:** registro JSONL de una operación, con `reverse_recipe` que documenta cómo deshacerla.
- **Dry-run:** modo de ejecución que reporta qué haría sin tocar nada.
- **Reverse recipe:** instrucción declarativa (no código) de cómo revertir una operación. Ej: `{"kind": "registry", "hive": "HKLM", "key": "...", "value": "...", "previous_data": "0"}`.
- **Restore Point:** snapshot del sistema vía `SRSetRestorePointW`. Reversa "atómica" del SO completo.
- **Preset:** combinación nombrada de selecciones del catálogo (ej. preset "Mínimo" del debloat).
