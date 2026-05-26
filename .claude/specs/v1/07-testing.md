# 07 — Testing

## Filosofía

ClearTool ejecuta operaciones irreversibles en sistemas reales. La prueba más relevante no es "el código compila", es "ejecutar la operación en una VM Windows 11 limpia deja el sistema en el estado esperado, y reaplicarla no rompe nada". La estrategia tiene tres capas con responsabilidades distintas.

## Pirámide de tests

```
                    ┌──────────────────────┐
                    │   E2E en VM Windows  │  ~10 escenarios, 1× por release
                    │   (Playwright + RDP) │
                    └──────────────────────┘
                  ┌────────────────────────────┐
                  │  Integration Rust + PS     │  ~50 tests por módulo
                  │  (sandbox: temp_dir, mock  │
                  │  registry hive, mock SCM)  │
                  └────────────────────────────┘
              ┌───────────────────────────────────┐
              │   Unit Rust + Vitest              │  cientos
              │   pure logic, validators, parsers │
              └───────────────────────────────────┘
```

## Capa 1: Unit

### Rust

- `cargo test --workspace` corre todo.
- Cobertura objetivo:
  - `services::filesystem`: 95 %.
  - `services::registry`: 95 %.
  - `services::audit_log`: 95 %.
  - `services::powershell`: 90 % (lo demás son scripts PS, ver "PS tests").
  - `services::catalog`: 100 % (parser).
  - `commands::*`: 80 % (mucho es delegación pura).
- Herramientas: `cargo-llvm-cov` para cobertura, `proptest` para fuzzing inputs (paths, hex, etc.), `insta` para snapshot tests de serialización.
- Ningún unit test toca el FS real fuera de `tempfile::tempdir()`.
- Ningún unit test toca el registro real. Para `services::registry`, mockear via trait:

```rust
pub trait RegistryBackend {
    fn read_value(&self, hive: Hive, path: &str, name: Option<&str>) -> Result<RegValueSnapshot, AppError>;
    fn write_value(&self, op: &RegistryOp) -> Result<(), AppError>;
    // ...
}

pub struct WinRegBackend;            // producción
pub struct InMemoryRegistryBackend;  // tests
```

Los servicios reciben `Arc<dyn RegistryBackend>` por inyección en la fase de tests; en producción se construye `WinRegBackend`. Mismo patrón para `ServiceManager` y `RestorePoint`.

### Frontend

- `vitest` para hooks y utils puros.
- `@testing-library/react` para componentes.
- Tests para `<ConfirmDestructive>` cubren los 4 tabs y el flow de checkbox-required.
- Mocks de `@tauri-apps/api/core::invoke` y `event::listen` con `vi.mock`.

### Catálogos

Cada catálogo (`cache-locations.json`, `bloatware-catalog.json`, `services-catalog.json`, `registry-tweaks.json`) tiene:

- Test que valida contra su `*.schema.json`.
- Test que verifica que ningún `id` se repite.
- Test que verifica que cada `path`/`hive+path` cae dentro de la allowlist global.
- Test que verifica que cada entrada destructiva tiene `reversal` o `reverse_operations` no vacíos (salvo flag explícito `not_reversible: true`).

## Capa 2: Integration

### Rust + sandbox FS

- Cada módulo destructivo tiene tests con `tempfile::tempdir()` simulando estructuras realistas.
- `cache-cleaner`:
  - Crear árbol `temp/` con 100 archivos de varias edades.
  - Ejecutar `clean_cache_locations` con filtro `older_than_days = 1`.
  - Verificar archivos > 1 día borrados, < 1 día intactos, conteos correctos.
  - Re-ejecutar y verificar idempotencia.
- `directory-explorer`:
  - Árbol con junctions (`mklink /J` solo si Windows + admin; en CI Linux saltar).
  - Verificar tamaños lógicos vs físicos.

### PowerShell scripts

- Cada `.ps1` tiene un test runner en `src-tauri/tests/ps/`.
- El runner ejecuta el script con args benignos y verifica el JSON output.
- Tests de fuzzing: inputs con `;`, `|`, `$()`, backticks → script debe rechazarlos vía `param([ValidatePattern(...)])` o el wrapper Rust debe rechazarlos antes.
- En CI Windows, el test corre con `pwsh.exe` real. En CI Linux, los tests PS se saltan (`#[cfg(target_os = "windows")]`).

### Registro

- Tests usan `InMemoryRegistryBackend`.
- Round trip: aplicar tweak → leer estado → assert `Applied`. Revertir → assert `NotApplied`.
- Test de allowlist: intentar escribir fuera y verificar `Permission`.
- Test de batch con fallo intermedio: mock que falla en op #3 → verificar rollback de ops #1 y #2.

### Audit log

- Test de concurrencia: 16 threads × 1 000 appends → archivo bien formado (líneas válidas, sin tearing).
- Test de rotación: forzar archivo > 50 MB → verificar `audit.jsonl.1` creado, nuevo `audit.jsonl` vacío.
- Test de chain hash (cuando se implemente): manipular una línea → detectar.

### Service Manager

- `InMemoryServiceManager` simula SCM.
- Lifecycle: stop → setStartType → start → assert estados.
- Guard pattern: panic mid-operation → verificar `Drop` restauró servicio (vía mock).

