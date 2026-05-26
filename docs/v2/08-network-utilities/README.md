# 08 — Network Utilities (M4, P2)

**Objetivo**: 6 botones simples accesibles desde Settings → Tools que ejecutan comandos comunes de red (flush DNS, reset Winsock, etc.).

**Spec de referencia**: `.claude/specs/v2/05-new-modules.md` §5.4.

## Pasos

1. [01 — Backend: 6 comandos de red](01-platform-network-cmds.md)
2. [02 — IPC + UI drawer en Settings](02-ipc-y-ui-drawer.md)

## Criterio de done

- [ ] 6 comandos funcionan (flush DNS, renew IP, reset Winsock, reset TCP/IP, reset proxy, restore hosts).
- [ ] Drawer en Settings agrupa los 6 botones.
- [ ] Cada uno: confirm modal + restore point + audit log.

## Tiempo total

5-6 horas.
