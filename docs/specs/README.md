# ClearTool — Specs

Estos specs documentan la arquitectura, módulos y decisiones de diseño de ClearTool. Están versionados en dos directorios:

| Versión | Carpeta | Estado | Para qué |
|---------|---------|--------|----------|
| **v1** | [`v1/`](v1/) | Implementado parcialmente (v0.1) | Refleja la arquitectura y módulos que existen hoy en `src-tauri/` y `src/`. Útil para entender el código actual. |
| **v2** | [`v2/`](v2/) | Plan para release público (v1.0) | Refactors necesarios, módulos nuevos y mejoras de UX antes de publicar. Documenta también los bugs conocidos y sus fixes propuestos. |

## Cómo elegir qué leer

- **Trabajando en un bug del código actual** → `v1/04-modules/<modulo>.md` para entender el contrato existente.
- **Diseñando una mejora o feature nueva** → `v2/` (empezar por `09-roadmap.md` y luego el módulo concreto).
- **Onboarding** → leer `v1/00-overview.md` y luego `v2/00-vision-public-release.md`.

## Sin doble fuente de verdad

Los specs v1 y v2 NO deben mantenerse sincronizados manualmente. Cuando una feature v2 se implemente:
1. Su spec se mueve de `v2/` a `v1/` (sobreescribiendo el antiguo).
2. La sección correspondiente en `v2/09-roadmap.md` se marca como completada.
3. El cambio queda registrado en `HISTORIAL-SESIONES.md`.

Esto evita el problema clásico de "el spec dice una cosa y el código hace otra".
