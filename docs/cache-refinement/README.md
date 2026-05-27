# Refinamiento del módulo Caché — ClearTool

> Generado: 2026-05-27
> Rama base: `fix-debloat`
> Trigger: feedback del usuario tras incidente real — el módulo cerró
> `explorer.exe` durante una limpieza y obligó a reiniciar el PC.
>
> Esta carpeta concentra **todo lo necesario para dejar la sección Caché
> "para tontos"**: que no te haga pensar, que no rompa nada, y que cuando
> termine te diga claramente qué pasó.

---

## Estructura

Los docs están repartidos en dos subcarpetas según la **dificultad de
ejecución** y, por tanto, qué modelo de IA debería atacarlos:

```
docs/cache-refinement/
├── README.md                                  ← este archivo (mapa global)
├── sonnet-4.6/                                ← DIFÍCILES (backend Rust, R3F, perf)
│   ├── README.md
│   ├── 01-shell-safe-close.md                 ✅ Set A
│   ├── 02-auto-analyze-on-boot.md             ✅ Set A
│   ├── 05-three-context-lost.md               ✅ Set A
│   ├── 08-redundancias-y-perf.md              ✅ Set A
│   ├── 09-progress-pipeline-backend.md        ✅ Set B
│   ├── 10-clean-console-frontend.md           ✅ Set B
│   ├── 11-clean-visualizer-r3f.md             ✅ Set B
│   ├── 12-eta-estimate-and-summary.md         ✅ Set B
│   ├── 13-cancellation-token-and-button.md    ⏳ Set C — quality of life
│   ├── 14-eta-calculating-forever-fix.md      ⏳ Set C
│   ├── 15-residual-bytes-investigation.md     ⏳ Set C
│   └── 16-performance-overhaul.md             ⏳ Set C (P0, ≥10× speedup)
└── qwen-3.6-plus/                             ← FÁCILES (config, copy, HTML)
    ├── README.md
    ├── 03-button-in-button.md                 ✅ Set A
    ├── 04-react-router-future-flags.md        ✅ Set A
    ├── 06-window-size.md                      ✅ Set A
    ├── 07-plan-view-copy.md                   ✅ Set A
    ├── 08-log-reset-between-runs.md           ⏳ Set C
    └── 09-log-readability-no-emoji.md         ⏳ Set C
```

**Set A — Refinamiento base** (P0/P1): proteger el shell, auto-analyze
al boot, Three.js Context Lost, redundancias. ✅ Implementado.

**Set B — Consola de limpieza** (P1/P2): sustituir el spinner de
"Limpiar" por una consola visual con animación 3D, ETA y resumen
final. ✅ Implementado.

**Set C — Quality of life** (P0/P1): tras probar la consola el usuario
reportó (a) log persistente entre limpiezas, (b) emojis del log poco
legibles, (c) falta de botón Cancelar, (d) ETA siempre "calculando…",
(e) ~400 MB residuales sin explicación, (f) **10 min para borrar
400 MB**. Set C resuelve esto en 6 docs (2 Qwen + 4 Sonnet).

### ¿Por qué Sonnet 4.6 para lo difícil?

- Toca código **destructivo** (procesos, FS, IPC) y necesita defensa en
  profundidad + tests.
- Implica **nuevos módulos backend** con lifecycle, locks y caché en
  memoria.
- Exige **expertise específica** (WebGL/Three.js internals, R3F frame
  loop) que un modelo más pequeño puede aplicar mal con consecuencias
  visibles para el usuario.

### ¿Por qué Qwen 3.6 Plus para lo fácil?

- Cambios en **un único archivo** o muy pocos.
- Sin lógica nueva — sólo **reemplazo de valor, atributo o markup**.
- **Sin riesgo de regresión sistémica**.
- Bajo coste por ejecución, dejando a Sonnet libre para lo crítico.

---

## Mapa de problemas

