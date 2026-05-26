# v2 · 09 — Roadmap (v0.1 → v1.0)

Plan de ejecución ordenado en milestones. Cada milestone tiene un objetivo claro, dependencias resueltas, y criterio de aceptación.

## Filosofía

- **No paralelizar lo no paralelizable**. P0 antes que P1. P1 antes que P2.
- **Cada milestone es releasable** como `0.X` en GitHub. Beta channel ofrece estos drops a early adopters.
- **No mezclar refactor + feature**. Si un milestone es refactor, no metemos features. Y al revés.

---

## M1 — "El UX no sangra" (Bug fixes críticos)

**Duración estimada**: 2-3 semanas
**Versión target**: `v0.2.0`
**Foco**: arreglar lo que es bochornoso enseñar.

### Tareas
- [ ] **Error model** (`v2/01-error-model-fix.md`)
  - `src/lib/errors.ts` con `normalizeError`, `formatError`.
  - Sustituir todos los `String(err)` por `formatError(err)`.
  - Integrar `sonner` en AppShell.
  - ErrorBoundary global en el router.
- [ ] **Catálogo de bugs ya identificados** (`v2/08-known-issues.md`)
  - Bug #008 — restore list error visible y accionable.
  - Bug #014 — glitch `\` en registry tweaks (inspeccionar y arreglar).
  - Bug #016 — read registry tweak states batch.
