# Skills de Claude Code para ClearTool

Cada skill es una carpeta con un `SKILL.md` (system prompt extenso) y opcionalmente `RESOURCES/` con datos canónicos.

## Ubicación

Esta carpeta es `.claude/skills/`. Claude Code la detecta automáticamente al abrir el repo.

## Skills incluidas

| Skill | Cuándo se carga |
|---|---|
| `windows-registry-ops` | Cualquier escritura/lectura "decisional" del registro. |
| `powershell-debloat` | Invocar PowerShell para Appx/Provisioned/Edge. |
| `restore-point-manager` | Antes de cualquier operación destructiva. |
| `cache-scanner` | Implementar/extender el módulo cache-cleaner. |
| `tauri-command-builder` | Crear un nuevo comando Tauri. |
| `directory-tree-explorer` | Implementar el módulo Explorer. |

## Convenciones

- `SKILL.md` empieza con frontmatter YAML (`name`, `description`).
- `description` debe ser específica para que Claude la cargue solo cuando aplica.
- Datos canónicos (catálogos, schemas) van en `RESOURCES/` y se referencian desde el SKILL.md.
- Cualquier cambio a un catálogo (`bloatware-catalog.json`, `cache-locations.json`, `services-catalog.json`, `registry-tweaks.json`) requiere revisión de `security-auditor`.
