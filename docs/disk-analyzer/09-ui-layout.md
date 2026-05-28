# 09 — Layout y look futurista

## Estructura global de la página

```
┌──────────────────────────────────────────────────────────────────┐
│  HEADER                                                          │
│  [HD▾ C: OS  786 / 1000 GB]  [Escanear]  [⚙ Opciones]  [↻]      │
│  C:\  ›  Users  ›  joels  ›  AppData                            │
├────────────────────┬─────────────────────────────────────────────┤
│ TREE (350 px)      │  STATS BLOCKS (4 tarjetas, fluido)          │
│ ▾ C:\              │  [Disco] [Totales] [Antigüedad] [Categorías]│
│   ▾ Users          ├─────────────────────────────────────────────┤
│     ▾ joels        │  TREEMAP CANVAS (zona expansiva)            │
│       AppData ...  │                                             │
│       Documents... │                                             │
│   Windows          │                                             │
│   ...              │                                             │
│                    ├─────────────────────────────────────────────┤
│                    │  TOP CARPETAS  |  TOP ARCHIVOS  |  TOP EXT  │
│                    │  (tabs)                                     │
└────────────────────┴─────────────────────────────────────────────┘
```

Grid CSS:

```css
.disk-page {
  display: grid;
  grid-template-rows: auto auto 1fr;
  grid-template-columns: minmax(280px, 350px) 1fr;
  grid-template-areas:
    "header  header"
    "crumbs  crumbs"
    "tree    main";
}
```

El `main` interior es `flex flex-col` con stats arriba, treemap en el medio
(con `flex: 1`), tabs de tops abajo.

## Componentes nuevos

| Componente | Archivo | Resp. |
|-----------|---------|--------|
| `DriveSelector` | `src/features/disk/drive-selector.tsx` | Dropdown con lista de discos. |
| `Breadcrumbs` | `src/features/disk/breadcrumbs.tsx` | Path navegable. |
| `StatsBlocks` | `src/features/disk/stats-blocks.tsx` | Conjunto de 4 tarjetas. |
| `DiskUsageDonut` | `src/features/disk/disk-usage-donut.tsx` | Donut SVG. |
| `AgeBar` | `src/features/disk/age-bar.tsx` | Barras horizontales. |
| `CategoryDonut` | `src/features/disk/category-donut.tsx` | Donut por categoría. |
| `HierarchyTree` | `src/features/disk/hierarchy-tree.tsx` | Tree virtualizado. |
| `TreeRow` | `src/features/disk/tree-row.tsx` | Fila del tree. |
| `TreemapCanvas` | `src/features/disk/treemap-canvas.tsx` | Canvas del treemap (refactor del existente). |
| `TopFoldersTable` | `src/features/disk/top-folders-table.tsx` | Tabla de top carpetas. |
| `TopFilesTable` | `src/features/disk/top-files-table.tsx` | Tabla de top archivos. |
| `TopExtensionsTable` | `src/features/disk/top-extensions-table.tsx` | Tabla de top extensiones. |
| `useDiskScan` | `src/features/disk/use-disk-scan.ts` | Hook que orquesta listen + invoke. |
| `disk-page` | `src/features/disk/disk-page.tsx` | Reescrito desde cero. |

## Tokens visuales

Reusar los ya disponibles en `tailwind.config.js`:

- `signal-cyan`, `signal-violet`, `signal-amber` para acentos.
- `panel-raised`, `surface-inset` para fondos.
- `edge-default/10`, `edge-default/30` para bordes.
- `ink-primary`, `ink-tertiary`, `ink-muted` para texto.

Añadir si no existen:

```css
.disk-glow {
  box-shadow:
    0 0 0 1px rgba(34, 211, 238, 0.15),
    0 0 24px -6px rgba(34, 211, 238, 0.25);
}
```

## Tipografía y números

- Nombres de carpeta / archivo: `font-medium tracking-tight`.
- Tamaños/conteos: `font-mono tabular-nums text-ink-primary`.
- Etiquetas: `text-[10px] uppercase tracking-wider text-ink-muted`.

## Estados de carga

- Empty (sin disco analizado): hero card central con icono `HardDrive`,
  CTA "Selecciona un disco y pulsa Escanear".
- Loading: `disk:progress` actualiza contadores en vivo en las tarjetas.
  El treemap muestra un shimmer animado mientras `disk:complete` no haya
  llegado.
- Error: toast + tarjeta de error en el área del treemap con botón "Reintentar".

## Animaciones

- Entrada de la página: stagger 80 ms desde stats blocks.
- Cambio de disco: fade del contenido viejo + scale-up del nuevo.
- Zoom in/out del treemap: `framer-motion` con transición `layout`.
- Glow pulsante muy sutil en el bloque seleccionado (treemap + tree).

## Responsive

Sólo desktop (>= 1024 px). Si la app se redimensiona por debajo, el panel
del tree se colapsa a un icono "Mostrar árbol" arriba a la izquierda. El
treemap siempre prima.
