# 06 — Ventana más grande

> **Severidad:** 🟢 P2 — preferencia del usuario.

## 1. Problema

El usuario reporta que la ventana se siente apretada. Hoy:

```
width:     1280
height:    800
minWidth:  1024
minHeight: 700
```

A `1280×800`, el sidebar de 220px + el contenido del PlanView se quedan
sin aire vertical. Las cards de "Listas / Bloqueadas / Permisos /
Omitidas" se apilan pero queda mucho espacio vacío bajo el botón
"Limpiar".

## 2. Causa raíz

`src-tauri/tauri.conf.json:13-25`:

```json
"windows": [
  {
    "label": "main",
    "title": "ClearTool",
    "width": 1280,
    "height": 800,
    "minWidth": 1024,
    "minHeight": 700,
    ...
  }
]
```

## 3. Fix propuesto

```json
"windows": [
  {
    "label": "main",
    "title": "ClearTool",
    "width": 1440,
    "height": 920,
    "minWidth": 1200,
    "minHeight": 800,
    "resizable": true,
    "decorations": false,
    "transparent": false,
    "shadow": true,
    "center": true
  }
]
```

Cambios:

| Campo | Antes | Después | Razón |
|---|---|---|---|
| `width` | 1280 | 1440 | Tamaño común en monitores 1080p y MacBook Pro 14" |
| `height` | 800 | 920 | Permite ver toda la página sin scroll en la mayoría de pantallas |
| `minWidth` | 1024 | 1200 | Por debajo de 1200, el sidebar + tablas se rompen |
| `minHeight` | 700 | 800 | El header + footer fijos comen ~120px |
| `center` | — | `true` | Centrar al abrir, más profesional |

### Persistencia del tamaño

Considerar el plugin `tauri-plugin-window-state` para recordar el
tamaño/posición entre lanzamientos:

```bash
cargo add tauri-plugin-window-state
npm install @tauri-apps/plugin-window-state
```

`src-tauri/src/lib.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_window_state::Builder::default().build())
    // ... resto ...
```

Eso guarda en `%APPDATA%\com.cleartool.app\window-state.json` y
restaura al siguiente arranque. Si la pantalla cambió (monitor
desconectado), Tauri ajusta automáticamente.

## 4. Criterio de done

- [ ] La ventana abre a `1440×920` centrada en pantalla.
- [ ] El usuario puede redimensionar entre `1200×800` y el máximo del
      monitor.
- [ ] Si se cierra a `1500×1000` y se reabre, se mantiene en
      `1500×1000` (con plugin activado).

## 5. Riesgos / efectos secundarios

- En monitores de 1366×768 (laptops viejos), `minHeight: 800` puede
  forzar scroll del shell de Windows. Mitigación: bajar a 768 si nos
  llegan reportes. Por ahora 800 es razonable porque el público de
  ClearTool 2026 está mayoritariamente en >= 1080p.
