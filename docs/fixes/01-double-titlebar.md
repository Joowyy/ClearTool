# Fix 01 — Doble titlebar (nativa + HTML)

## Síntoma
La app mostraba dos barras superiores: la titlebar nativa de Windows encima de nuestra titlebar HTML personalizada.

## Causa raíz
`tauri.conf.json` tenía `"decorations": true`, lo que hace que Tauri dibuje el chrome nativo de Windows (titlebar + bordes) además de nuestro `<Titlebar>` HTML. El resultado es que el usuario ve la barra nativa de Windows arriba y nuestra barra cyan debajo.

## Fix
```json
// src-tauri/tauri.conf.json
"decorations": false   // antes: true
```

Con `decorations: false`, Tauri crea una ventana sin chrome nativo. La app queda borderless. Los botones min/max/close son los nuestros (ya en `<WindowControls>`). El drag se maneja vía `data-tauri-drag-region` en `<Titlebar>`.

**¿Qué se pierde?** El border de resize nativo de 4 px. En Windows 11, Tauri conserva igualmente los resize handles invisibles en los bordes. Las Snap Layouts siguen funcionando con Win+Z / Win+Arrow.

## Archivos tocados
- `src-tauri/tauri.conf.json` — `app.windows[0].decorations: false`