- [ ] **Settings ampliados**
  - Diagnostic mode toggle.
  - Botón "Exportar reporte de diagnóstico" (bug #037).
  - About con versión + build info (bug #038).

### Done
- En todas las páginas, ningún error muestra `[object Object]`.
- Toast system funciona con success/error/warning/info.
- ErrorBoundary captura crash de cualquier ruta.
- Reporte de diagnóstico genera ZIP descargable.

---

## M2 — "El cleaner funciona de verdad" (Cache engine + Process Manager)

**Duración estimada**: 4-5 semanas
**Versión target**: `v0.3.0`
**Foco**: arreglar la funcionalidad principal de la app.

### Tareas
- [ ] **Process Manager** (`v2/03-process-manager.md`)
  - Modelo `ProcessInfo` con categorización.
  - `list_processes_extended()` con sysinfo + win32.
  - `kill_process`, `kill_process_tree`, `suspend`, `resume`, `close_gracefully`.
  - `who_locks_path` con Restart Manager API.
  - Pestaña Procesos en UI.
  - Preset "Liberar para limpieza".
- [ ] **Cache Engine Rewrite** (`v2/02-cache-engine-rewrite.md`)
  - Strategies tipificadas en el catálogo.
  - `analyze_locations` → `CleanPlan` con secciones ready/blocked/permission/skipped.
  - Retry x3 con backoff.
  - `MoveFileEx` + `PendingFileRenameOperations` integrados.
  - `reset_uwp_app` para escape hatch.
  - UI rediseñada: 4 secciones, acciones contextuales.
  - Verificación automática post-clean.

### Done
- Limpieza de Spotify/Claude/Discord con apps abiertas: muestra bloqueador + ofrece cerrarlo.
- Tras "Limpiar", la métrica final dice exactamente cuánto se liberó vs cuánto está bloqueado.
- Sin archivos UWP "perdidos" en el limbo (todo o se borra o se programa para reboot, no falla silente).
- En VM Win11 OEM clean, el cache cleaner libera mínimo 80% de lo escaneado en un solo pase.

---

## M3 — "Hay contenido que justifica usarla" (Catálogo + Módulos nuevos)

**Duración estimada**: 4-6 semanas
**Versión target**: `v0.4.0`
**Foco**: contenido y diferenciación.

### Tareas
- [ ] **Debloat Catalog Expansion** (`v2/04-debloat-catalog-expansion.md`)
  - 120+ entradas curadas en 8 categorías.
  - Schema v2 con campos extra.
  - Reverse recipes específicas para cada entrada.
  - Disclaimers UI para packages destructivos.
  - Carga de `bloatware-catalog.user.json` opcional.
  - Smoke test en VM OEM detectando 40+.
- [ ] **Startup Manager** (`v2/05-new-modules.md` §5.1)
  - 4 orígenes (registry, folder, scheduled task, service, UWP).
  - Disable reversible (rename `.disabled`).
  - Medición de impacto desde Event Log.
- [ ] **Boot-time Cleanup** (`v2/05-new-modules.md` §5.2)
  - Listar/cancelar pending renames.
  - Schedule via API limpia.
- [ ] **Privacy Hardening preset** (`v2/05-new-modules.md` §5.5)
  - 3 niveles (Balanced, Strict, Paranoid).
  - "Ver cambios" antes de aplicar.
  - Dry-run.

### Done
- Catálogo Debloat ofrece reconocimiento útil en una VM OEM real (Lenovo o HP).
- Startup Manager permite deshabilitar entradas y revertir.
- Preset "Modo paranoid" aplica ~50 cambios coordinados con un click + restore point.

---

## M4 — "Brilla" (UI/UX refactor + Disk Analyzer)

**Duración estimada**: 3-4 semanas
**Versión target**: `v0.5.0` (RC1)
**Foco**: presentación pulida + feature wow.

### Tareas
- [ ] **UI/UX Refactor** (`v2/06-ui-ux-refactor.md`)
  - Sidebar colapsable (sustituye titlebar tabs).
  - Command Palette Ctrl+K.
  - 3 themes (dark-cyan, dark-amber, light) + follow-system.
  - Density toggle.
  - Confirm dialogs estandarizados.
  - EmptyStates con acción.
  - TableSkeletons.
  - Atajos teclado + modal Ctrl+/.
  - i18n base (ES + EN).
  - Lazy loading de rutas pesadas.
- [ ] **Disk Analyzer** (`v2/05-new-modules.md` §5.3)
  - `build_treemap_data` en backend.
  - Treemap canvas con d3-hierarchy.
  - Zoom + acciones contextuales.
- [ ] **Network Utilities** (`v2/05-new-modules.md` §5.4)
  - 6 botones en Settings → Tools.

### Done
- Sidebar funciona en idiomas largos sin overflow.
- Command Palette responde <100ms con búsqueda en todos los catálogos.
- 3 themes con contraste WCAG AA.
- Treemap navegable con 5k+ nodos sin lag.

---

## M5 — "Lista para gente desconocida" (Distribución + Polish)

**Duración estimada**: 2-3 semanas
**Versión target**: `v1.0.0` 🎉
**Foco**: confianza, signing, web.

### Tareas
- [ ] **Code signing** (`v2/07-distribution.md`)
  - Azure Trusted Signing setup.
  - CI workflow con firma automática.
- [ ] **Installer polish**
  - NSIS customizado (logo, idiomas, license display).
  - Test en 3 VMs (22H2, 23H2, 24H2).
- [ ] **Auto-updater**
  - Plugin updater configurado.
  - Endpoint web + JSON `latest.json`.
  - Canales stable/beta/nightly.
- [ ] **Web propia**
  - Landing en `cleartool.app`.
  - Docs básicos (install, FAQ, troubleshooting).
  - Changelog auto-generado.
- [ ] **Distribución**
  - GitHub Releases pipeline.
  - winget PR.
  - README + LICENSE + CONTRIBUTING + PRIVACY-POLICY.
- [ ] **QA final**
  - Smoke test manual en 3 VMs limpias.
  - Stress test: 500+ entries de audit log.
  - Backwards compat: instalación sobre v0.5 conserva settings.

### Done
- Binario firmado y aceptado por SmartScreen sin warning.
- Web pública con descarga + landing decente.
- `winget install ClearTool.ClearTool` funciona.
- Auto-update de `v0.9-rc1` → `v1.0.0` sin intervención.

---

## Post-1.0 (no comprometido)

Features que están en specs pero NO entran en v1.0:

- Plugin system con APIs externas.
- Análisis ML predictivo.
- Driver cleanup.
- BCD/boot config.
- Diff entre PCs.
- Modo "wizard" para usuarios novatos.
- Microsoft Store distribution.
- macOS / Linux builds (no aplica — Win-only).

Si alguno cobra urgencia, se prioriza en v1.1+.

---

## Timeline visual

```
0.1 ──── 2-3 sem ──── 0.2 ──── 4-5 sem ──── 0.3 ──── 4-6 sem ──── 0.4 ──── 3-4 sem ──── 0.5(RC) ──── 2-3 sem ──── 1.0
       (M1: UX        (M2: Cache              (M3: Contenido        (M4: UI               (M5: Distribución
        no sangra)     funcional)              + módulos)            refactor)             + signing)
```

Total: **15-21 semanas** ≈ **4-5 meses** de trabajo enfocado a tiempo parcial. Más realista en 5-6 meses con interrupciones.

## Capacidad y supuestos

Este timeline asume:
- **Un desarrollador principal** trabajando ~10-15h/semana.
- **Sin pivots** sobre el spec v2.
- **Curación de catálogo en paralelo** (16-24h humanas para el debloat).
- **Acceso a 3 VMs Windows 11** para testing (OEM Lenovo, OEM HP, clean ISO).

Si la dedicación es full-time (40h/sem), aproximadamente **9-12 semanas** total.

## Checkpoints de decisión

Al final de cada milestone, **evaluar**:
1. ¿El milestone está realmente "done" según su criterio?
2. ¿Hay deuda técnica que hay que pagar antes del siguiente milestone?
3. ¿Hay bugs P0 nuevos descubiertos en testing que bloqueen avance?

Si la respuesta a 1 es "no", **no avanzar al siguiente milestone**. Bloomscope > velocity.

## Riesgos identificados

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Restart Manager API más complejo de lo esperado | Media | Alto | Spike de 2 días al inicio de M2; si falla, fallback a `handle.exe` distribuido |
| Azure Trusted Signing tarda en aprobar identity | Alta | Medio | Empezar el proceso desde M3, no esperar a M5 |
| Catálogo Debloat curación más lenta que estimado | Alta | Medio | Empezar curación incremental ya en M2; aceptar 80 entradas en M3 si 120 es too much |
| Win 11 24H2 cambia APIs que rompen módulos | Baja | Alto | Test continuo en VM 24H2 desde M2; flag de features con detección de build |
| Auto-updater complicado de probar | Media | Medio | Beta channel desde M3 con usuarios reales testeando upgrades |

## Métricas que medimos

Durante el desarrollo:
- LOC añadidas/eliminadas por milestone.
- Bugs cerrados vs abiertos por milestone.
- Tests añadidos por milestone.
- Tiempo desde commit a release artifact (debería bajar con CI).

Post-launch:
- Downloads GitHub Releases.
- Open issues / closed issues ratio.
- Critical bugs reportados (target: 0).
- Time to first response en issues.
