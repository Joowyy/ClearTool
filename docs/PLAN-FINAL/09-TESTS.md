# 09 — Test suite + QA en VM Windows 11

> **Posición:** 9/14. Solo se entra aquí si [08-SEGURIDAD-FINAL](08-SEGURIDAD-FINAL.md) está en verde.
> **Dependencias:** todos los módulos 00-07 cerrados.
> **Output:** test suite Rust + Vitest mínima viable, procedimiento de QA manual en VM limpia, scripts de validación.

---

## 1. Resumen ejecutivo

ClearTool tiene **cero tests** hoy más allá de los unitarios sueltos descritos en archivos anteriores. Para una herramienta destructiva, eso es un riesgo de release.

Este archivo define el **mínimo viable** para v1.0:

1. **Unit tests Rust** — los helpers críticos (path expansion, filters, regex validators, parsers).
2. **Integration tests Rust** — los flujos completos en una VM virgen.
3. **Vitest frontend** — los hooks y reducers que tocan lógica de presentación (no UI).
4. **Procedimiento QA manual en VM limpia** — checklist humana antes de release.
5. **CI mínima en GitHub Actions** — `cargo test` + `vitest` + `cargo clippy` + `cargo fmt --check`.

No buscamos cobertura 90%. Buscamos cobertura del **camino destructivo** y de los **invariantes de seguridad**.

---

## 2. Diagnóstico

### 2.1 Qué SÍ se puede testear bien

| Tipo | Cobertura realista |
|---|---|
| Helpers puros (regex, expand_path, parse_wmi_datetime, file_passes_filters) | 100% |
| Parsing de catálogos JSON | 100% (5 schemas) |
| Audit log (write/read/rotate/revert dispatch) | 90% |
| Domain layer con mocks del platform | 70% |
| Comandos IPC contra catálogos forjados | 80% |

### 2.2 Qué NO se puede testear automáticamente

| Caso | Por qué | Estrategia |
|---|---|---|
| `SRSetRestorePointW` real | Modifica el SO | Test manual en VM, marcado `#[ignore]` |
| `Remove-AppxPackage` real | Toca el sistema | Test manual en VM |
| `ChangeServiceConfigW` real | Toca SCM | Test manual en VM |
| UAC dialog en release | Requiere interacción humana | Manual |
| SmartScreen warning | Comportamiento del SO en máquina virgen | Manual |
| Cancelación de scan a medio camino | Timing-dependent | Test integration con `ignore` |

### 2.3 Stack de test

| Layer | Herramienta | Razón |
|---|---|---|
| Rust unit | `cargo test` builtin | Cero deps extra |
| Rust integration | `cargo test --test <name>` | Mismo runner |
| Rust mocks (opcional) | `mockall` v0.13 | Solo para domain layer si se quiere desacoplar de platform |
| JS unit | Vitest | Ya en deps (presumido) — si no, añadir |
| JS DOM (opcional) | `@testing-library/react` | Skippeado para v1.0 |
| E2E (opcional) | Playwright + tauri-driver | Skippeado para v1.0 |
| CI | GitHub Actions Windows runner | Tauri necesita Windows nativo |

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| Tests destructivos **siempre** `#[ignore]` por default | Evitar tocar el SO en `cargo test` automático |
| Tests de catálogo (parseo) en CI obligatorio | Si un catálogo está roto, build falla |
| Frontend: tests solo de lógica pura, no de componentes | Hooks + reducers son lo que importa |
| Procedimiento QA manual documentado paso a paso | Es la verdad final antes de release |
| CI corre en `windows-latest` runner | Win32 APIs no compilan en linux runner |
| Coverage report opcional (no bloquea) | Métricas pueden distraer del foco real |

---

## 4. Estructura de tests

```
src-tauri/
  tests/                                ← integration tests
    catalogs.rs                         ← creado en 00-CATALOGOS
    audit.rs                            ← creado en 02-AUDIT-LOG
    registry.rs                         ← creado en 03-REGISTRY
    registry_apply.rs                   ← creado en 03-REGISTRY
    services_platform.rs                ← creado en 04-SERVICES
    cache_filters.rs                    ← creado en 06-CACHE-FINAL
    adversarial.rs                      ← creado en 08-SEGURIDAD-FINAL
    settings_roundtrip.rs               ← creado en 07-SETTINGS
    debloat_validators.rs               ← creado en 05-DEBLOAT
    restore_integration.rs              ← creado en 01-RESTORE-POINTS
    explorer_streaming.rs               ← nuevo (ver 4.1)
  src/
    domain/<modulo>.rs                  ← unit tests inline #[cfg(test)] mod tests
    platform/<modulo>.rs                ← unit tests inline

src/
  features/<modulo>/__tests__/          ← vitest
    use-<hook>.test.ts
    grouping.test.ts                    ← cache categorization
  lib/__tests__/
    utils.test.ts                       ← formatBytes, formatDate, etc.

scripts/
  qa-vm/
    01-prepare-vm.ps1                   ← snapshot, install ClearTool
    02-run-checklist.md                 ← procedure
    03-collect-evidence.ps1             ← captura logs/screenshots
```

