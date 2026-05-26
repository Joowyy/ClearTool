# Fix 04 — Puntos de restauración no aparecen en lista

## Síntoma
Al crear un punto de restauración, aparece el mensaje de éxito, pero la tabla permanece vacía (EmptyState). Las fechas no se mostraban.

## Causas raíz

### 1. Mismatch de campo `creationTime` vs `createdAt`
El struct Rust `RestorePoint` tiene `creation_time` → serializa como `"creationTime"`, pero la interfaz TS decía `createdAt`. Resultado: fechas `undefined`.

### 2. Mismatch de `RestoreReport`
El backend devuelve `{ sequenceNumber, createdAt, description, bypassedThrottle }` pero la interfaz TS esperaba `{ runId, sequenceNumber, success, message }`.

### 3. Errores silenciosos en PowerShell
El script `Get-ComputerRestorePoint` usa `$ErrorActionPreference = 'SilentlyContinue'`. Si no hay permisos de admin, devuelve stdout vacío. El backend interpreta esto como lista vacía (`Ok(Vec::new())`). No hay error visible.

**Consecuencia**: La query React Query "tiene éxito" pero con `[]`. La UI muestra EmptyState sin explicación.

## Fix
1. **`src/api/types.ts`** — `RestorePoint.createdAt` → `creationTime`. `RestoreReport` reescrito para coincidir con Rust.
2. **`src/features/restore-points/restore-page.tsx`** — `point.createdAt` → `point.creationTime`. Añadido display de error (`isError`) y botón Refresh manual.

## Nota operativa
Si `listRestorePoints` devuelve vacío, verificar:
- La app corre con permisos de administrador (manifest `requireAdministrator` en release)
- System Protection está habilitado en C:\ (`Enable-ComputerRestore -Drive 'C:\'`)
- En modo debug (`asInvoker`) los comandos WMI pueden fallar silenciosamente

## Archivos tocados
- `src/api/types.ts`
- `src/features/restore-points/restore-page.tsx`
