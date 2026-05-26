# 05 — Startup Manager (M3, P1)

**Objetivo**: nueva pestaña Arranque que lista entries de auto-arranque (registry Run, Startup folder, scheduled tasks at logon, services Automatic, UWP startupTask) y permite deshabilitar/habilitar.

**Spec de referencia**: `.claude/specs/v2/05-new-modules.md` §5.1.

## Pasos

1. [01 — Modelo `StartupEntry` + 5 orígenes](01-modelo-y-origenes.md)
2. [02 — Listado: agregador de los 5 orígenes](02-listado-agregado.md)
3. [03 — Disable/Enable reversible](03-disable-enable.md)
4. [04 — Medición de impacto + frontend](04-impacto-y-frontend.md)

## Criterio de done

- [ ] Lista los 5 tipos de auto-arranque en una sola tabla.
- [ ] Disable es reversible (renombra con `.disabled`, no borra).
- [ ] Impacto medido desde Event Log de diagnóstico.
- [ ] UI agrupa por categoría con badges de impacto.

## Tiempo total

12-15 horas.
