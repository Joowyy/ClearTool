# 06 — UI / UX

## Principio rector

ClearTool es una **herramienta de poder**, no un asistente para principiantes. La UI no oculta complejidad ni romantiza el "borrar todo de un click". Antes de cada operación destructiva muestra **qué exactamente** va a pasar, **por qué** podría salir mal y **cómo** revertirlo. Diseño tipo "panel de control profesional", no marketing.

## Tono

- Idioma por defecto: **español**. Inglés disponible en settings.
- Voz: directa, técnica, sin emojis ni marketing. "Vas a quitar 12 paquetes" en lugar de "✨ ¡Limpiando tu PC!".
- Errores: explicativos. "No se pudo detener el servicio `wuauserv` porque el proceso `<PID>` lo mantiene activo. Reintentar | Forzar | Saltar."

## Layout global

```
┌─────────────────────────────────────────────────────────────┐
│  [Logo] ClearTool       [chip: build] [chip: admin/limited] │ ← Topbar (40px)
├──────────┬──────────────────────────────────────────────────┤
│          │                                                  │
│ Sidebar  │   Page content                                   │
│ (220px)  │                                                  │
│          │                                                  │
│ • Home   │                                                  │
│ • Explorer                                                  │
│ • Cache  │                                                  │
│ • Debloat│                                                  │
│ • Services                                                  │
│ • Registry                                                  │
│ • Restore│                                                  │
│ • Audit  │                                                  │
│ • Settings                                                  │
│          │                                                  │
└──────────┴──────────────────────────────────────────────────┘
```

- Topbar siempre visible. El chip "limited" se vuelve "admin" en verde si is_elevated.
- Sidebar colapsable a iconos en viewport < 1024 px.
- En el footer del contenido, breadcrumbs cuando la página tiene jerarquía (`/explorer > C:\ > Users > jowy`).

## Sistema visual

### Tokens

```css
:root {
  --bg-base: #0b0d10;
  --bg-elevated: #14171c;
  --bg-overlay: #1a1f26;
  --border: #232a33;
  --border-strong: #2e3742;
  --text-primary: #e6edf3;
  --text-secondary: #9aa6b2;
  --text-muted: #6b7684;
  --accent: #4f8cff;        /* azul ClearTool */
  --accent-press: #3a72e0;
  --success: #4ade80;
  --warning: #fbbf24;
  --danger: #f87171;
  --danger-press: #dc2626;
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 14px;
  --shadow-1: 0 1px 0 rgba(255,255,255,0.04) inset, 0 1px 2px rgba(0,0,0,0.3);
}
[data-theme="light"] { /* … invert … */ }
```

Tema por defecto: **dark**. Light se ofrece pero no es donde vive el producto.

### Tipografía

- UI: `Inter` (system fallback `Segoe UI Variable`, `Segoe UI`, sans-serif).
- Mono: `JetBrains Mono` (system fallback `Cascadia Mono`, `Consolas`).
- Escalas:
  - `text-xs` 11/14, `text-sm` 13/18, `text-base` 14/20, `text-md` 15/22, `text-lg` 18/26, `text-xl` 22/30, `text-2xl` 28/36.
- Solo se usan tres pesos: 400, 500, 700.

### Iconografía

- `lucide-react`. Tamaño base 16 px, 18 px en CTAs grandes.
- Para risk: `circle-dot` (low, verde), `triangle-alert` (medium, ámbar), `octagon-alert` (high, rojo).

### Componentes base (shadcn/ui)

- `Button` con variantes: `default`, `secondary`, `ghost`, `destructive`, `outline`.
- `DataTable` virtualizada (TanStack table + virtual). Filas 36 px, denso.
- `Dialog` con `DialogTabs` para confirmaciones destructivas.
- `Tooltip` con delay 400 ms.
- `Tabs` con underline ámbar en activo.
- `Badge` chips de categoría/risk.
- `Switch` para tweaks individuales.
- `Progress` linear y radial.
- `Toast` arriba-derecha, durations: success 3 s, error 8 s, warning 5 s.

## Páginas

### `/` Home

- Card "Sistema": versión Windows, build, drive principal con barra (libre / total), última operación destructiva (link a audit).
- Card "Estado": chip admin/limited; si `limited`, CTA "Reiniciar como admin".
- Card "Restore Points": último punto creado (description + fecha) + atajo "Crear punto manual".
- Cuatro tiles grandes: `Explorer`, `Cache`, `Debloat`, `Services` con descripción de 1 línea.