### Restore Point

- En CI no podemos crear restore points reales. Mock `RestorePointBackend` para tests Rust.
- Test que `create_restore_point` es llamado **antes** de cualquier write destructivo en cada comando que lo requiera (assertion via spy).

## Capa 3: E2E en VM Windows 11

### Setup

- VM gold: Windows 11 23H2 limpia + Office stub + bloatware OEM realista preinstalado (Candy Crush, Spotify, Xbox stack, Cortana, Edge, OneDrive, Teams system).
- Snapshot baseline antes de cada batería de tests.
- Tras cada test, revert al snapshot.
- Orquestación: Hyper-V o VirtualBox con `vagrant`. Los tests corren desde el host vía:
  - `playwright` apuntando a la VM con Tauri WebDriver expuesto en un puerto.
  - O `winrm`/`ssh` para invocar el binario y leer el audit log.

### Escenarios mínimos para release

| # | Escenario | Resultado esperado |
|---|---|---|
| 1 | App levanta sin admin | Banner "modo limitado", Explorer y Cache scan funcionan |
| 2 | App levanta con admin | Chip "admin", todas las funciones disponibles |
| 3 | Limpiar `user.temp` + `system.temp` (dry-run) | Reporte muestra MB previstos, nada cambió |
| 4 | Limpiar `user.temp` + `system.temp` (real) | Archivos < 1 día intactos; antiguos borrados; restore point creado; audit entry presente |
| 5 | Re-ejecutar #4 | Idempotente: 0 errores, MB liberados ≈ 0 |
| 6 | `Debloat Recomendado` | Candy Crush, Spotify, Xbox, Cortana removidos; Edge intacto; reporte coincide con detección post-run |
| 7 | `Debloat Total` | Edge, Store, OneDrive removidos; restore point creado; audit entry con reverse_recipe |
| 8 | `restore_to_point` tras #7 | Tras reboot, Edge/Store/OneDrive presentes de nuevo |
| 9 | `apply_service_preset(TelemetryOff)` | DiagTrack `Stopped + Disabled`; reapply = no-op |
| 10 | Aplicar batch tweaks `taskbar.left-align` + `widgets.disable` + reboot Explorer | UI de Windows refleja cambios |

### Métricas de release

Una release no se publica si:

- Algún escenario E2E falla.
- Algún catálogo falla validación.
- Cobertura de tests Rust en módulos destructivos < 90 %.
- Algún audit entry generado en E2E es `not_reversible` y la operación no lo declaró así.
- `cargo clippy -- -D warnings` o `cargo fmt --check` fallan.

## Tests negativos obligatorios

Cada PR que toque módulos destructivos debe incluir al menos un test que verifica:

- Que la operación **no toca** un path/key/service fuera del scope declarado.
- Que la operación **falla limpiamente** ante ausencia de admin (cuando aplica).
- Que la operación **rollback** ante un fallo intermedio.

Sin estos tests, `security-auditor` rechaza el PR.

## Property-based tests

Usar `proptest` para:

- Validador de paths: arbitrary strings → debe terminar siempre en aceptado-y-canonicalizado o rechazado-con-Permission, jamás panic.
- Parser de catálogos: arbitrary JSON malformado → siempre `Err`, jamás panic.
- Encoder de args PS: arbitrary string → si pasa el validador, no contiene `;|&backtick` ni se escapa de forma rota.

## Smoke local antes de PR

Script `scripts/precheck.ps1`:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix . run lint
npm --prefix . run typecheck
npm --prefix . run test
npm --prefix . run build
```

PR rechazado si alguno falla.

## CI

- GitHub Actions, runner `windows-latest`.
- Jobs:
  - `rust-checks` (fmt + clippy + test).
  - `frontend-checks` (eslint + tsc + vitest).
  - `bundle-build` (cargo tauri build, sube artefactos MSI/NSIS para descarga manual).
  - `e2e-vm` (manual trigger, no en cada push: corre escenarios contra la VM gold via runner self-hosted).

## Observabilidad de tests

- Output de tests Rust: `--nocapture` en CI para diagnóstico, formato JSON con `cargo test -- --format json` cuando se publica reporte.
- Reporte HTML de cobertura adjunto al artifact del workflow.
- E2E: cada escenario produce un bundle (`audit.jsonl` snapshot, screenshot final, log app) que se sube como artifact.

## Manual QA checklist

Para cada release, además de los E2E automáticos, un humano (o `qa-tester` agent en una sesión interactiva) ejecuta:

- [ ] Levantar la app sin admin, recorrer cada página, ningún botón destructivo está activo.
- [ ] Levantar con admin, abrir cada página, verificar que la lista de items detectados es no vacía.
- [ ] Aplicar `Debloat Total` en VM con bloatware OEM real → reporte coincide con expectativas.
- [ ] Reiniciar PC, verificar que la UI de Windows respeta los tweaks aplicados.
- [ ] Restaurar al punto creado por `Debloat Total` → verificar bloatware vuelve.
- [ ] Borrar manualmente `audit.jsonl` → la app la recrea sin crashear.
- [ ] Apagar VSS service → la app refleja `RestoreUnavailable` y bloquea destructivas.
