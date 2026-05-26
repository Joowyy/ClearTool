# Paso 05 — Sweep: reemplazar `String(err)` en todo el código

**Área**: 01-error-handling
**Tiempo estimado**: 4-6 horas
**Dependencias**: Paso 01, 02, 03

## Qué hacemos

Auditoría completa: buscar todas las ocurrencias de `String(err)`, `${err}`, `err.message`, etc. en JSX/TS, y reemplazarlas por `formatError(err)` + `toast.error()` cuando aplique.

## Por qué

Es el grunt work del fix. Sin esto, el `errors.ts` y el `toast.ts` quedan sin usar y los `[object Object]` siguen apareciendo.

## Archivos a auditar (lista de sospechosos)

- `src/features/audit-log/audit-page.tsx`
- `src/features/restore-points/restore-page.tsx`
- `src/features/debloat/debloat-page.tsx`
- `src/features/cache-cleaner/cache-page.tsx`
- `src/features/registry-tweaks/registry-page.tsx`
- `src/features/services/services-page.tsx`
- `src/features/explorer/explorer-page.tsx`
- `src/features/settings/settings-page.tsx`
- Todos los `hooks/*.ts`

## Cómo

### 1. Encuentra todas las ocurrencias

Grep en el repo:

```bash
# En la raíz del proyecto:
grep -rn "String(\(.*Error\|err\|error\)" src/ --include="*.ts" --include="*.tsx"
grep -rn "\${err}" src/ --include="*.ts" --include="*.tsx"
grep -rn "\.message}" src/ --include="*.tsx"
grep -rn "JSON.stringify(err" src/ --include="*.tsx"
```

Apunta cada match en una lista (o usa el output del grep como tu lista de trabajo).

### 2. Por cada hit, decide el patrón de reemplazo

Hay 3 patrones según el caso:

#### Patrón A — Error transitorio (mutation onError)
Sustituye el banner inline por un toast en `onError`:

ANTES:
```tsx
const removeMutation = useMutation({ mutationFn: ... });
// ...
{removeMutation.isError && (
  <div className="p-3 bg-red-900/30 ...">
    Error: {String(removeMutation.error)}
  </div>
)}
```

DESPUÉS:
```tsx
import { toast } from "../../lib/toast";

const removeMutation = useMutation({
  mutationFn: ...,
  onError: (err) => toast.error("No se pudieron eliminar los paquetes", err),
});
// El banner JSX se ELIMINA por completo
```

#### Patrón B — Error inline que debe persistir (banner amarillo, query error)
Mantén banner inline pero usa `formatError`:

ANTES:
```tsx
{isError && (
  <div className="bg-red-900/30 ...">
    Error al cargar: {String(error)}
  </div>
)}
```

DESPUÉS:
```tsx
import { formatError } from "../../lib/errors";

{isError && (
  <div className="bg-red-900/30 ...">
    Error al cargar: {formatError(error)}
  </div>
)}
```

#### Patrón C — Error en un try/catch manual
Sustituye el `setError(String(e))` por `toast.error()`:

ANTES:
```tsx
try {
  await doStuff();
} catch (err) {
  setError(String(err));
}
```

DESPUÉS:
```tsx
try {
  await doStuff();
} catch (err) {
  toast.error("Algo falló", err);
}
// Elimina el state `error` y su banner si solo se usaba para esto
```

### 3. Casos especiales del v0.1 actual

#### audit-page.tsx

Hay esto:
```tsx
{revertMutation.isError && (
  <div className="p-3 bg-red-900/30 ...">
    Error al revertir: {String(revertMutation.error)}
  </div>
)}
```

→ Mover a `onError` del `useMutation` con `toast.error`.

#### restore-page.tsx

Hay esto (después del Paso 1 ya parchado):
```tsx
{isError && (
  <div className="bg-red-900/30 ...">
    Error al cargar puntos de restauración: {String(error)}
  </div>
)}
```

→ Cambiar a `formatError(error)` (Patrón B — es de un useQuery, persistente).

```tsx
{isError && (
  <div className="bg-red-900/30 ...">
    Error al cargar puntos de restauración: {formatError(error)}.
    Verifica que la app tiene permisos de administrador.
  </div>
)}
```

#### debloat-page.tsx

Mismo patrón. Aplicar A para mutations, B para queries.

### 4. Verifica con grep que no quedan

```bash
grep -rn "String(.*[Ee]rror" src/ --include="*.ts" --include="*.tsx"
# debe devolver 0 líneas relevantes (las ocurrencias en errors.ts mismas son OK)
```

### 5. Smoke test manual

Lanza la app y provoca cada tipo de error conocido:

- Debloat → Detectar instalados sin admin → debe salir toast rojo, NO banner inline con `[object Object]`.
- Restore → Verificar estado sin admin → toast.
- Cache → Limpiar con apps abiertas → toast con bloqueadores.

## Criterio de done

- [ ] `grep -rn "String(.*[Ee]rror" src/` devuelve 0 resultados (excluyendo `errors.ts`).
- [ ] `grep -rn "\${err\(or\)?}" src/` en JSX devuelve 0 resultados.
- [ ] Todos los `useMutation` con `isError` JSX banner movidos a `onError` con `toast.error`.
- [ ] Todos los `useQuery` con `isError` usan `formatError(error)`.
- [ ] Smoke test pasa los 3 escenarios (debloat sin admin, restore sin admin, cache con apps abiertas).
- [ ] No queda ningún banner rojo inline residual sin lógica detrás (limpieza de JSX huérfano).
