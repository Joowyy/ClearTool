# 06 — Boot-time Cleanup (M3, P1)

**Objetivo**: visibilidad y control sobre los `PendingFileRenameOperations` (archivos programados para borrarse en el próximo reboot).

**Spec de referencia**: `.claude/specs/v2/05-new-modules.md` §5.2.

## Pasos

1. [01 — Backend: list/cancel pending renames](01-listar-cancelar-pending.md)
2. [02 — IPC + helpers de schedule](02-ipc-y-helpers.md)
3. [03 — UI: card en Settings + badge en titlebar](03-ui-cards-y-badge.md)

## Dependencias

- `02-cache-engine/02-pending-rename-helper.md` ya implementó las funciones core. Aquí solo es UI + IPC + badge.

## Criterio de done

- [ ] Settings → "Operaciones pendientes para próximo reinicio" muestra la lista.
- [ ] Cancelar entry individual funciona.
- [ ] Si >50 pendings, badge "Reiniciar pronto" en la titlebar.
- [ ] Click en badge → navega a Settings.

## Tiempo total

6-8 horas.
