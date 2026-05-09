# Subagentes de Claude Code para ClearTool

Esta carpeta contiene definiciones de subagentes especializados.

## Ubicación

Esta carpeta es `.claude/agents/`. Claude Code la detecta automáticamente al abrir el repo.

## Subagentes incluidos

| Archivo | Cuándo invocarlo |
|---|---|
| `windows-systems-expert.md` | Dudas sobre Win32, registro, servicios, internals. |
| `tauri-rust-backend.md` | Plumbing Tauri, comandos, IPC, Rust. |
| `react-frontend.md` | UI, componentes, estado, comunicación con backend. |
| `debloat-specialist.md` | Catálogo de bloatware y estrategias de remoción. |
| `security-auditor.md` | Revisión obligatoria de cambios destructivos. |
| `qa-tester.md` | Plan de tests, VM smoke, reversibilidad. |

## Cómo se invocan

En Claude Code:

```
Quiero limpiar el caché de Windows Update — ¿qué rutas son seguras?
```

Claude detectará por la `description` que el subagente `windows-systems-expert` aplica y lo invocará. También puedes forzar con:

```
Usa el subagente debloat-specialist para curar la lista final de paquetes Xbox.
```

## Convenciones

- Cada subagente tiene un `description` que dice **cuándo** invocarlo.
- Cada subagente declara las `tools` que necesita (principio de mínimo privilegio).
- El cuerpo del archivo es el system prompt del subagente.
- Los subagentes se delegan trabajo entre sí mediante texto (no hay invocación programática).
