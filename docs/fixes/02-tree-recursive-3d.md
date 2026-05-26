# Fix 02 — Árbol del explorador: hijos recursivos + propuesta estilos 3D

## Síntoma
Al hacer clic en una carpeta, el chevron giraba pero no aparecían subcarpetas. El árbol solo mostraba el nivel raíz.

## Causa raíz (3 puntos combinados)

### 1. `expandNode` never populated children
`use-explorer-tree.ts` → `expandNode` llamaba a `scanTree` (que emite eventos). Pero esos eventos `explorer:node` iban al listener raíz de `explorer-page.tsx`, que los añadía a `state.nodes` (nivel raíz), mezclándolos con las entradas de la raíz. Además, al terminar, `expandNode` forzaba `children: [] as TreeNode[]` (array vacío literal).

### 2. `TreeView` no pasaba `byPath` a `TreeRow`
`tree-view.tsx` renderizaba `<TreeRow>` sin pasarle ni los hijos ni el mapa de estado. El `children` prop del componente nunca se rellenaba.

### 3. `TreeRow` no era recursivo
Tenía `{expanded && isDir && children}` pero `children` siempre era `undefined` porque nadie lo pasaba.

## Fix aplicado

### Nuevo comando Rust `list_dir`
```rust
// src-tauri/src/ipc/explorer.rs
#[tauri::command]
pub async fn list_dir(path: String, follow_reparse_points: bool) -> AppResult<Vec<TreeNode>> {
    domain::explorer::list_top_level(&path, follow_reparse_points)
}
```
Devuelve hijos directos **sin eventos**, sincrónico desde el punto de vista del frontend.

### `use-explorer-tree.ts` — expandNode reescrito
```ts
const expandNode = useCallback(async (path: string) => {
    setState(prev => { byPath.set(path, { children: "loading", ... }); });
    const children = await listDir(path, false);
    setState(prev => { byPath.set(path, { children: sorted, ... }); });
}, []);
```

### `TreeRow` ahora recursivo
```tsx
// Acepta byPath como prop, lee sus propios hijos de byPath.get(node.path)
{expanded && isDir && Array.isArray(nodeState?.children) &&
  nodeState.children.map(child => (
    <TreeRow key={child.path} node={child} depth={depth + 1} onExpand={onExpand} byPath={byPath} />
  ))
}
```

---

## Propuesta de estilos 3D para el árbol

### Opción A — CSS puro (recomendada, zero deps)

Aplica perspectiva y desplazamiento Z basado en la profundidad. Sin librerias extra.

```css
/* globals.css o tailwind plugin */
.tree-wrapper {
  perspective: 1200px;
  perspective-origin: 50% 20%;
}

.tree-row {
  transform-style: preserve-3d;
  transition: transform 120ms ease, background 120ms ease;
}

/* depth-based Z offset: cada nivel se aleja 6px en Z */
.tree-row[data-depth="0"] { transform: translateZ(0px); }
.tree-row[data-depth="1"] { transform: translateZ(-6px) translateX(0); }
.tree-row[data-depth="2"] { transform: translateZ(-12px); }
.tree-row[data-depth="3"] { transform: translateZ(-18px); }

/* hover: levita hacia el usuario */
.tree-row:hover {
  transform: translateZ(8px) scale(1.002);
  background: hsl(var(--signal-cyan) / 0.06);
  box-shadow: 0 2px 16px hsl(var(--signal-cyan) / 0.1);
}

/* carpeta expandida: efecto de "apertura" con rotateX */
.tree-row[data-expanded="true"] {
  transform: translateZ(4px) rotateX(-1deg);
}
```

En `TreeRow`:
```tsx
<tr
  data-depth={depth}
  data-expanded={expanded}
  className="tree-row border-b border-border/30 cursor-pointer group"
  onClick={handleClick}
>
```

En `TreeView`:
```tsx
<div className="tree-wrapper flex-1 overflow-auto border border-border rounded-lg">
  <table ...>
```

### Opción B — Glassmorphism por nivel de profundidad

Cada nivel añade un fondo con `backdrop-blur` y opacidad decreciente, simulando profundidad "de vidrio":

```tsx
// TreeRow: background basado en depth
const depthBg = [
  "bg-surface-1/0",
  "bg-signal-cyan/3 backdrop-blur-[1px]",
  "bg-signal-cyan/5 backdrop-blur-[2px]",
  "bg-signal-cyan/7 backdrop-blur-[3px]",
][Math.min(depth, 3)];

<tr className={`${depthBg} border-b border-border/30 ...`}>
```

### Opción C — Three.js/R3F (futura, separada)

Si en algún momento se quiere un árbol completamente 3D (como un grafo de árbol orbital), usar `@react-three/fiber` + `@react-three/drei`:

```tsx
// src/features/explorer/tree-view-3d.tsx (componente alternativo)
// Nodos como esferas, ramas como tubes, cámara orbital con OrbitControls
// Solo activar como "modo 3D" opcional — el modo tabla sigue siendo el por defecto
```

Esta opción requiere cambios no triviales de UX (navegación por clic en nodo 3D, tooltip de info). Recomendado como feature separada, no como fix.

### Opción recomendada para producción
Mezclar **A + B**: CSS perspective + glassmorphism por depth. Costo: ~20 líneas CSS + 2 atributos data-* en `TreeRow`. Sin dependencias extra.

## Archivos tocados (fix recursivo)
- `src-tauri/src/ipc/explorer.rs` — nuevo comando `list_dir`
- `src-tauri/src/lib.rs` — registro del comando
- `src/api/client.ts` — wrapper `listDir`
- `src/features/explorer/use-explorer-tree.ts` — `expandNode` reescrito
- `src/features/explorer/tree-view.tsx` — prop `byPath`
- `src/features/explorer/tree-row.tsx` — renderizado recursivo
- `src/features/explorer/explorer-page.tsx` — pasa `byPath` a `TreeView`