### `/explorer`

- Toolbar: input "ruta" + botón Go; selector de drive; toggle "ocultos"; toggle "junctions"; input "tamaño mínimo"; botón "Cancelar escaneo" cuando hay scan activo.
- `<FileTree>` ocupa toda la altura. Scroll virtualizado.
- Footer: stats `12 543 archivos · 4 822 dirs · 187 GB en C:\`.

### `/cache`

- Filter bar: por categoría (chips toggle), por riesgo, "solo requiere admin".
- DataTable: nombre, categoría, tamaño, riesgo, requiere-admin, bloqueadores.
- Toolbar derecha: `Escanear seleccionadas`, `Limpiar seleccionadas` (destructive).
- Vacío: "Selecciona ubicaciones a escanear. Puedes empezar con `Limpiezas seguras` (preset risk=low)."

### `/debloat`

- Banner explicativo arriba: "Perfil **Total** activo. Marca todo lo que ClearTool considera no esencial. Revisa antes de aplicar."
- Tres botones de preset (chip toggle): `Mínimo` · `Recomendado` · **`Total` (default)**.
- Filter bar: categoría (ConsumerApp, Ai, MsConsumer, MsCore, Telemetry, Oem); riesgo; "solo instalados"; "solo reversibles".
- Checkboxes globales: `Aplicar policies registry`, `Deshabilitar servicios asociados` (default ambos `on`).
- DataTable de bloatware con detalle expandible (consequences + reversal).
- CTA principal: `Aplicar (N entradas)` — destructive.

### `/services`

- Filter bar + búsqueda libre.
- DataTable con columnas: nombre, displayName, status (chip), startType (chip), categoría, recomendación.
- Por fila menú con: Start, Stop, Restart, Set start type → submenú.
- Toolbar: presets `TelemetryOff`, `XboxOff`, `FullDebloat`.
- Detalle al click en fila (drawer derecho): descripción, dependencias entrantes/salientes, ruta del binario, account.

### `/registry`

- Filter por categoría y riesgo.
- DataTable con switch on/off por fila (refleja `TweakState`).
- Tooltip con `description` y `consequences` por fila.
- Toolbar: `Aplicar paquete recomendado`, `Aplicar selección`, `Revertir selección`.
- Footer: "Tweaks que requieren reiniciar Explorer: N. [Reiniciar Explorer]".

### `/restore`

- Sección 1: "Estado de System Restore" (vía `ensure_restore_enabled`). Si algo falta, banner ámbar con CTA "Habilitar (3 pasos)".
- Sección 2: tabla de restore points (sequence, descripción, fecha, tipo).
- Acciones por fila: `Restaurar (reinicia tu PC)` con confirm.
- Footer: botón "Crear punto manual" + input descripción.

### `/audit`

- Filter por kind, fecha (last 7d / 30d / all), "solo reversibles".
- DataTable con: timestamp, kind, payload_summary, restore_point_seq, reversible (chip), botón "Deshacer".
- Detalle al click → drawer con JSON formateado y diff `before/after` cuando aplica.

### `/settings`

- Tema, idioma (es/en), nivel de log, ruta del audit.jsonl, retención (días/MB).
- Toggle "abrir como admin siempre" (toca el manifest del shortcut).
- Toggle "preset por defecto en `/debloat`" (Total | Recomendado | Mínimo).
- Botón "Exportar audit log" / "Borrar audit log".

## Flujos críticos

### Flujo "Limpieza de caché"

```
Usuario: marca 5 ubicaciones, click `Limpiar seleccionadas`.
  └─ Modal <ConfirmDestructive>
        Tab "Resumen":
          Vas a procesar 5 ubicaciones.
          Espacio libre estimado: 4.3 GB.
          Servicios afectados: wuauserv, bits (se restauran al final).
          Restore point: ClearTool: clean cache run <id>
        Tab "Detalle":
          Lista de 5 entradas con paths, tamaños, filtros.
        Tab "Riesgos":
          • Próxima sesión Windows Update re-descargará pendientes.
          • Logs CBS se truncarán; troubleshooting de Windows Update se complica.
        Tab "Reversa":
          El restore point te permite revertir.
          Borrar archivos no es reversible archivo a archivo.
        Checkbox "He revisado los detalles" (deshabilita Aplicar hasta marcar)
        Botones: [Cancelar] [Dry-run] [Aplicar]
  └─ Click Aplicar
  └─ Pantalla "Operación en curso" con barra global y per-location.
  └─ Toast verde: "4.1 GB liberados · 5 ubicaciones".
  └─ Redirección a `/restore` mostrando el punto creado.
