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

## Mapa de problemas y de fixes

| # | Problema reportado | Severidad | Fix |
|---|---|---|---|
| 01 | El "Cerrar procesos bloqueantes" cerró el Explorador de Windows | 🔴 P0 | [`01-shell-safe-close.md`](01-shell-safe-close.md) |
| 02 | El usuario tiene que pulsar "Analizar" y esperar antes de poder limpiar | 🟡 P1 | [`02-auto-analyze-on-boot.md`](02-auto-analyze-on-boot.md) |
| 03 | `<button>` dentro de `<button>` en `PendingRenamesPanel` (devtools warning) | 🟢 P2 | [`03-button-in-button.md`](03-button-in-button.md) |
| 04 | `React Router will begin wrapping state updates in React.startTransition in v7` | 🟢 P2 | [`04-react-router-future-flags.md`](04-react-router-future-flags.md) |
| 05 | `THREE.WebGLRenderer: Context Lost` durante el análisis | 🟡 P1 | [`05-three-context-lost.md`](05-three-context-lost.md) |
| 06 | Ventana muy pequeña; se siente apretada | 🟢 P2 | [`06-window-size.md`](06-window-size.md) |
| 07 | El copy de "Listas / Bloqueadas / Permisos / Omitidas" no comunica nada al no-técnico | 🟡 P1 | [`07-plan-view-copy.md`](07-plan-view-copy.md) |
| 08 | Redundancias generales — análisis duplicado, refetch innecesario, UI ruidosa | 🟢 P2 | [`08-redundancias-y-perf.md`](08-redundancias-y-perf.md) |

Cada documento sigue el formato:

```
1. Problema (qué pasa, dónde está, por qué duele)
2. Causa raíz (referencia al código)
3. Fix propuesto (con paths exactos y snippets pegables)
4. Criterio de done
5. Riesgos / efectos secundarios
```

---

## Orden de ejecución sugerido

**P0 antes que nada** — `01-shell-safe-close.md` puede dejar al usuario sin escritorio. No se mergea ningún cambio del módulo Caché hasta que esta corrección esté revisada por `security-auditor`.

Luego:

1. **01** — proteger el shell (bloquea el resto).
2. **07** — re-copy del PlanView (cambia cómo el usuario percibe el resto).
3. **02** — auto-analyze al boot (depende de 07 porque la primera pantalla cambia).
4. **05** — quitar/pausar el 3D que pierde contexto (depende de 02 porque el dashboard se queda más quieto).
5. **03**, **04**, **06**, **08** — fixes paralelos y cosméticos.

---

## Resumen ejecutivo para el usuario

- **Lo que cerró tu Explorador** fue una combinación de dos cosas: el analizador detecta que `explorer.exe` tiene archivos abiertos en `thumbcache_*.db` y `iconcache_*.db`, lo marca como "locker", y la opción "cerrar procesos bloqueantes automáticamente" lo ejecuta. La whitelist actual (`PROTECTED_PROCESS_NAMES` en `src-tauri/src/platform/processes.rs:15`) no incluye `explorer.exe`, así que pasa. **Fix en doc 01**.
- **El análisis tardío** (tienes que pulsar Re-analizar para ver algo) viene de que `analyze_locations` solo se llama bajo demanda. Lo movemos a un **worker en background al iniciar la app**, con caché y refresco automático. **Fix en doc 02**.
- **Las "Omitidas"** son ubicaciones que no tienen nada que limpiar (vacías, no existen, o fuera de allowlist). Tu intuición es correcta. Lo que falta es **etiquetar el motivo de cada una** para que no parezca un error. **Fix en doc 07**.
- **Las "Bloqueadas por procesos"** se programan para reboot vía `MoveFileEx` con `PendingFileRenameOperations`. Tu intuición es correcta. **Fix en doc 07**: cambiar el copy de "Bloqueadas" a algo como "Se limpiarán al reiniciar".
- **El WebGL Context Lost** lo causa el dashboard 3D del home cuando el sistema está bajo carga (limpieza intensiva). **Fix en doc 05**: pausar el render cuando la app no es la ventana activa o cuando el módulo Caché está ejecutando.
- **Ventana** se sube de `1280×800` a `1440×900` con min `1200×800`. **Fix en doc 06**.

---

## Estado

| Doc | Generado | Implementado | Verificado |
|---|---|---|---|
| 01-shell-safe-close | ✅ | ❌ | ❌ |
| 02-auto-analyze-on-boot | ✅ | ❌ | ❌ |
| 03-button-in-button | ✅ | ❌ | ❌ |
| 04-react-router-future-flags | ✅ | ❌ | ❌ |
| 05-three-context-lost | ✅ | ❌ | ❌ |
| 06-window-size | ✅ | ❌ | ❌ |
| 07-plan-view-copy | ✅ | ❌ | ❌ |
| 08-redundancias-y-perf | ✅ | ❌ | ❌ |
