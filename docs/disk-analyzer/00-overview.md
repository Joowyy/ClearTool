# 00 — Overview

## Objetivo

Reescribir el módulo "Disco" para que un usuario pueda, en menos de 60 s
desde que abre la pestaña:

1. Ver todos los discos físicos/lógicos del sistema sin escribir rutas.
2. Elegir uno y lanzar un análisis completo.
3. Entender en una sola pantalla **dónde** está su espacio: por
   carpeta, por extensión y visualmente como treemap.
4. Profundizar (zoom) hasta el archivo concreto que está pesando.

## Usuario tipo

- Conoce WinDirStat o TreeSize y espera una experiencia similar.
- Quiere identificar carpetas inflables (Downloads, AppData, node_modules,
  Steam libraries, OneDrive offline, etc.) para tomar decisión informada.
- No quiere que la app le borre nada por sí sola — el Disk Analyzer es
  estrictamente **read-only**. La eliminación se delega al usuario vía
  Explorer del SO, o a otros módulos (Cache, Debloat, Inventory).

## No-objetivos

- **No borrar archivos**. Disk Analyzer es de lectura.
- **No detectar duplicados** en MVP (lo cubrimos en iteración 3).
- **No análisis programado en background**. Solo análisis on-demand.
- **No multi-disk simultáneo**. El usuario elige uno cada vez.
- **No exportar reportes** en MVP (CSV/JSON queda para iteración 2).

## Métricas de éxito (DoD)

- [ ] El usuario puede elegir cualquier disco del sistema desde un
      selector sin escribir nada.
- [ ] Tras pulsar "Analizar", aparece progreso incremental (barra de
      avance o contador de bytes/archivos vistos) antes de 500 ms.
- [ ] Un disco de 200 GB se escanea en < 2 min en hardware típico.
- [ ] Los tres paneles (stats, tree, treemap) son coherentes: si el
      treemap dice 12 GB para `node_modules`, el árbol también.
- [ ] Hover sobre cualquier elemento muestra: path completo, size, %,
      último acceso/modificación, conteo de archivos.
- [ ] Click sobre una carpeta del treemap o del árbol hace zoom in.
- [ ] El módulo Explorer original ha desaparecido por completo.

## Restricciones técnicas

- Backend: Rust 2021, dentro de `src-tauri/`. Nada nuevo en `Cargo.toml`
  salvo lo imprescindible (ya tenemos `walkdir`, `windows`, `serde`,
  `tokio` vía Tauri).
- Frontend: React 18 + TS estricto + Tailwind + framer-motion + d3-hierarchy
  (ya en `package.json`).
- Sin dependencias externas nuevas — confirmar en `12-roadmap.md` antes
  de añadir cualquier crate.
- Sigue la convención del proyecto: comentarios en español, identificadores
  en inglés, errores `thiserror` en libs.
