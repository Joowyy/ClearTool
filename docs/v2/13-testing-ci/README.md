# 13 — Testing + CI (continuo, P2)

**Objetivo**: red de seguridad mínima para refactorizar sin romper. Tests unitarios en Rust, tests de componentes en frontend, smoke en VM, CI que valida todo en cada PR.

**Spec de referencia**: `.claude/v1/07-testing.md` (heredado).

## Filosofía

- **No testing-first dogmático**. Algunos módulos son inherentemente difíciles de testear (registry, MoveFileEx). Testear lo testeable, smoke-test lo inestable.
- **VM snapshots > mocks**. Para módulos destructivos, validar contra una VM real con snapshots Hyper-V/VMware. Los mocks dan falsa seguridad.
- **CI mínimo viable**. Build + clippy + cargo test + tsc + ajv catalogs. No bloquear PR por falta de tests E2E.

## Pasos en orden

1. [01 — Rust unit tests para platform + domain](01-rust-unit-tests.md)
2. [02 — Frontend component + integration tests](02-frontend-tests.md)
3. [03 — VM snapshot plan (manual)](03-vm-snapshot-plan.md)
4. [04 — GitHub Actions: build + release pipelines](04-github-actions.md)

## Cuándo empezar cada paso

- **Paso 04 (CI básico)**: en M1, paralelo con error-handling. Tener un build verde da confianza.
- **Paso 01 (Rust tests)**: a medida que se escribe cada módulo nuevo. NO de golpe al final.
- **Paso 02 (Frontend tests)**: opcional para v1.0. Útil pero no bloqueante.
- **Paso 03 (VM snapshots)**: antes de release público — validación final M5.

## Criterio de done (área)

- [ ] `cargo test` corre 50+ tests verdes.
- [ ] `npm test` corre tests de `errors.ts`, `toast.ts`, normalizers.
- [ ] `npm run validate-catalogs` valida los 4 JSON contra schemas.
- [ ] CI en cada PR: build + lint + tests + catalog validation.
- [ ] CI en cada tag: release firmado + auto-updater latest.json + upload artifacts.
- [ ] Plan de VM snapshots documentado con 5 escenarios obligatorios pre-release.

## Tiempo total

15-20 horas (más continuo durante el desarrollo de otros módulos).
