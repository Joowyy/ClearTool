# 07 — Árbol jerárquico filtrado por peso

El árbol vive en el lateral izquierdo. Es la lectura analítica del mismo
escaneo que pinta el treemap: el usuario ve los nombres y los tamaños
ordenados de mayor a menor, en lugar de áreas proporcionales.

## Modelo (UI)

```ts
interface TreeRow {
  node: TreemapNode;
  depth: number;
  expanded: boolean;
  hasChildren: boolean;
}
```

Aplanamos el árbol al estilo virtual: cada vez que cambia el estado
expand/collapse, regeneramos un `TreeRow[]`. El render usa
`@tanstack/react-virtual` (ya disponible o, en su defecto, `react-window`)
para no pintar miles de filas.

## Ordenación

Siempre **desc por size**. No se ofrece otro orden en MVP: añade
complejidad sin justificación clara.

## Visualización de cada fila

Layout horizontal (alto 32 px):

```
[▾]  📁 node_modules            ▰▰▰▰▰▰▱▱▱▱  12.4 GB · 38.2 %   84.213 files   hace 2 días
```

- Triángulo expand/collapse (rota 90° en expanded).
- Icono según `kind` + `extension` (uses lucide-react + paleta de categoría).
- Nombre truncado con `text-ellipsis`, mostrar full en title.
- Mini-bar (8 px alto, 80 px ancho) de % respecto al padre. Color de la
  categoría dominante de la carpeta (calculado opcionalmente, o gris).
- Size formateado (`formatBytes`).
- `%` respecto al root o al padre — switch en el header de la columna.
- File count.
- `last_modified` en formato relativo (`dayjs.fromNow()`).

Selección (click) → resalta esa fila Y resalta el bloque correspondiente
en el treemap. Doble click → zoom in en treemap.

## Profundidad inicial

- Root abierto.
- Primer nivel (hijos directos) cargado.
- Resto plegado. Expand por click; al expandir, si la carpeta tiene
  `truncated: true`, se reemite `build_treemap_data` con root = ese
  path para cargar sus descendientes. (En MVP, dado que `max_depth_emit
  = 6`, lo normal es que ya estén en memoria.)

## Filtros del árbol

Header de la columna ofrece toggles rápidos:

- **% relativo a**: padre | root.
- **Mostrar archivos sueltos** (sí / no — default sí).
- **Min size**: input numérico en MB (default 10). Bajar lo recalcula
  client-side (lo que hay en memoria); subir solo oculta.
- **Solo de tipo**: chips para cada `ExtCategory` (multi-select, off por
  defecto).

## Resaltado bidireccional

State compartido `selectedPath`:

- Click en treemap → setea `selectedPath` → fila correspondiente
  enscrolla y se resalta.
- Click en tree → setea `selectedPath` → leaf correspondiente del
  treemap glow.

Cuando coinciden, el header del Disk Analyzer muestra breadcrumbs:
`C:\\ › Users › joels › AppData` con cada segmento clicable.

## Atajos de teclado dentro del tree

- `↑` / `↓` movimiento entre filas visibles.
- `→` expand. `←` collapse o salta al padre.
- `Enter` → zoom en treemap.
- `Ctrl+C` copia path.
- `Ctrl+O` abre en Explorer del SO.
