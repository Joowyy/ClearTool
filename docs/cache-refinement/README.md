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
├── README.md                            ← este archivo (mapa global)
├── sonnet-4.6/                          ← tareas DIFÍCILES (backend Rust, internals, R3F)
│   ├── README.md
│   ├── 01-shell-safe-close.md           ✅ Set A
│   ├── 02-auto-analyze-on-boot.md       ✅ Set A
│   ├── 05-three-context-lost.md         ✅ Set A
│   ├── 08-redundancias-y-perf.md        ✅ Set A
│   ├── 09-progress-pipeline-backend.md  ⏳ Set B — consola de limpieza
│   ├── 10-clean-console-frontend.md     ⏳ Set B
│   ├── 11-clean-visualizer-r3f.md       ⏳ Set B
│   └── 12-eta-estimate-and-summary.md   ⏳ Set B
└── qwen-3.6-plus/                       ← tareas FÁCILES (config, copy, HTML)
    ├── README.md
    ├── 03-button-in-button.md           ✅
    ├── 04-react-router-future-flags.md  ✅
    ├── 06-window-size.md                ✅
    └── 07-plan-view-copy.md             ✅
```

**Set A — Refinamiento base** (P0/P1): proteger el shell, auto-analyze
al boot, Three.js Context Lost, redundancias. Ya implementado.

**Set B — Consola de limpieza** (P1/P2): sustituir el spinner de
"Limpiar" por una consola visual con animación 3D, ETA y resumen
final. Cuatro bloques que se implementan en orden estricto: 09 → 10 →
11 → 12.

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

### Set B — Consola de limpieza (estado: ⏳ pendiente)

Estricto en este orden, sin paralelismo (cada uno depende del anterior):

7. **Sonnet → 09** — pipeline de eventos enriquecidos del backend
   (CleanProgressPayload, CleanPhase, CleanSummaryPayload + throughput
   stats persistidos).
8. **Sonnet → 10** — componente `CleanConsole` (modal + header con
   fase/ETA/barra + log scrolleable; el toast se mantiene).
9. **Sonnet → 11** — visualizador 3D R3F (anillo + halo + partículas
   en el header de la consola).
10. **Sonnet → 12** — estimación previa en PlanView + `CleanSummaryHero`
    al terminar + invalidación de queries al cerrar.

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
| 09-progress-pipeline-backend | sonnet-4.6 | ✅ | ❌ | ❌ |
| 10-clean-console-frontend | sonnet-4.6 | ✅ | ❌ | ❌ |
| 11-clean-visualizer-r3f | sonnet-4.6 | ✅ | ❌ | ❌ |
| 12-eta-estimate-and-summary | sonnet-4.6 | ✅ | ❌ | ❌ |
