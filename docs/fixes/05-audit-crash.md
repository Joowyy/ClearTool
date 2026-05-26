# Fix 05 — Auditoría crash "Objects are not valid as a React child"

## Síntoma
```
Objects are not valid as a React child (found: object with keys {kind, operations}).
```
La página de Auditoría crasheaba al intentar renderizar las entradas.

## Causa raíz
El enum Rust `ReverseRecipe` se serializa como objeto con discriminante `kind`:
```json
{ "kind": "Noop", "reason": "método appx-user no reversible automáticamente" }
{ "kind": "AppxReinstall", "packageFamilyName": "...", "storeUrl": null }
{ "kind": "Registry", "operations": [...] }
```

Pero la interfaz TS declaraba:
```ts
interface AuditEntry {
  reverseRecipe: string | null;  // ← INCORRECTO, es un objeto
  restorePointId: number | null; // ← incorrecto, backend manda "restorePointSeq"
  itemsCount: number;            // ← incorrecto, backend manda "itemsAffected: string[]"
  success: boolean;              // ← incorrecto, backend manda "status: string"
}
```

El JSX `{entry.reverseRecipe ?? "—"}` intentaba renderizar el objeto directamente → crash.

## Fix

### 1. `src/api/types.ts`
- Añadida union type `ReverseRecipe` con variantes `Registry | Service | AppxReinstall | Noop`.
- `AuditEntry` reescrito para coincidir exactamente con el struct Rust.

### 2. `src/features/audit-log/audit-page.tsx`
- `entry.success` → `entry.status === "success"`
- `entry.itemsCount` → `entry.itemsAffected.length`
- `first.restorePointId` → `first.restorePointSeq`
- `{entry.reverseRecipe ?? "—"}` → función helper `renderRecipe(recipe)` que devuelve un string legible según el `kind`
- `hasReversible` ahora comprueba `reverseRecipe.kind !== "Noop"`

## Archivos tocados
- `src/api/types.ts`
- `src/features/audit-log/audit-page.tsx`