| # | Problema reportado | Sev | Asignado a | Doc |
|---|---|---|---|---|
| 01 | El "Cerrar procesos bloqueantes" cerró el Explorador de Windows | 🔴 P0 | Sonnet 4.6 | [`sonnet-4.6/01-shell-safe-close.md`](sonnet-4.6/01-shell-safe-close.md) |
| 02 | El usuario tiene que pulsar "Analizar" antes de poder limpiar | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/02-auto-analyze-on-boot.md`](sonnet-4.6/02-auto-analyze-on-boot.md) |
| 03 | `<button>` dentro de `<button>` en `PendingRenamesPanel` | 🟢 P2 | Qwen 3.6 Plus | [`qwen-3.6-plus/03-button-in-button.md`](qwen-3.6-plus/03-button-in-button.md) |
| 04 | `React Router` future flag warning (v7_startTransition) | 🟢 P2 | Qwen 3.6 Plus | [`qwen-3.6-plus/04-react-router-future-flags.md`](qwen-3.6-plus/04-react-router-future-flags.md) |
| 05 | `THREE.WebGLRenderer: Context Lost` durante el análisis | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/05-three-context-lost.md`](sonnet-4.6/05-three-context-lost.md) |
| 06 | Ventana muy pequeña; se siente apretada | 🟢 P2 | Qwen 3.6 Plus | [`qwen-3.6-plus/06-window-size.md`](qwen-3.6-plus/06-window-size.md) |
| 07 | Copy "Listas/Bloqueadas/Permisos/Omitidas" no comunica al no-técnico | 🟡 P1 | Qwen 3.6 Plus | [`qwen-3.6-plus/07-plan-view-copy.md`](qwen-3.6-plus/07-plan-view-copy.md) |
| 08 | Redundancias generales — análisis duplicado, refetch innecesario | 🟢 P2 | Sonnet 4.6 | [`sonnet-4.6/08-redundancias-y-perf.md`](sonnet-4.6/08-redundancias-y-perf.md) |
| 09 | El "Limpiar" sólo muestra un spinner mudo — no hay feedback de progreso/ETA | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/09-progress-pipeline-backend.md`](sonnet-4.6/09-progress-pipeline-backend.md) |
| 10 | Falta una consola visual con fase, barra y log durante la limpieza | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/10-clean-console-frontend.md`](sonnet-4.6/10-clean-console-frontend.md) |
| 11 | El usuario pidió "animaciones 3D" en la consola de limpieza | 🟢 P2 | Sonnet 4.6 | [`sonnet-4.6/11-clean-visualizer-r3f.md`](sonnet-4.6/11-clean-visualizer-r3f.md) |
| 12 | No se ve ETA antes de pulsar Limpiar ni resumen rico al terminar | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/12-eta-estimate-and-summary.md`](sonnet-4.6/12-eta-estimate-and-summary.md) |
| Q08 | El log de la consola se mantiene entre dos limpiezas seguidas | 🟡 P1 | Qwen 3.6 Plus | [`qwen-3.6-plus/08-log-reset-between-runs.md`](qwen-3.6-plus/08-log-reset-between-runs.md) |
| Q09 | Iconos del log poco legibles; sin emojis, sólo ✓/✗ | 🟢 P2 | Qwen 3.6 Plus | [`qwen-3.6-plus/09-log-readability-no-emoji.md`](qwen-3.6-plus/09-log-readability-no-emoji.md) |
| 13 | Falta un botón Cancelar la limpieza | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/13-cancellation-token-and-button.md`](sonnet-4.6/13-cancellation-token-and-button.md) |
| 14 | ETA dice "calculando…" eternamente con throughput <1 MB/s | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/14-eta-calculating-forever-fix.md`](sonnet-4.6/14-eta-calculating-forever-fix.md) |
| 15 | ~400 MB residuales tras limpiar; ni explicación ni opción de ignorar | 🟡 P1 | Sonnet 4.6 | [`sonnet-4.6/15-residual-bytes-investigation.md`](sonnet-4.6/15-residual-bytes-investigation.md) |
| 16 | 7 min 36 s para borrar 56 archivos / 400 MB — motor lentísimo | 🔴 P0 | Sonnet 4.6 | [`sonnet-4.6/16-performance-overhaul.md`](sonnet-4.6/16-performance-overhaul.md) |

Cada documento sigue el formato:

```
1. Problema (qué pasa, dónde está, por qué duele)
2. Causa raíz (referencia al código)
3. Fix propuesto (con paths exactos y snippets pegables)
4. Criterio de done
5. Riesgos / efectos secundarios
```

---

## Orden de ejecución global

**P0 antes que nada** — `sonnet-4.6/01-shell-safe-close.md` puede dejar
al usuario sin escritorio. No se mergea ningún cambio del módulo Caché
hasta que esta corrección esté revisada por `security-auditor`.

Luego (puede haber paralelismo entre Sonnet y Qwen):

### Set A — Refinamiento base (estado: ✅ implementado en `0e3110d`/`64b8632`)

1. **Sonnet → 01** — proteger el shell (bloquea el resto del backend).
2. **Qwen → 07** — re-copy del PlanView (cambia cómo el usuario percibe
   el resto).
3. **Sonnet → 02** — auto-analyze al boot (depende de 07 porque la
   primera pantalla cambia).
4. **Sonnet → 05** — pausar el 3D que pierde contexto (depende de 02
   porque el dashboard se queda más quieto).
5. **Qwen → 03, 04, 06** — fixes paralelos y cosméticos.
6. **Sonnet → 08** — redundancias y perf (depende de 02).

### Set B — Consola de limpieza (estado: ✅ implementado)

7. **Sonnet → 09** — pipeline de eventos enriquecidos del backend. ✅
8. **Sonnet → 10** — componente `CleanConsole`. ✅
9. **Sonnet → 11** — visualizador 3D R3F. ✅
10. **Sonnet → 12** — estimación previa + `CleanSummaryHero`. ✅

### Set C — Quality of life de la consola (estado: ⏳ pendiente)

Tras probar la consola, el usuario reportó 6 puntos. Repartidos entre
los dos modelos según dificultad:

**Paralelos** (Qwen y Sonnet a la vez):

11. **Qwen → Q08** — reset del log entre limpiezas (trivial).
12. **Qwen → Q09** — log más legible sin emojis (cosmético).
13. **Sonnet → 14** — fix del ETA "calculando…" eterno (independiente
    del resto del Set C).
14. **Sonnet → 13** — cancelación E2E con token + botón.

**Después de los anteriores:**

15. **Sonnet → 15** — desglose de los 400 MB residuales + UI
    "Quedó pendiente" + acción Ignorar. Depende de 13 (caso cancelled
    en el summary) y de 14 (el ETA correcto es prerequisito visual).
16. **Sonnet → 16** — **performance overhaul** (P0). Reescribe el
    motor de limpieza para pasar de ~880 KB/s a ≥10 MB/s. Tocará casi
    todo `cache.rs`. Pasa por `security-auditor` obligatoriamente. Va
    al final del Set C porque su scope es el mayor y depende
    indirectamente de todo lo anterior estar verde.

---

## Resumen ejecutivo para el usuario

- **Lo que cerró tu Explorador** fue una combinación de dos cosas: el
  analizador detecta que `explorer.exe` tiene archivos abiertos en
  `thumbcache_*.db` y `iconcache_*.db`, lo marca como "locker", y la
  opción "cerrar procesos bloqueantes automáticamente" lo ejecuta. La
  whitelist actual (`PROTECTED_PROCESS_NAMES` en
  `src-tauri/src/platform/processes.rs:15`) no incluye `explorer.exe`,
  así que pasa. **Fix en doc 01 (Sonnet)**.
- **El análisis tardío** (tienes que pulsar Re-analizar para ver algo)
  viene de que `analyze_locations` solo se llama bajo demanda. Lo
  movemos a un **worker en background al iniciar la app**, con caché y
  refresco automático. **Fix en doc 02 (Sonnet)**.
- **Las "Omitidas"** son ubicaciones que no tienen nada que limpiar
  (vacías, no existen, o fuera de allowlist). Tu intuición es correcta.
  Lo que falta es **etiquetar el motivo de cada una** para que no
  parezca un error. **Fix en doc 07 (Qwen)**.
- **Las "Bloqueadas por procesos"** se programan para reboot vía
  `MoveFileEx` con `PendingFileRenameOperations`. Tu intuición es
  correcta. **Fix en doc 07 (Qwen)**: cambiar el copy de "Bloqueadas" a
  algo como "Se limpiarán al reiniciar".
- **El WebGL Context Lost** lo causa el dashboard 3D del home cuando el
  sistema está bajo carga (limpieza intensiva). **Fix en doc 05
  (Sonnet)**: pausar el render cuando la app no es la ventana activa o
  cuando el módulo Caché está ejecutando.
- **Ventana** se sube de `1280×800` a `1440×920` con min `1200×800`.
  **Fix en doc 06 (Qwen)**.

---

## Estado

### Set A — Refinamiento base

| Doc | Carpeta | Generado | Implementado | Verificado |
|---|---|---|---|---|
| 01-shell-safe-close | sonnet-4.6 | ✅ | ✅ | ✅ (tests Rust) |
| 02-auto-analyze-on-boot | sonnet-4.6 | ✅ | ✅ | ✅ (+ fix `tauri::async_runtime`) |
| 05-three-context-lost | sonnet-4.6 | ✅ | ✅ | ✅ |
| 08-redundancias-y-perf | sonnet-4.6 | ✅ | ✅ | ✅ |
| 03-button-in-button | qwen-3.6-plus | ✅ | ✅ | ✅ (+ fix `id="pending-renames-list"`) |
| 04-react-router-future-flags | qwen-3.6-plus | ✅ | ✅ | ✅ (+ fix `v7_startTransition` en `<RouterProvider>`) |
| 06-window-size | qwen-3.6-plus | ✅ | ✅ | ✅ |
| 07-plan-view-copy | qwen-3.6-plus | ✅ | ✅ | ✅ |

### Set B — Consola de limpieza

| Doc | Carpeta | Generado | Implementado | Verificado |
|---|---|---|---|---|
| 09-progress-pipeline-backend | sonnet-4.6 | ✅ | ✅ | ✅ |
| 10-clean-console-frontend | sonnet-4.6 | ✅ | ✅ | ✅ |
| 11-clean-visualizer-r3f | sonnet-4.6 | ✅ | ✅ | ✅ |
| 12-eta-estimate-and-summary | sonnet-4.6 | ✅ | ✅ | ⚠ ETA bug → doc 14 |

### Set C — Quality of life

| Doc | Carpeta | Generado | Implementado | Verificado |
|---|---|---|---|---|
| 08-log-reset-between-runs | qwen-3.6-plus | ✅ | ❌ | ❌ |
| 09-log-readability-no-emoji | qwen-3.6-plus | ✅ | ❌ | ❌ |
| 13-cancellation-token-and-button | sonnet-4.6 | ✅ | ❌ | ❌ |
| 14-eta-calculating-forever-fix | sonnet-4.6 | ✅ | ❌ | ❌ |
| 15-residual-bytes-investigation | sonnet-4.6 | ✅ | ❌ | ❌ |
| 16-performance-overhaul | sonnet-4.6 | ✅ | ❌ | ❌ |
