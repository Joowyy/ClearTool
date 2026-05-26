# Fix 03 — Debloat "Detectar instalados" no funcionaba

## Síntoma
Al pulsar "Detectar instalados" en la página Debloat, ningún paquete mostraba el badge "Instalado" aunque los paquetes estuviesen realmente instalados.

## Causa raíz
Desincronización entre el struct Rust `DetectedPackage` y la interfaz TypeScript del mismo nombre.

### Lo que manda el backend (Rust → JSON camelCase)
```json
{
  "id": "ms-xbox-app",
  "displayName": "Xbox App",
  "installedForUser": true,
  "installedProvisioned": false,
  "sizeEstimateMb": null,
  "installLocation": "C:\\..."
}
```

### Lo que esperaba el frontend (TS antes del fix)
```ts
interface DetectedPackage {
  entryId: string;           // ← no existe, backend manda "id"
  installedAllUsers: boolean; // ← no existe en Rust
  provisioned: boolean;       // ← nombre equivocado (era "installedProvisioned")
  packageFullName: string;    // ← no existe en Rust
  sizeEstimateBytes: number;  // ← nombre equivocado (era "sizeEstimateMb")
}
```

La página usaba `p.entryId` → `undefined` → el Set `installedIds` nunca contenía ningún ID → ningún paquete marcado.

## Fix
1. **`src/api/types.ts`** — `DetectedPackage` reescrito para coincidir con el struct Rust.
2. **`src/features/debloat/debloat-page.tsx`** — `p.entryId` → `p.id`, `detected?.packageFullName` → `detected?.installLocation`.

## Archivos tocados
- `src/api/types.ts`
- `src/features/debloat/debloat-page.tsx`
