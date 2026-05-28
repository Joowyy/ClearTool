# 12 — Roadmap

## Iteración 1 (MVP — esta entrega)

- [x] Specs en `/docs/disk-analyzer/` por bloques.
- [ ] Borrado completo del módulo Explorer.
- [ ] `list_drives` + tipos + cliente TS.
- [ ] `TreemapNode` extendido + `DiskAnalysisReport`.
- [ ] Reescritura de `domain::disk::build_treemap` con streaming de
      progreso y cómputo de stats.
- [ ] Comando `cancel_disk_scan`.
- [ ] Componentes UI nuevos (`DriveSelector`, `Breadcrumbs`, `StatsBlocks`,
      `HierarchyTree`, `TreemapCanvas` refactor, `TopFolders/Files/ExtensionsTable`).
- [ ] Layout 3-zonas con grid CSS.
- [ ] Resaltado bidireccional tree ↔ treemap.
- [ ] Hooks `useDiskScan` + eventos Tauri.
- [ ] Validación de build (cargo + tsc).

## Iteración 2 (post-MVP)

- [ ] Tamaño físico real (`GetCompressedFileSizeW`) opcional desde el header.
- [ ] Detección de OneDrive cloud y archivos sparse.
- [ ] Paralelismo con `rayon` en el walk del primer nivel.
- [ ] Export del report a JSON / CSV.
- [ ] Refresco automático en eventos `WM_DEVICECHANGE`.
- [ ] Pestaña "Por extensión" en el panel principal con tabla pivotada.
- [ ] Filtro global por antigüedad (`< 7 días`, etc.) que dim los bloques
      del treemap fuera del filtro.

## Iteración 3 (avanzado)

- [ ] Detección de duplicados (hash incremental + matching por tamaño).
- [ ] Comparación de dos snapshots (qué creció).
- [ ] Pestaña "Salud del disco" con SMART (vía WMI).
- [ ] Integración con el módulo Caché: si Disk Analyzer detecta un
      directorio que coincide con una ubicación del catálogo de caché,
      mostrar un atajo "Limpiar con Caché".
- [ ] Integración con Inventory: si un directorio del top 20 corresponde
      a una app instalada, ofrecer "Desinstalar con Inventory".

## Decisiones a tomar

- ¿Movemos `NodeKind` definitivamente a `models/disk.rs`? — sí.
- ¿Quitamos `tauri-plugin-fs` si Explorer ya no lo necesita? — pendiente
  de verificar que ningún otro módulo lo use; probable que sí (cache).
- ¿Conservar `last_modified` como ISO string o cambiar a `u64` ms? — ISO
  string es lo que ya usa el resto del proyecto.

## Dependencias del paquete

- Nada nuevo en MVP. `d3-hierarchy`, `framer-motion`, `lucide-react`,
  `@tanstack/react-virtual` ya están.
- Verificar con `npm ls @tanstack/react-virtual` que está presente. Si
  no, añadirlo (es la única dep externa probable).

## Tests

- Unit:
  - `enumerate_drives()` smoke test (que retorne al menos C:).
  - `categorize_extension("mkv")` → `Media`, etc.
  - `compute_age_bucket(ts)` cubre las 6 cubetas.
- Integration (en VM con dataset conocido):
  - Escanear un dir con 100 archivos de tamaños conocidos y verificar
    `totalBytes`, `totalFiles`, top.
  - Cancelación: lanzar, cancelar al 50 %, asegurar que el comando
    devuelve `Cancelled` y no panickea.

## Pendientes de la sesión

Cuando esta entrega cierre, actualizar `HISTORIAL-SESIONES.md` con
resumen y `CLAUDE.md` con la decisión arquitectónica "Disk Analyzer
sustituye a Explorer; eliminamos el módulo viejo".
