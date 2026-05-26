# 07 — Disk Analyzer (Treemap) (M4, P2)

**Objetivo**: visualización treemap de uso de disco al estilo WinDirStat, con zoom + acciones contextuales.

**Spec de referencia**: `.claude/specs/v2/05-new-modules.md` §5.3.

## Pasos

1. [01 — Backend: `build_treemap_data`](01-backend-treemap-data.md)
2. [02 — Frontend canvas treemap con d3-hierarchy](02-frontend-canvas-treemap.md)
3. [03 — Zoom + acciones contextuales](03-zoom-y-acciones.md)

## Tech stack decidido

- Canvas 2D + d3-hierarchy (no R3F, no SVG, no librería treemap completa).
- Pros: <50KB extra, performance hasta 10k nodos, control fino.

## Criterio de done

- [ ] Treemap navegable de `C:\` con 4 niveles de profundidad.
- [ ] Color por extensión.
- [ ] Hover tooltip con path + tamaño.
- [ ] Click zoom in, doble click → "Abrir" o "Eliminar carpeta" con confirmación.
- [ ] Performance: render <500ms para 5k nodos.

## Tiempo total

15-18 horas (la pieza más visual del módulo).
