# 11 — UI/UX Refactor (M4, P2)

**Objetivo**: refactor visual completo: sidebar colapsable, command palette, themes, density, atajos, i18n, lazy routes.

**Spec de referencia**: `.claude/specs/v2/06-ui-ux-refactor.md`.

## Pasos en orden

1. [01 — Sidebar colapsable (sustituye tabs de titlebar)](01-sidebar-component.md)
2. [02 — Persistencia del estado del sidebar](02-sidebar-state-persistencia.md)
3. [03 — Command Palette con cmdk](03-command-palette.md)
4. [04 — Theme system (3 themes + follow-system)](04-themes-css-vars.md)
5. [05 — Density toggle (comfortable / compact)](05-density-toggle.md)
6. [06 — Confirm Dialog hook estandarizado](06-confirm-dialog-hook.md)
7. [07 — TableSkeleton y EmptyState con action](07-skeletons-y-empty.md)
8. [08 — Atajos de teclado + i18n + lazy routes](08-shortcuts-i18n-lazy.md)

## Criterio de done (área)

- [ ] Sidebar reemplaza tabs en titlebar.
- [ ] Ctrl+K abre Command Palette con búsqueda fuzzy.
- [ ] 3 temas (dark-cyan, dark-amber, light) + auto-detect OS.
- [ ] Density toggle persistente.
- [ ] `useConfirm()` reemplaza state local de "confirmDelete" en todas las páginas.
- [ ] Loading states usan `<TableSkeleton />` consistente.
- [ ] EmptyStates con `action` opcional.
- [ ] Atajos teclado funcionando, modal Ctrl+/ documenta todos.
- [ ] i18n base (ES + EN) operativo.
- [ ] Rutas pesadas lazy-loaded.

## Tiempo total

40-50 horas. Es el módulo más grande de M4.