```

### Flujo "Debloat Total"

```
Usuario: entra a `/debloat`. Default = Total, todo marcado.
  └─ Banner ámbar: "Estás a punto de tocar el ecosistema Microsoft completo.
                   Revisa categoría 'MsCore' antes de aplicar."
  └─ Click `Aplicar (78 entradas)`
  └─ Modal <ConfirmDestructive>
        Tab Resumen: 78 entradas, 4.2 GB estimados, restore point a crear.
        Tab Detalle: lista paginada, agrupada por categoría.
        Tab Riesgos: warnings rojos por Edge, Store, OneDrive, Cortana.
        Tab Reversa: lista de qué se puede reinstalar via Store, qué requiere
                     instaladores oficiales, qué necesita restore-point.
        Checkbox "He leído los riesgos" + segundo checkbox específico
                 "Entiendo que sin Microsoft Store, reinstalar apps requiere WinGet o
                  scripts manuales".
  └─ Click Aplicar
  └─ Pantalla "Operación en curso":
        - Barra global: N de 78 entradas.
        - Tabla viva con cada entrada y sus pasos (chip OK/Falló por step).
        - Botón "Cancelar" disabled tras la primera operación que toca SO
          (PolicyOnly se puede cancelar entre entradas; mid-Appx no).
  └─ Toast verde + redirección a `/restore`.
```

### Flujo "Modo limitado"

- App levanta. UAC fue cancelado. `is_elevated() = false`.
- Topbar muestra chip rojo "Modo limitado".
- Banner persistente debajo del topbar:
  > "Algunas funciones requieren permisos elevados. **[Reiniciar como admin]**"
- Páginas destructivas se cargan con sus tablas pero los botones primarios están disabled con tooltip "Requiere admin. Reinicia con privilegios".
- Únicas funciones plenamente disponibles: `/explorer` (lectura) y `/cache` (escaneo, no limpieza salvo entradas user-temp).

## Microcopy clave

| Situación | Copy |
|---|---|
| Confirm destructiva | "He revisado los detalles" |
| Dry-run en lugar de aplicar | "Simular" |
| Sin admin | "Reiniciar como administrador" |
| Restore unavailable | "System Restore no está disponible. Habilítalo en 3 pasos para garantizar reversibilidad." |
| Operación parcial | "Algunas operaciones fallaron. Revisa el reporte." |
| Operación completa | "Listo. Liberados X · Sin errores." |
| Cancelando mid-run | "Cancelando tras la operación actual…" (no abortar mid-side-effect) |

## Accesibilidad

- Contraste mínimo AAA en texto principal (`#e6edf3` sobre `#0b0d10` → 14.8:1).
- Targets clickables ≥ 32×32 px (los rows de tabla son denso 36 px).
- Navegación completa con teclado:
  - Tab/Shift-Tab entre zonas.
  - Flechas dentro de tabla.
  - Espacio = toggle checkbox / switch.
  - Enter = abrir detalle.
  - `Cmd/Ctrl+K` = paleta de comandos (futuro).
- Focus rings visibles en alto contraste (no `outline: none`).
- Screen-reader: cada DataTable expone roles correctos; los chips de status tienen `aria-label` con la palabra completa.
- Reduced motion: si `prefers-reduced-motion`, animaciones de transición se desactivan.

## Estados vacíos

- Cada tabla con datos vacíos muestra una ilustración monocroma + 1 línea + 1 CTA. No se queda en blanco.
- Ej. `/cache` vacío: "Aún no has escaneado nada. **[Escanear ubicaciones seguras]**".

## Responsive

ClearTool es desktop. Tamaños soportados: 1024×700 mínimo, 1920×1080 ideal, 2560+ (paneles de detalle se ensanchan, tablas no).

## Animaciones

- Transiciones de tabs: 120 ms ease-out.
- Chevrons de expand: 150 ms.
- Toast in/out: 200 ms.
- Sin scroll snap. Sin parallax. Sin "shimmer" salvo skeletons reales con datos en vuelo.

## Performance UX

- Páginas destructivas no bloquean: el spinner en CTA aparece a partir de los 200 ms; antes de eso, el botón solo se deshabilita.
- Eventos de progreso no actualizan la UI más rápido que 60 fps (throttle a 16 ms).
- Tabs en modal de confirmación se montan lazy: el JSON detallado de un debloat de 78 entradas no se renderiza hasta abrir esa tab.
