# 10 — App Reset (UWP) (M3, P2)

**Objetivo**: botón "Reset" en página Debloat junto a cada entrada UWP que llama `Reset-AppxPackage`. Escape hatch para casos donde Spotify/Claude/etc. siguen rotos tras un cache cleanup malo.

**Spec de referencia**: `.claude/specs/v2/05-new-modules.md` §5.6.

## Pasos

1. [01 — Platform: `reset_uwp_app`](01-platform-reset-uwp.md)
2. [02 — Integración UI en Debloat](02-frontend-integration.md)

## Criterio de done

- [ ] `Reset-AppxPackage` se ejecuta con validación regex del input.
- [ ] Botón "Reset" en cada fila UWP de Debloat.
- [ ] Confirm modal explica que pierde data local.
- [ ] Audit log entry.

## Tiempo total

3-4 horas.
