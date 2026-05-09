# Specs de ClearTool

Documentos de diseño que guían la implementación. Léelos en orden la primera vez.

Ubicación: `.claude/specs/`. Conviven aquí porque son contexto que Claude Code consulta junto con agents y skills.

| Archivo | Tema |
|---|---|
| `00-overview.md` | Visión, objetivos, no-objetivos, stakeholders. |
| `01-architecture.md` | Arquitectura general, capas, decisiones técnicas. |
| `02-backend-rust.md` | Estructura `src-tauri/`, dependencias, patrones. |
| `03-frontend-react.md` | Estructura `src/`, stack UI, navegación. |
| `04-modules/directory-explorer.md` | Módulo Explorer. |
| `04-modules/cache-cleaner.md` | Módulo limpieza de caché. |
| `04-modules/debloat-engine.md` | Módulo debloat. |
| `04-modules/service-manager.md` | Servicios y telemetría. |
| `04-modules/registry-tweaks.md` | Tweaks de registro. |
| `04-modules/restore-point-system.md` | Restore points y log reversible. |
| `05-security.md` | Modelo de amenazas y mitigaciones. |
| `06-ui-ux.md` | Lenguaje visual, flujos críticos. |
| `07-testing.md` | Pirámide de tests y plan VM. |

Cualquier cambio sustancial a una spec requiere mención en `CLAUDE.md` -> "Decisiones arquitectónicas vivas".
