# 06 — Treemap renderer

## Por qué canvas y no SVG

Un disco grande genera fácilmente 5–20 k nodos visibles. SVG con tantos
nodos es injugable (cada uno es un nodo DOM con reflow). Canvas 2D pinta
todo en un solo elemento, perfectly fluido.

## Librerías

- `d3-hierarchy` para el cálculo del layout (squarified treemap).
- Canvas 2D puro para el render.
- No usamos `d3-selection` ni nada DOM.

## Layout

```ts
function buildLayout(root, width, height): LayoutLeaf[] {
  const hier = d3h.hierarchy(root, d => d.children ?? null)
    .sum(d => isLeaf(d) ? d.sizeBytes : 0)
    .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

  d3h.treemap()
    .size([width, height])
    .paddingTop(node => node.depth === 0 ? 0 : 14)   // banda para nombre de carpeta
    .paddingInner(1)
    .round(true)(hier);

  return collectLeaves(hier, MIN_FRACTION_VISIBLE);
}
```

Diferencias vs MVP actual:

- `paddingTop` por profundidad → renderizamos un "header" para carpetas
  con el nombre encima de su área.
- `MIN_FRACTION_VISIBLE = 0.0008` (más permisivo, hasta el ~0.08 %).
- Cada leaf lleva su path completo para tooltip y navegación.

## Pintado

```ts
function drawTreemap(ctx, leaves, dpr, hoveredPath) {
  ctx.save();
  ctx.scale(dpr, dpr);
  for (const leaf of leaves) {
    const { x0, y0, x1, y1, data } = leaf;
    const w = x1 - x0, h = y1 - y0;
    if (w < 1 || h < 1) continue;

    // Fondo: gradiente vertical sutil para sensación 3D futurista.
    const grad = ctx.createLinearGradient(x0, y0, x0, y1);
    const baseColor = extColor(data);
    grad.addColorStop(0, lighten(baseColor, 0.10));
    grad.addColorStop(1, darken(baseColor, 0.18));
    ctx.fillStyle = grad;
    ctx.fillRect(x0, y0, w, h);

    // Borde 1px más oscuro.
    ctx.strokeStyle = darken(baseColor, 0.35);
    ctx.lineWidth = 1;
    ctx.strokeRect(x0 + 0.5, y0 + 0.5, w - 1, h - 1);

    // Glow si está hovered.
    if (data.path === hoveredPath) {
      ctx.shadowColor = baseColor;
      ctx.shadowBlur = 18;
      ctx.strokeStyle = "rgba(255,255,255,0.85)";
      ctx.lineWidth = 1.5;
      ctx.strokeRect(x0, y0, w, h);
      ctx.shadowBlur = 0;
    }

    // Texto.
    if (w > 38 && h > 16) drawLabel(ctx, leaf);
  }
  ctx.restore();
}
```

## Categorías y paleta

Cada extensión cae en una categoría (`ExtCategory`). Paleta:

| Categoría | Color base | Vibra |
|-----------|-----------|-------|
| `media` (video/audio) | `#a855f7` | purple-500 |
| `image` | `#06b6d4` | cyan-500 |
| `code` | `#22c55e` | green-500 |
| `docs` | `#3b82f6` | blue-500 |
| `archive` | `#f59e0b` | amber-500 |
| `executable` | `#ef4444` | red-500 |
| `database` | `#f43f5e` | rose-500 |
| `font` | `#ec4899` | pink-500 |
| `threeD` | `#14b8a6` | teal-500 |
| `other` | `#71717a` | zinc-500 |
| `dir` (header) | `#1f2937` | gray-800 |

La categorización corre **en backend** dentro de `ExtensionStat.category`.
El frontend tiene la misma tabla para colorear leaves del treemap (que
no llevan categoría serializada).

## Interacciones

- **Hover** → tooltip flotante con: name, path completo, size formateado,
  % del root, % del padre, fileCount, last_modified relativo
  ("hace 3 días").
- **Click izquierdo** sobre carpeta con hijos → zoom in (push a navStack).
- **Click derecho** → menú: "Abrir en Explorer del SO", "Copiar ruta",
  "Buscar similares por extensión".
- **Backspace** o botón ← → pop del navStack.
- **Doble click sobre el fondo** → reset al root.

## Hit-test

Iteración invertida sobre las leaves (las últimas son las más pequeñas
por encima de las grandes). Para 20 k leaves es < 1 ms.

## Resize

`ResizeObserver` con debounce 200 ms. Recalcula layout y repinta.
Mantenemos el `path` actualmente seleccionado durante el resize.
