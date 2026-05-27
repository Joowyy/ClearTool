# 09 — Log más legible, sin emojis decorativos

> **Severidad:** 🟢 P2 — preferencia del usuario.
> _"que sea más legible con una fuente clara y concisa sin emotes,
> como mucho el de error y tick para saber si pasó o no alguna
> carpeta"._
> **Modelo:** Qwen 3.6 Plus.

## 1. Problema

`src/features/cache-cleaner/clean-console-log.tsx:10-15` define 4
iconos con color para cada nivel:

```ts
const ICONS = {
  info: { Icon: Activity, className: "text-sky-400" },
  success: { Icon: CheckCircle2, className: "text-emerald-400" },
  warn: { Icon: AlertTriangle, className: "text-amber-400" },
  error: { Icon: XCircle, className: "text-rose-400" },
} as const;
```

Cuatro iconos distintos por línea es ruido visual. El usuario pide
sólo:

- **✓** para "pasó" (success).
- **✗** para "no pasó" (error).
- Una marca neutra para info (un punto, un guion, o nada).
- **warn** mantiene un icono distinto pero discreto (triángulo
  pequeño).

Además, la fuente actual `font-mono text-xs` con `text-ink-secondary`
contrasta poco con el fondo. Subir un escalón de tamaño + un punto de
contraste hace el log mucho más cómodo de seguir.

## 2. Fix propuesto

### 2.1 Reducir el set de iconos

`src/features/cache-cleaner/clean-console-log.tsx:1-15`:

```tsx
import { useEffect, useRef, useState } from "react";
import { Check, X, AlertTriangle } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import type { CleanLogLinePayload } from "../../api/events";

interface Props {
  log: CleanLogLinePayload[];
}

const LEVEL = {
  info:    { mark: <span aria-hidden className="text-ink-muted">·</span>, className: "text-ink-secondary" },
  success: { mark: <Check className="h-3.5 w-3.5 text-emerald-400" aria-label="ok" />, className: "text-ink-secondary" },
  warn:    { mark: <AlertTriangle className="h-3.5 w-3.5 text-amber-400" aria-label="aviso" />, className: "text-amber-300/90" },
  error:   { mark: <X className="h-3.5 w-3.5 text-rose-400" aria-label="error" />, className: "text-rose-300/90" },
} as const;
```

- `Activity` (info) → punto medio (`·`).
- `CheckCircle2` → `Check` (sólo el tick, sin círculo).
- `XCircle` → `X` (sólo la cruz, sin círculo).
- `AlertTriangle` (warn) se mantiene pero más discreto.

`Check` y `X` son los iconos más simples de lucide; reducen ruido y
mantienen las dos señales que el usuario pidió expresamente.

### 2.2 Fuente y contraste

El contenedor del log:

```tsx
// ANTES
className="h-64 overflow-y-auto rounded-lg bg-bg-canvas/60 border border-edge-default/30 p-3 font-mono text-xs text-ink-secondary"

// DESPUÉS
className="h-64 overflow-y-auto rounded-lg bg-bg-canvas/80 border border-edge-default/40 p-3 font-sans text-[13px] leading-relaxed text-ink-primary/90"
```

Cambios:

| Propiedad | Antes | Después | Razón |
|---|---|---|---|
| `font-mono` | sí | **no** (`font-sans`) | Una fuente sans-serif del sistema es más legible para texto narrativo (los timestamps mantienen `tabular-nums` para alinear). |
| `text-xs` (12 px) | sí | `text-[13px]` | 1 px más de tamaño = mucho menos cansancio. |
| `leading-relaxed` | no | sí | `line-height: 1.625` → respira. |
| `text-ink-secondary` | sí | `text-ink-primary/90` | Más contraste. |
| `bg-bg-canvas/60` | sí | `bg-bg-canvas/80` | Fondo más opaco, las líneas resaltan más. |
| `border-edge-default/30` | sí | `border-edge-default/40` | Borde más definido. |

### 2.3 Layout de cada línea

`LogLine` actual (líneas 73-100) → simplificar:

