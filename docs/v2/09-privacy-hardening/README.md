# 09 — Privacy Hardening (M3, P1)

**Objetivo**: 3 niveles de "modo privado" que aplican bundles coordinados de registry tweaks + servicios + scheduled tasks + debloat.

**Spec de referencia**: `.claude/specs/v2/05-new-modules.md` §5.5.

## Pasos

1. [01 — Definir los 3 presets en JSON](01-presets-definir.md)
2. [02 — Domain: `apply_privacy_preset`](02-domain-apply-preset.md)
3. [03 — IPC + página dedicada](03-ipc-y-frontend.md)

## Criterio de done

- [ ] 3 presets (Balanced, Strict, Paranoid) definidos en JSON.
- [ ] `apply_privacy_preset(level, dry_run)` aplica el bundle.
- [ ] Página /privacy con 3 radio buttons + preview.
- [ ] "Ver cambios" antes de aplicar muestra exactamente qué se va a tocar.
- [ ] Restore point obligatorio antes de aplicar.

## Tiempo total

10-12 horas.
