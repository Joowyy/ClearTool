# 02 — Borrado del módulo Explorer

El antiguo "Explorador" (`/explorer`) es redundante con el nuevo Disk
Analyzer. Se elimina por completo: ruta, UI, comandos Tauri, dominio,
modelos exclusivos, hooks y referencias en localización.

## Inventario a eliminar

### Frontend

- `src/features/explorer/` (carpeta entera):
  - `explorer-page.tsx`
  - `tree-view.tsx`
  - `tree-row.tsx`
  - `use-explorer-tree.ts`
- `ROUTES.EXPLORER` en `src/lib/routes.ts`.
- Import lazy `ExplorerPage` y ruta `{ path: ROUTES.EXPLORER, ... }` en
  `src/router.tsx`.
- Item de navegación en `src/components/layout/sidebar.tsx`.
- Item de navegación en `src/components/layout/tab-nav.tsx`.
- Item `nav-explorer` en `src/components/command-palette.tsx`.
- Atajo `Ctrl+E` en `src/hooks/use-keyboard-shortcuts.ts`.
- Locale key `nav.explorer` en `src/locales/{en,es}/common.json`.
- Tipos sólo usados por explorer en `src/api/types.ts`:
  - `ScanTreeInput`, `ScanTreeHandle`, `TreeNode`, `DirectorySize`,
    `SizeStrategy`.
- Funciones cliente en `src/api/client.ts`:
  - `scanTree`, `listDir`, `cancelScan`, `computeDirectorySize`.
- Eventos `ExplorerNode`, `ExplorerDone` en `src/api/events.ts` y sus
  payloads.
- Mocks en `src/api/client.ts` (modo web): `scan_tree`, `list_dir`,
  `cancel_scan`, `compute_directory_size`.

### Backend

- `src-tauri/src/ipc/explorer.rs` (archivo entero).
- `pub mod explorer;` en `src-tauri/src/ipc/mod.rs`.
- `src-tauri/src/domain/explorer.rs` (archivo entero).
- `pub mod explorer;` en `src-tauri/src/domain/mod.rs`.
- Comandos en `invoke_handler!` de `src-tauri/src/lib.rs`:
  - `ipc::explorer::scan_tree`
  - `ipc::explorer::list_dir`
  - `ipc::explorer::cancel_scan`
  - `ipc::explorer::compute_directory_size`
- Modelo `src-tauri/src/models/tree.rs`: la mayoría desaparece. Pero
  **`NodeKind` se preserva** y se traslada a `src-tauri/src/models/disk.rs`
  (es el único símbolo que el Disk Analyzer todavía usa).

## Orden de operaciones (evita compilación rota)

1. Mover `NodeKind` a `models/disk.rs` y actualizar el `use` en
   `domain/disk.rs`.
2. Eliminar referencias en `lib.rs` (los 4 `invoke_handler!` lines).
3. Eliminar `ipc::explorer` (módulo + archivo).
4. Eliminar `domain::explorer` (módulo + archivo).
5. Eliminar `models/tree.rs` y su mod en `models/mod.rs`.
6. Frontend: borrar `src/features/explorer/`.
7. Frontend: quitar tipos/funciones explorer-only de `api/types.ts` y
   `api/client.ts` (incluyendo mocks).
8. Frontend: quitar evento y tipo de `api/events.ts`.
9. Frontend: quitar ruta, sidebar, tab-nav, command-palette, shortcuts.
10. Frontend: quitar key de locales.
11. Compilar (`cargo check` + `npm run build`) — fijar lo que quede roto.

## Verificación

```powershell
# No debería haber matches después del borrado.
rg -n "ROUTES\.EXPLORER|nav-explorer|explorer-page|tree-view|tree-row|use-explorer-tree" src/
rg -n "ipc::explorer|domain::explorer|scan_tree|list_dir|cancel_scan|compute_directory_size|ScanTreeInput|TreeNode|DirectorySize" src-tauri/src/
```

Excepción: `"explorer.exe"` (string literal) en `domain/cache.rs` y
`platform/processes.rs` se preserva — es el proceso shell de Windows, no
nuestro módulo.
