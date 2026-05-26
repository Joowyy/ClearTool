// Punto único de importación para todo el frontend.
//
// Uso esperado en el resto del código:
//   import { listCacheLocations, type CacheLocation } from "@/api";
//
// (cuando se configure el path alias `@/` en tsconfig.json + vite.config.ts)

export * from "./client";
export * from "./events";
export * from "./types";
