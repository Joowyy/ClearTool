# Disk Analyzer — Documentación

Este directorio contiene la especificación fragmentada en bloques del módulo
**Disk Analyzer** de ClearTool. El módulo sustituye al antiguo "Explorador",
que queda eliminado (ver `02-explorer-removal.md`).

## Posicionamiento del módulo

Disk Analyzer es el **segundo módulo más importante** de ClearTool después
del Caché. Resuelve la pregunta "¿qué se está comiendo mi disco?" combinando
tres lecturas complementarias del mismo escaneo:

1. **Treemap** estilo WinDirStat — bloques rectangulares coloreados por
   categoría de archivo, area proporcional al peso.
2. **Árbol jerárquico** ordenado por mayor peso, con porcentaje del padre y
   conteo de subarchivos visibles inline.
3. **Bloques de stats** — tarjetas con totales (size, files, dirs), top
   extensiones, distribución por antigüedad, carpetas más pesadas.

La premisa de diseño es **WinDirStat pero futurista**: el mismo treemap,
pero con tipografía moderna, glow sutil, paleta de marca, métricas
contextualizadas y selector de disco real (sin escribir rutas a mano).

## Mapa de bloques (lee en orden)

| # | Archivo | Qué cubre |
|---|---------|-----------|
| 00 | `00-overview.md` | Objetivo, audiencia, no-objetivos, métricas de éxito. |
| 01 | `01-disk-enumeration.md` | Cómo enumerar discos del sistema y popular el selector. |
| 02 | `02-explorer-removal.md` | Plan de borrado del módulo Explorer (UI + backend + residuos). |
| 03 | `03-scan-engine.md` | Algoritmo de escaneo: recorrido, streaming, cancelación. |
| 04 | `04-data-model.md` | DTOs Rust ↔ TS (TreemapNode, DiskAnalysisReport, DriveListing). |
| 05 | `05-ipc-contract.md` | Comandos Tauri, payloads de eventos, errores. |
| 06 | `06-treemap-render.md` | Cómo se dibuja el treemap canvas, hit-test, navegación. |
| 07 | `07-tree-hierarchy.md` | Árbol jerárquico ordenado por peso, lazy-expand, bars. |
| 08 | `08-stats-blocks.md` | Bloques de stats (totales, top ext, edad, carpetas top). |
| 09 | `09-ui-layout.md` | Layout 3 zonas, futurismo, tokens visuales, estados. |
| 10 | `10-performance.md` | Presupuestos, paralelismo, throttling, memoria. |
| 11 | `11-edge-cases.md` | Symlinks, reparse points, permisos, paths >MAX_PATH, drives offline. |
| 12 | `12-roadmap.md` | Iteración 1 (MVP) → iteración 3 (avanzado). |

## Decisiones cerradas (no negociar sin nueva justificación)

- **Sin tamaño físico (clúster) en MVP.** Solo lógico (`metadata.len()`).
  El extra de leer `GetCompressedFileSizeW` por archivo cuesta demasiado
  IO y no compensa en MVP. Se añade en iteración 2 como opción.
- **Sin caché de escaneos a disco.** Cada análisis vive en memoria del
  proceso. Persistir snapshots queda fuera del MVP.
- **No seguir reparse points por defecto.** Evita ciclos y duplicados (el
  caso clásico: `C:\Users\All Users` → `C:\ProgramData`). El usuario puede
  habilitarlo desde el header.
- **Profundidad inicial = ilimitada con throttling.** WinDirStat va al
  fondo; nosotros también, pero filtramos para emisión (`min_size_mb`)
  para no inundar el frontend.
- **Streaming de progreso vía evento Tauri**, no respuesta única. Disco
  típico (200 GB, 1M archivos) tarda minutos; UI debe pintar lo escaneado.

## Cómo abordar el trabajo

1. Lee `00-overview.md` y `02-explorer-removal.md` para entender el
   contrato del cambio.
2. Borra Explorer (paso destructivo, primero).
3. Implementa enumeración de discos (`01`) — desbloquea todo lo demás.
4. Refuerza modelos y comandos (`04`, `05`).
5. Reescribe scan engine (`03`).
6. Recompone UI por capas: stats blocks → tree → treemap (`06`-`09`).
7. Cierra con `10-performance.md` y `11-edge-cases.md`.
