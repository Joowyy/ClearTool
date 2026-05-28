# 08 — Bloques de stats

Encima del treemap, el panel principal arranca con 4 tarjetas (`StatsBlock`)
y un set de paneles secundarios con tablas tops. Todo es derivado del
`DiskAnalysisReport`.

## Tarjeta 1 — Uso del disco

```
┌─────────────────────────────────┐
│ Uso del disco                   │
│                                 │
│   ◐  786.5 GB / 1.0 TB usados   │
│       (78.6 %)                  │
│                                 │
│   213.5 GB libres               │
│   Analizado: 728.3 GB           │
└─────────────────────────────────┘
```

Donut chart minimalista (radial-progress de Tailwind o SVG manual). Si el
"analizado" difiere del "usado", muestra ambos — la diferencia normalmente
es System Volume Information + Recycle.Bin + permisos denegados.

## Tarjeta 2 — Totales del escaneo

```
┌─────────────────────────────────┐
│ Contenido analizado             │
│                                 │
│   📁 124.832 carpetas            │
│   📄 1.247.918 archivos          │
│   ⏱  Escaneo: 1 min 42 s         │
│   ⚠  3 errores                  │
└─────────────────────────────────┘
```

## Tarjeta 3 — Distribución por antigüedad

Barras horizontales apiladas con el % de bytes en cada cubeta:

```
┌─────────────────────────────────┐
│ Antigüedad de los archivos      │
│                                 │
│   < 7 días     ████░░░░░░  12 % │
│   < 30 días    ██████░░░░  18 % │
│   < 90 días    ███░░░░░░░   8 % │
│   < 1 año      ███████░░░  21 % │
│   < 5 años     ████████░░  25 % │
│   > 5 años     █████░░░░░  16 % │
└─────────────────────────────────┘
```

Útil para detectar archivos viejos olvidados (típicamente borrables).

## Tarjeta 4 — Categorías dominantes

Donut o stacked bar con `topExtensions` agrupadas por `ExtCategory`. Una
porción por categoría; tooltip muestra extensiones más grandes dentro de
esa categoría.

## Panel — Top 20 carpetas

Tabla densa de `largestFolders`:

| % | Carpeta | Tamaño | Archivos | Última mod. |
|---|---------|--------|----------|-------------|
| 12.4 % | `C:\Users\joels\AppData\Roaming\node_modules` | 92.1 GB | 824.011 | hace 1 día |

Filas clicables → zoom al path en el treemap.

## Panel — Top 20 archivos

Igual pero `largestFiles`. Útil para encontrar el ISO de 60 GB que quedó
en `Downloads`.

## Panel — Top 20 extensiones

```
| % | Ext  | Categoría   | Tamaño | Archivos |
|---|------|-------------|--------|----------|
| 22.1 % | .mkv | media   | 161 GB | 218      |
```

## Estilo visual ("futurista")

- Tarjetas con borde 1px de gradiente `from-cyan-500/30 via-blue-500/10 to-transparent`.
- Fondo `panel-raised` (ya disponible en design tokens del proyecto).
- Tipografía: nombres en `font-medium`, números con tabular-nums + mono.
- Animación de entrada: stagger 60 ms, fade + translate-y-2.
- Hover: borde brilla un tick más (cambia opacidad del gradiente de 30→60).
- Iconos lucide-react: `HardDrive`, `Files`, `Clock`, `PieChart`,
  `FolderTree`, `FileBarChart`.

## Estado vacío vs. cargando

- **Sin escaneo**: tarjetas en esqueleto translúcido con un placeholder
  "Selecciona un disco para empezar".
- **Escaneando**: tarjetas en modo "live" — los contadores se incrementan
  con el evento `disk:progress`. La donut crece mientras el `bytesScanned`
  aumenta.
- **Completado**: valores finales con micro-animación de cierre.
