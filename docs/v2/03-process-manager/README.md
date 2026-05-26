# 03 — Process Manager (M2, P0)

**Objetivo**: módulo nuevo que lista procesos, los clasifica, permite acciones (kill, suspend, close gracefully), y sirve de base al cache cleaner v2.

**Spec de referencia**: `.claude/specs/v2/03-process-manager.md`.

**Por qué es P0**: el cache cleaner v2 (módulo 02) lo necesita. Sin `who_locks` y `close_gracefully`, el cleaner no puede ofrecer "Cerrar Spotify y limpiar".

## Pasos en orden

1. [01 — Modelos `ProcessInfo`, `ProcessCategory`](01-modelo-process-info.md)
2. [02 — Listado: `list_processes_extended`](02-list-processes-extended.md)
3. [03 — Acciones: kill, suspend, resume](03-acciones-kill-suspend.md)
4. [04 — Close gracefully con WM_CLOSE](04-close-gracefully.md)
5. [05 — Lista de procesos protegidos](05-protected-system-list.md)
6. [06 — IPC + frontend (pestaña Procesos)](06-ipc-y-frontend.md)

## Dependencias

- `01-error-handling/` terminado (errores tipados desde el inicio).
- `01-restart-manager-api.md` del módulo cache (puede vivir aquí también — coloca según prefiera el equipo).

## Criterio de done

- [ ] `list_processes_extended` devuelve 100+ procesos en <100ms en una máquina típica.
- [ ] Clasificación heurística asigna `ProcessCategory` razonable a ≥80% de procesos.
- [ ] `kill_process` y `kill_process_tree` funcionan.
- [ ] `suspend_process` / `resume_process` funcionan (verificable con Task Manager).
- [ ] `close_gracefully(Spotify_PID, 5000)` cierra Spotify limpiamente con WM_CLOSE.
- [ ] Lista de procesos protegidos del sistema (csrss, smss, lsass, etc.) bloquea acciones con tooltip.
- [ ] Pestaña Procesos en UI agrupa por categoría y permite filtrar.
- [ ] Preset "Liberar para limpiar" cierra browsers/spotify/discord/claude con confirmación.

## Tiempo total estimado

20-25 horas.
