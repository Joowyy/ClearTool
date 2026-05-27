# 08 — Reset del log entre limpiezas

> **Severidad:** 🟡 P1 — bug funcional reportado por el usuario.
> Al ejecutar una segunda limpieza en la misma instancia de ClearTool,
> la consola muestra las líneas de la limpieza anterior **mezcladas
> con las nuevas**.
> **Modelo:** Qwen 3.6 Plus.

## 1. Problema

En `src/features/cache-cleaner/cache-page.tsx:672-690`, el callback
`mutationFn` de `executeMutation`:

```ts
const executeMutation = useMutation({
  mutationFn: async (opts: { ... }) => {
    if (!plan) throw new Error("No hay plan");
    setConsoleOpen(true);                  // ← abre la consola
    const r = await executeCleanPlan(plan, { ... });
    const v = await verifyClean(plan, r);
    return { report: r, verify: v, dryRun: opts.dryRun };
  },
  ...
});
```

**Nunca llama a `cleanStream.reset()`**. El hook `useCleanStream` (en
`use-clean-stream.ts`) mantiene `log`, `status` y `runId` del run
anterior hasta que algo dispare `reset`. Como `runId` no se actualiza
(no se llama `start()` tampoco), los nuevos eventos se aceptan junto
con los viejos visibles.

Además, `clean-console.tsx:25-27` sólo resetea el `showFullLog` cuando
`status.kind === "idle"`, no el log en sí.

## 2. Fix propuesto

### 2.1 Reset al iniciar la mutación

`src/features/cache-cleaner/cache-page.tsx:679`:

```ts
const executeMutation = useMutation({
  mutationFn: async (opts: { ... }) => {
    if (!plan) throw new Error("No hay plan");
    cleanStream.reset();                   // ← NUEVO: limpiar antes de abrir
    setConsoleOpen(true);
    const r = await executeCleanPlan(plan, { ... });
    const v = await verifyClean(plan, r);
    return { report: r, verify: v, dryRun: opts.dryRun };
  },
  ...
});
```

`cleanStream.reset()` ya existe (`use-clean-stream.ts:93`) y deja
`{ status: { kind: "idle" }, log: [], runId: null }`. Después, los
eventos del backend para el nuevo `runId` rellenan la consola desde
cero.

### 2.2 Reset también al cerrar (defensa en profundidad)

En `cache-page.tsx`, donde se define `setConsoleOpen(false)` (al
cerrar la consola), envolver en una función:

```ts
const handleConsoleClose = useCallback(() => {
  setConsoleOpen(false);
  cleanStream.reset();
  // Refrescar plan caliente y throughput stats para el próximo entry.
  void qc.invalidateQueries({ queryKey: ["cache-plan-warm"] });
  void qc.invalidateQueries({ queryKey: ["throughput-stats"] });
}, [cleanStream, qc]);

// Y en el JSX:
<CleanConsole
  open={consoleOpen}
  isRunning={executeMutation.isPending}
  onClose={handleConsoleClose}
/>
```

(Si ya existe esta lógica con otro nombre, reusar — no duplicar.)

### 2.3 Reset también al desmontar la página

Cuando el usuario navega a otra pestaña y vuelve, los eventos zombies
(si hubiera alguno en cola) no contaminan. En `CachePage`:

```ts
useEffect(() => {
  return () => {
    // Al desmontar la página, limpiar el stream para que la
    // próxima visita arranque virgen.
    cleanStream.reset();
  };
}, []); // eslint-disable-line react-hooks/exhaustive-deps
```

## 3. Criterio de done

- [ ] Tras una limpieza, cerrar la consola y volver a pulsar **Limpiar**
      muestra un log **vacío** al principio.
- [ ] Las primeras líneas que aparecen son del nuevo run, no del anterior.
- [ ] Después de un `dry-run` y luego un `Limpiar` real, no se mezclan
      logs.
- [ ] Navegar a otra pestaña y volver no muestra el log antiguo.

## 4. Riesgos / efectos secundarios

- **Race de evento "summary" tardío**: si el backend emite `cache:summary`
  con un retraso de 50-200 ms después de que el frontend ya resolvió
  la promesa y cerró/reseteó la consola, ese evento se descarta
  silenciosamente (el reducer ya está en `idle` con `runId: null`).
  Es lo deseable — el resumen ya lo cubre el toast + el state local
  del PlanView.
- **Volver a abrir consola justo después de cerrar**: el doble reset
  (close + start) no es problema porque `reset()` es idempotente.