### 4.1 Nuevo: `explorer_streaming.rs`

```rust
//! Test que el contrato nuevo de eventos del explorer (post PLAN-EXPLORER-REDISENIO.md)
//! emite scanId + parentPath consistentes.

#[cfg(windows)]
#[tokio::test]
#[ignore]  // depende de FS real
async fn explorer_emite_eventos_con_scan_id() {
    // Setup: usar tempdir con árbol conocido
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("a")).unwrap();
    std::fs::create_dir(tmp.path().join("b")).unwrap();
    std::fs::create_dir(tmp.path().join("a/aa")).unwrap();

    // Llamar list_top_level y verificar que devuelve a y b, no aa
    let nodes = cleartool::domain::explorer::list_top_level(
        &tmp.path().to_string_lossy(), false
    ).expect("list ok");

    assert_eq!(nodes.len(), 2);
    let names: Vec<_> = nodes.iter().map(|n| n.name.clone()).collect();
    assert!(names.contains(&"a".to_string()));
    assert!(names.contains(&"b".to_string()));
    assert!(!names.contains(&"aa".to_string()), "aa es hijo de a, no debe aparecer en top level");
}
```

---

## 5. Test inventory por módulo

### 5.1 Catálogos (`tests/catalogs.rs`)

Definido en [00-CATALOGOS §9](00-CATALOGOS.md#9-tests). Mínimo:

- [ ] `cache_locations_parsea` — 1 entry mínimo válido.
- [ ] `bloatware_catalog_parsea` — sin IDs duplicados.
- [ ] `services_catalog_parsea` — sin nombres duplicados.
- [ ] `registry_tweaks_parsea` — sin IDs duplicados.
- [ ] `registry_tweaks_hives_validos` — hive ∈ {HKLM, HKCU, HKCR, HKU}.
- [ ] `risk_es_lowercase` — sin variantes "Low"/"low" mixtas.
- [ ] `presets_son_validos` — preset ∈ {minimal, recommended, aggressive, total}.
- [ ] Test JSON Schema (opcional, recomendado): cada catálogo cumple su `*.schema.json`.

**CI:** estos tests NO son `#[ignore]`. Bloquean el build.

### 5.2 Audit log (`tests/audit.rs`)

Definido en [02-AUDIT-LOG §7](02-AUDIT-LOG.md#71-unit-tests). Mínimo:

- [ ] `make_entry_genera_uuid_unico`.
- [ ] `write_and_list_roundtrip`.
- [ ] `linea_corrupta_no_rompe_lectura`.
- [ ] `rotacion_a_10mb_funcional` — escribir 11 MB de entries, verificar archivo nuevo + archived.

```rust
#[test]
fn rotacion_a_10mb_funcional() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::env::set_var("APPDATA", tmp.path());

    let big_entry = make_big_entry(50_000); // ~50KB per entry
    for _ in 0..220 {  // 220 * 50KB ≈ 11MB
        cleartool::domain::audit::write_entry(&big_entry).unwrap();
    }

    let archive = cleartool::core::config::audit_log_archive_dir();
    let archived = std::fs::read_dir(&archive).unwrap().count();
    assert!(archived >= 1, "esperado al menos un archivo rotado");
    let current_size = std::fs::metadata(cleartool::core::config::audit_log_path()).unwrap().len();
    assert!(current_size < 10 * 1024 * 1024);
}
```

### 5.3 Registry (`tests/registry.rs` + `tests/registry_apply.rs`)

Definido en [03-REGISTRY §7](03-REGISTRY.md#7-tests).

**CI ejecuta:**
- [ ] `hive_no_permitido_falla` — sin admin.
- [ ] `dry_run_no_escribe`.

**Manual:**
- [ ] `aplicar_y_revertir_show_extensions`.

### 5.4 Servicios (`tests/services_platform.rs`)

Definido en [04-SERVICES §7](04-SERVICES.md#7-tests).

**CI:**
- [ ] `list_all_no_panicea_y_devuelve_lista`.
- [ ] `allowlist_rechaza_servicio_fuera`.

**Manual:**
- [ ] Aplicar preset recomendado en VM, verificar SCM, verificar audit.

### 5.5 Debloat (`tests/debloat_validators.rs`)

Definido en [05-DEBLOAT §7](05-DEBLOAT.md#7-tests).

**CI:**
- [ ] `pkg_family_name_validador` — acepta válidos, rechaza inyecciones.
- [ ] `prov_name_validador`.

**Manual:**
- [ ] Eliminar Bing News + Bing Weather, verificar audit + Get-AppxPackage.

### 5.6 Cache (`tests/cache_filters.rs`)

Definido en [06-CACHE-FINAL §7](06-CACHE-FINAL.md#7-tests).

**CI:**
- [ ] `exclude_extension_funciona`.
- [ ] `filtros_vacios_dejan_pasar_todo`.
- [ ] `older_than_days_funciona`.
- [ ] `expand_path_variables_conocidas`.

### 5.7 Restore points (`tests/restore_integration.rs`)

Definido en [01-RESTORE-POINTS §7](01-RESTORE-POINTS.md#7-tests).

**CI:**
- [ ] `parse_wmi_datetime_basico`.

**Manual:**
- [ ] `is_enabled_no_panicea`, `list_no_panicea`, `ciclo_completo`.

### 5.8 Settings (`tests/settings_roundtrip.rs`)

Definido en [07-SETTINGS §7](07-SETTINGS.md#71-test-de-roundtrip-settings).

**CI:**
- [ ] `settings_roundtrip`.
- [ ] `settings_corruptos_restauran_defaults`.

### 5.9 Adversarial (`tests/adversarial.rs`)

Definido en [08-SEGURIDAD-FINAL §5](08-SEGURIDAD-FINAL.md#52-cómo-ejecutar-test-manual). Mínimo 8 casos.

**CI:** todos.

### 5.10 Frontend (`src/**/__tests__/`)

```ts
// src/lib/__tests__/utils.test.ts
import { describe, it, expect } from "vitest";
import { formatBytes, formatDate } from "../utils";

describe("formatBytes", () => {
  it("formatea bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1024 * 1024)).toBe("1.0 MB");
    expect(formatBytes(1.5 * 1024 * 1024 * 1024)).toBe("1.50 GB");
  });

  it("maneja números negativos como 0", () => {
    expect(formatBytes(-100)).toBe("0 B");
  });
});

describe("formatDate", () => {
  it("ISO a locale", () => {
    const r = formatDate("2026-05-21T14:32:00Z");
    expect(r).toMatch(/2026/);
  });
});
```

```ts
// src/features/cache-cleaner/__tests__/grouping.test.ts
import { describe, it, expect } from "vitest";
import { groupByCategory, CATEGORIES } from "../lib/grouping";
import type { CacheLocation } from "../../../api";

const make = (id: string, cat: string): CacheLocation => ({
  id, displayName: id, path: `%TEMP%\\${id}`,
  category: cat, requiresAdmin: false, risk: "low",
  consequences: [], filters: [], preconditions: [],
});

describe("groupByCategory", () => {
  it("agrupa por category", () => {
    const locs = [make("a", "system"), make("b", "user"), make("c", "system")];
    const map = groupByCategory(locs);
    expect(map.get("system")!.length).toBe(2);
    expect(map.get("user")!.length).toBe(1);
  });

  it("categoría desconocida cae en tools", () => {
    const locs = [make("a", "no-existe")];
    const map = groupByCategory(locs);
    expect(map.get("tools")!.length).toBe(1);
  });
});
```

---

## 6. Configuración CI (GitHub Actions)

**Archivo:** `.github/workflows/ci.yml` (nuevo)

```yaml
name: CI

on:
  push:
    branches: [main, "refactor/**", "feat/**"]
  pull_request:
    branches: [main]

jobs:
  build-and-test:
    runs-on: windows-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Install npm deps
        run: npm ci

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"

      - name: Rustfmt
        run: cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

      - name: Clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

      - name: Cargo test (no ignored)
        run: cargo test --manifest-path src-tauri/Cargo.toml
        env:
          # Aseguramos APPDATA bajo TEMP del runner para tests aislados
          APPDATA: ${{ runner.temp }}\AppData

      - name: TypeScript check
        run: npx tsc --noEmit

      - name: Vitest
        run: npx vitest run

      - name: Build Tauri (dev — verifica que compila)
        run: npm run tauri build -- --debug
        env:
          TAURI_SIGNING_PRIVATE_KEY: ""
```

> **Nota:** el job `Build Tauri` puede ser pesado (5-10 min en cold cache). Si CI es lenta, moverlo a un workflow separado triggered solo en tags `v*`.

---

## 7. Procedimiento QA manual en VM limpia

### 7.1 Preparación de la VM

**Archivo:** `scripts/qa-vm/01-prepare-vm.ps1` (documento — no se ejecuta automáticamente).

1. Hyper-V o VirtualBox o VMware Workstation.
2. Imagen base: **Windows 11 Pro 23H2 ISO** (descarga directa de Microsoft).
3. Configuración:
   - 4 vCPU, 8 GB RAM, 80 GB disco dinámico.
   - Sin telemetría adicional (Express settings → manual setup).
   - Cuenta local "qa-tester" (sin cuenta Microsoft).
   - **NO instalar updates iniciales** (para tener instalación virgen).
4. Tras primer boot:
   - Crear snapshot "VIRGEN_PRE_CLEARTOOL".
   - `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser`.

### 7.2 Checklist de QA (`02-run-checklist.md`)

```markdown
# QA Checklist ClearTool 1.0 — VM virgen

## Setup
- [ ] Snapshot revertida a "VIRGEN_PRE_CLEARTOOL".
- [ ] Build de ClearTool (`ClearTool_1.0.0_x64-setup.exe`) copiado a la VM.
- [ ] Instalar — verificar pantalla SmartScreen aparece, ignorar y continuar.
- [ ] App lanza, UAC salta, dashboard pinta.

## Home / Dashboard
- [ ] CPU + RAM ring renderizan datos reales.
- [ ] GPU card detecta tarjeta (en VM puede ser "Microsoft Basic Display").
- [ ] Disk tube muestra C:\.
- [ ] Top processes table puebla.
- [ ] No hay consolas de PowerShell parpadeando.

## Explorer
- [ ] Selector de disco aparece (post rediseño).
- [ ] C:\ escaneado → primer nivel correcto.
- [ ] Expansión: hijos aparecen anidados (no al lado).
- [ ] Cancel scan funciona.

## Cache
- [ ] Lista categorizada (Sistema/Usuario/Navegadores/Package mgrs).
- [ ] Preset "Usuario" marca 8 entries.
- [ ] Re-escanear suma tamaños correctos.
- [ ] Dry-run reporta sin modificar.
- [ ] Limpiar real: restore point creado + audit entry + tamaños decrecen.

## Debloat
- [ ] Detect identifica Cortana, Bing News, Bing Weather (apps default).
- [ ] Click "Microsoft Edge" → disclaimer modal salta.
- [ ] Apply preset "Recomendado" remueve apps + audit + restore point.
- [ ] Get-AppxPackage en PowerShell confirma remoción.

## Registry
- [ ] 45 tweaks listados, categorizados.
- [ ] Apply "explorer-show-extensions" → modal preview.
- [ ] Confirmar → HKCU\...\HideFileExt = 0 verificado en regedit.
- [ ] Revert from audit → HideFileExt vuelve a 1.

## Services
- [ ] 248+ servicios listados.
- [ ] Filtro "Solo catálogo" → ~58.
- [ ] Apply preset "Recomendado" → progress modal, audit entries.
- [ ] DiagTrack en services.msc = Disabled.

## Restore Points
- [ ] Banner "System Protection OFF" si aplica.
- [ ] Click "Activar" → enabled tras admin elevation.
- [ ] Crear punto manual → aparece arriba de la lista.
- [ ] Restore-to NO se ejecuta en QA (reiniciaría la VM).

## Audit Log
- [ ] Lista contiene todos los runs de tests anteriores.
- [ ] Revertir entry de registry → status visible.
- [ ] Click "Exportar log" → JSON descargado.

## Settings
- [ ] Tabs cambian de contenido.
- [ ] Activar dry-run global → banner amarillo en todas las pantallas.
- [ ] Apply tweak en dry-run global → audit entry con dryRun: true.
- [ ] Desactivar dry-run global → banner desaparece.
- [ ] Cambiar idioma a EN → UI re-renderiza (post i18n).

## Uninstall
- [ ] Panel de control → Uninstall ClearTool.
- [ ] %APPDATA%\ClearTool persiste (decisión: NO borrar audit log al desinstalar).
- [ ] Reinstall → settings recuperados.

## Stress
- [ ] Lanzar batch grande (preset agresivo registry + servicios + debloat).
- [ ] App no se congela durante batch (progress bar avanza).
- [ ] Total > 100 audit entries generadas.
- [ ] Rotación audit log a 10 MB no rompe app.

## Errores esperados (deben verse, no romper)
- [ ] Apagar System Restore antes de batch → app reporta error claro, no crashea.
- [ ] Lanzar sin admin (debug) → banner "Modo limitado", botones disabled.
- [ ] Cerrar app durante batch → al reabrir, ningún proceso huérfano de PowerShell.
```

### 7.3 Recolección de evidencia (`03-collect-evidence.ps1`)

```powershell
# Captura logs, audit, screenshots para reporte de QA.
param([string]$OutDir = "C:\QA-Output")

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force $OutDir | Out-Null

# Logs operativos
Copy-Item -Recurse "$env:LOCALAPPDATA\ClearTool\logs" "$OutDir\logs"

# Audit log
Copy-Item "$env:APPDATA\ClearTool\audit.jsonl" "$OutDir\audit.jsonl"
Copy-Item -ErrorAction SilentlyContinue "$env:APPDATA\ClearTool\settings.json" "$OutDir\settings.json"

# Captura de restore points
Get-ComputerRestorePoint | ConvertTo-Json | Out-File "$OutDir\restore-points.json"

# Estado de servicios tuneados
$tuned = "DiagTrack","dmwappushservice","MapsBroker","RemoteRegistry","XblAuthManager"
Get-Service $tuned -ErrorAction SilentlyContinue |
    Select-Object Name, Status, StartType |
    ConvertTo-Json | Out-File "$OutDir\services-state.json"

# Appx instalados
Get-AppxPackage | Select-Object Name, PackageFamilyName |
    ConvertTo-Json -Depth 3 | Out-File "$OutDir\appx-after.json"

Write-Host "Evidencia en $OutDir"
```

---

## 8. Smoke test en CI release

Cuando se taggea `v1.0.0`, workflow extra que:

1. Buildea release artifact.
2. Lanza el `.exe` headless en runner (no es viable; Tauri necesita UI).
3. **Alternativa práctica:** corre solo `cargo test --release` + verifica que el `.exe` no es 0 bytes y `signtool verify` (cuando haya firma) pasa.

```yaml
# .github/workflows/release.yml (esqueleto)
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  build-release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: "20", cache: "npm" }
      - run: npm ci
      - run: npm run tauri build
      - name: Verify artifact exists and is non-empty
        shell: pwsh
        run: |
          $exe = Get-ChildItem -Recurse -Filter "ClearTool_*_x64-setup.exe" | Select-Object -First 1
          if ($null -eq $exe) { throw "exe no generado" }
          if ($exe.Length -lt 1MB) { throw "exe sospechosamente pequeño" }
          Write-Host "OK: $($exe.Name) — $([math]::Round($exe.Length/1MB,2)) MB"
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            src-tauri/target/release/bundle/nsis/*.exe
            src-tauri/target/release/bundle/msi/*.msi
```

---

## 9. Tests del frontend a evitar (anti-patterns)

- **NO** test de componentes con `@testing-library/react` snapshot — frágiles, alto mantenimiento.
- **NO** test de routing con MemoryRouter — añade complejidad sin valor.
- **NO** test con `tauri-driver` para v1.0 — setup costoso, mejor invertir en QA manual.

Lo que sí: tests de hooks puros + funciones de transformación de datos (grouping, sorting, formatting).

---

## 10. Definition of Done

- [ ] `.github/workflows/ci.yml` existe y pasa en CI.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` pasa local (no `#[ignore]`).
- [ ] `cargo clippy -- -D warnings` pasa.
- [ ] `cargo fmt --check` pasa.
- [ ] `npx tsc --noEmit` pasa.
- [ ] `npx vitest run` pasa (mínimo 5 tests).
- [ ] Procedimiento QA documentado en `scripts/qa-vm/`.
- [ ] Checklist QA ejecutado al menos una vez en VM virgen, evidencia archivada.
- [ ] `scripts/qa-vm/03-collect-evidence.ps1` ejecutado, output review.
- [ ] Sin tests `#[ignore]` que deberían correr — todos los ignored tienen razón documentada.
- [ ] Commit `test: suite mínima viable + CI + QA procedure`.

---

## 11. Próximo archivo

→ [10-DISTRIBUCION.md](10-DISTRIBUCION.md) — generación del instalador NSIS, manifest final, iconos, metadatos, manejo de SmartScreen sin firma.