```tsx
function LogLine({ line, animated }: { line: CleanLogLinePayload; animated: boolean }) {
  const lvl = LEVEL[line.level as keyof typeof LEVEL] ?? LEVEL.info;
  const time = new Date(line.timestampMs).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  const content = (
    <div className={`flex items-center gap-2.5 py-1 ${lvl.className}`}>
      <span className="text-ink-muted tabular-nums text-[11px] w-[58px] shrink-0">
        {time}
      </span>
      <span className="w-4 flex items-center justify-center shrink-0">{lvl.mark}</span>
      <span className="text-ink-tertiary text-[12px] truncate max-w-[180px]" title={line.location}>
        {line.location}
      </span>
      <span className="flex-1 break-words">{line.message}</span>
    </div>
  );

  if (!animated) return content;
  return (
    <motion.div
      initial={{ opacity: 0, x: -3 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.12 }}
    >
      {content}
    </motion.div>
  );
}
```

Cambios concretos:

- **Timestamp**: `tabular-nums` + ancho fijo de 58 px → todas las
  horas se alinean en columna sin que la siguiente columna salte.
- **Mark column**: ancho fijo de 16 px → tick, equis, punto y
  triángulo ocupan exactamente el mismo espacio.
- **Location column**: subida a 180 px (antes 120) y color
  `text-ink-tertiary`. Ya no se queda recortada en nombres como
  "Microsoft Edge cache".
- **Mensaje**: `break-words` (antes `break-all`) → no parte palabras
  cortas a mitad, sólo cuando es necesario.

### 2.4 Quitar la animación interna costosa

`ANIMATED_TAIL = 30` permite que sólo las últimas 30 líneas se animen
con `framer-motion`. Reducir a 12 — es la "cola visible"
aproximadamente — para minimizar coste en limpiezas largas:

```ts
const ANIMATED_TAIL = 12;
```

Y bajar la duración a `0.12` (en el snippet de §2.3 ya está). La
animación pasa a ser una sensación, no un efecto consciente.

## 3. Captura visual antes/después

**Antes:**
```
17:42:13  ⚡  Microsoft Edge cache         Iniciando escaneo de: %LOCALAPPDATA%\...
17:42:14  ✓  Microsoft Edge cache         Completado: 42 archivos eliminados...
17:42:15  ⚠  Spotify cache                Archivo en uso, se eliminará en el próximo reboot...
17:42:16  ⚡  Windows temp                 Iniciando escaneo de: %TEMP%
```

**Después:**
```
17:42:13  ·   Microsoft Edge cache         Iniciando escaneo de: %LOCALAPPDATA%\...
17:42:14  ✓   Microsoft Edge cache         Completado: 42 archivos eliminados, 18 MB liberados
17:42:15  ▲   Spotify cache                Archivo en uso, se eliminará en el próximo reboot
17:42:16  ·   Windows temp                 Iniciando escaneo de: %TEMP%
```

Más limpio, todos los marcadores alineados, jerarquía visual clara
(tick verde = éxito, equis roja = error, ▲ ámbar = aviso, · neutro =
informativo).

## 4. Criterio de done

- [ ] El log usa `font-sans` con `text-[13px]` y `leading-relaxed`.
- [ ] Los iconos visibles son sólo: **✓** (success), **✗** (error),
      **▲** (warn) y un **·** (info).
- [ ] Las cuatro columnas (hora, marcador, ubicación, mensaje) están
      alineadas verticalmente sin saltos.
- [ ] No se ve `Activity`, `CheckCircle2`, `XCircle` por ningún sitio
      del componente.
- [ ] `ANIMATED_TAIL` está en 12; la animación es `0.12 s`.
- [ ] Líneas con `level: "error"` se ven rojizas pero discretas
      (`text-rose-300/90`), no chillonas.

## 5. Riesgos / efectos secundarios

- **A11y**: el `<span>` del punto neutro lleva `aria-hidden`. Los
  iconos llevan `aria-label`. El lector de pantalla recibe el nivel.
- **Líneas existentes**: los emits del backend ya envían `level`
  ("info", "warn", "error", "success"). El cambio es puramente
  cosmético.
- **Si llega un nivel desconocido** (e.g., el backend introduce
  "debug" sin avisar): el fallback `?? LEVEL.info` lo cubre.
- **Fuente del sistema**: `font-sans` resuelve a la fuente del
  proyecto (Inter o equivalente). Si en el futuro se quiere usar una
  monospace específica para el log, hay un escape: añadir clase
  `font-mono` sólo a `time` y `location`. No hace falta hoy.
