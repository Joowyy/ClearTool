---
name: rust-testing-ci
description: Estrategia de tests y CI para ClearTool — tests obligatorios de módulos destructivos en Rust (validación de allowlist, idempotencia, dry-run no toca nada, revert restaura el estado previo), uso de VM Windows 11 para tests de integración reales, y workflow de GitHub Actions para lint+build (fmt, clippy -D warnings, cargo test, eslint, build Tauri). Usar al escribir tests de cualquier módulo que toque registro/servicios/borrado, al configurar CI, o al definir el criterio de "done" de un milestone. Empieza CI en M1/M2, no al final.
---

# Skill: rust-testing-ci

Disciplina de pruebas y CI. Regla transversal: **ningún módulo destructivo se da por hecho sin sus tests**; el CI básico (lint+build) arranca ya en M1, los tests del error model en M2.

## Cuándo usar

- Escribes tests de un módulo que toca registro, servicios, procesos o borrado de archivos.
- Configuras o amplías GitHub Actions.
- Defines/evalúas el criterio "done" de un milestone.

## Los 4 tests no negociables de un módulo destructivo

Todo comando destructivo (debloat, registry tweak, disable servicio, kill proceso, clean cache) prueba como mínimo:

1. **Allowlist / validación**: inputs fuera de la whitelist → `AppError::Permission`, sin tocar nada.
2. **Dry-run inerte**: con `dry_run = true` no se crea restore point, no se borra, no se escribe registro — solo se devuelve el plan.
3. **Idempotencia**: ejecutar dos veces no rompe ni deja estado inconsistente (segunda vez → `already-done`/`already-clean`).
4. **Reversibilidad**: `revert` (o aplicar el `reverse_recipe` del audit log) restaura exactamente el estado previo.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_outside_allowlist() {
        let r = validate_write_path(Hive::Hklm, r"SAM\foo");
        assert!(matches!(r, Err(AppError::Permission(_))));
    }

    #[tokio::test]
    async fn dry_run_creates_no_restore_point() {
        let mock = RestoreSpy::default();
        let _ = clean_location(&loc(), /*dry_run*/ true).await.unwrap();
        assert_eq!(mock.calls(), 0);
    }

    #[test]
    fn applying_twice_is_idempotent() {
        let edit = sample_edit();
        edit.apply().unwrap();
        edit.apply().unwrap();              // no debe entrar en pánico ni error
    }

    #[test]
    fn revert_restores_previous_value() {
        let edit = sample_edit();
        edit.apply().unwrap();
        edit.revert().unwrap();
        assert_eq!(read_value(&edit), edit.previous);
    }
}
```

## Tres niveles de test

- **Unitarios (puros)**: validadores, parseo de catálogos, clasificación de procesos, `normalizeError`. Corren en CI Linux/Windows sin permisos.
- **Integración con mocks**: servicios destructivos con dobles de restore point / powershell / registro. Verifican orquestación (orden: precondición → restore point → operación → audit log).
- **Integración real en VM Windows 11**: lo que toca el SO de verdad (Appx remove, restore point real, who_locks_path). NO corre en CI por defecto; checklist manual en VMs OEM (Lenovo, HP) + ISO limpia, en builds 22H2/23H2/24H2.

## PowerShell / Win32: cómo testear sin VM

- Validación de args: confirmar que `;`, `&`, `|`, `$`, backtick son rechazados.
- Timeout: script con `Start-Sleep` largo debe abortar.
- `who_locks_path`: abrir un archivo desde un proceso de prueba y verificar PID; cerrar y verificar `[]`.
- Parsear salidas JSON canónicas guardadas como fixtures (no llamar a PowerShell real en el test unitario).

## GitHub Actions

### ci.yml — lint + build (arranca en M1)

```yaml
on: [push, pull_request]
jobs:
  check:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - run: npm ci
      - run: npm run lint            # eslint + prettier --check
      - run: cargo fmt --check
        working-directory: src-tauri
      - run: cargo clippy -- -D warnings
        working-directory: src-tauri
      - run: cargo test
        working-directory: src-tauri
      - run: npm run build           # vite typecheck + build del frontend
```

`clippy -D warnings` y `fmt --check` son obligatorios (convención del proyecto). El build de Tauri completo puede ir en un job aparte por coste/tiempo.

### Relación con release.yml

`release.yml` (firma + bundle, ver skill `tauri-release-signing`) se dispara por tag y asume que `ci.yml` ya pasó. No dupliques lint en release.

## Métricas por milestone

Registrar al cerrar cada milestone: tests añadidos, bugs cerrados/abiertos, LOC +/-, y tiempo commit→artifact (debe bajar con CI). Post-launch el target de critical bugs es 0.

## Checklist de "done" de un milestone (plantilla)

- [ ] Los 4 tests destructivos cubren cada comando nuevo destructivo.
- [ ] `cargo clippy -D warnings` y `cargo fmt --check` verdes.
- [ ] `npm run lint` y build del frontend verdes.
- [ ] Smoke test manual en al menos 1 VM Windows 11.
- [ ] Sin deuda técnica P0 arrastrada al siguiente milestone.
