# 13 — Lanzamiento v1.0 (checklist final + comunicación + post-mortem)

> **Posición:** 13/14 — último archivo del plan.
> **Dependencias:** 00-12 cerrados.
> **Output:** checklist pre-release ejecutado, comunicación pública preparada, post-mortem template listo para post-lanzamiento.

---

## 1. Resumen ejecutivo

Cuando este archivo está completamente en verde, ClearTool 1.0.0 puede salir al mundo.

Estructura:

1. **Pre-flight checklist** — verificación exhaustiva pre-tag.
2. **Release execution** — secuencia exacta del día del lanzamiento.
3. **Comunicación pública** — README, sitio, redes, comunidades.
4. **Post-lanzamiento** — monitoreo, respuesta a issues, primer parche.
5. **Template de post-mortem** — para v1.0.1.

---

## 2. Pre-flight checklist

### 2.1 Código

- [ ] Branch `release/v1.0.0` cortado desde `main` (o desde la rama de integración correspondiente).
- [ ] Versión sincronizada en:
  - [ ] `src-tauri/Cargo.toml` — `version = "1.0.0"`
  - [ ] `src-tauri/tauri.conf.json` — `"version": "1.0.0"`
  - [ ] `package.json` — `"version": "1.0.0"`
- [ ] Script `scripts/check-version-sync.ps1` pasa.
- [ ] `CHANGELOG.md` actualizado con sección v1.0.0.
- [ ] `LICENSE`, `PRIVACY.md`, `INSTALL.md`, `THIRD-PARTY-NOTICES.md` revisados.
- [ ] `README.md` del repo actualizado a la versión final (screenshots, link al release, badge de licencia).

### 2.2 Catálogos

- [ ] `cache-locations.json` ≥ 30 entries curadas.
- [ ] `bloatware-catalog.json` ≥ 50 entries (cubre Win11 23H2 + 24H2).
- [ ] `services-catalog.json` ≥ 58 entries.
- [ ] `registry-tweaks.json` ≥ 45 entries.
- [ ] Schemas JSON publicados en repo + accesibles vía URL.
- [ ] Tests de parseo pasan en CI.

### 2.3 Backend

- [ ] Todos los IPC commands respondiendo (lista en `src-tauri/src/lib.rs::run` invoke_handler).
- [ ] `validate_all_at_startup` se ejecuta sin panic.
- [ ] `settings::init` se llama antes de cualquier comando.
- [ ] No hay `NotImplemented` rebote en path real.
- [ ] `cargo clippy -- -D warnings` pasa.
- [ ] `cargo fmt --check` pasa.
- [ ] `cargo test --release` pasa.

### 2.4 Frontend

- [ ] `npx tsc --noEmit` pasa sin errores.
- [ ] `npx vitest run` pasa (mínimo 5 tests, idealmente >15).
- [ ] Build de producción `npm run build` exitoso.
- [ ] No hay `console.log` orfans (verificar con grep).
- [ ] No hay `TODO:` críticos pendientes (los aceptados están en `08-SEGURIDAD-FINAL §6`).
- [ ] Bundle final < 5 MB (compress chunk).

### 2.5 Seguridad ([08-SEGURIDAD-FINAL](08-SEGURIDAD-FINAL.md))

- [ ] Los 87 items del checklist en verde.
- [ ] `issues-seguridad.md` solo contiene issues categorizados como "aceptados v1.0".
- [ ] Subagente `security-auditor` invocado: veredicto `release_blocked: false`.
- [ ] Tests adversarial `8/8` pasan.

### 2.6 Tests ([09-TESTS](09-TESTS.md))

- [ ] CI en GitHub Actions verde en `main`.
- [ ] QA manual en VM virgen ejecutado al menos 2 veces (con tester distinto si es posible).
- [ ] Evidencia archivada en `qa-evidence/v1.0.0/`.
- [ ] Sin regresiones en flujos golden path: cache clean, registry apply, debloat preset, services preset.

### 2.7 Distribución ([10-DISTRIBUCION](10-DISTRIBUCION.md))

- [ ] `ClearTool_1.0.0_x64-setup.exe` builds limpio.
- [ ] `ClearTool_1.0.0_x64_en-US.msi` builds limpio.
- [ ] SHA256 calculados y archivados.
- [ ] Iconos en alta resolución verificados visualmente.
- [ ] Instalación en VM virgen → desinstalación limpia (sin residuos en Add/Remove).
- [ ] Manifest `requireAdministrator` confirmado en `signtool /verify /pa` (sin firma, pero estructura OK).

### 2.8 Auto-update ([11-AUTO-UPDATE](11-AUTO-UPDATE.md))

- [ ] Clave Ed25519 generada, privada en password manager + backup offline.
- [ ] `tauri.conf.json` con pubkey final.
- [ ] Repo `cleartool-updates` configurado con CNAME → GitHub Pages.
- [ ] DNS `updates.cleartool.app` resolviendo.
- [ ] Workflow `publish-update.yml` testeado con release dummy.
- [ ] Test de update local end-to-end con manifest localhost OK.

### 2.9 I18n + Onboarding ([12-I18N-ONBOARDING](12-I18N-ONBOARDING.md))

- [ ] `es.json` y `en.json` con misma cobertura de keys.
- [ ] Test `i18n coverage` pasa.
- [ ] Onboarding wizard se muestra al primer arranque en VM virgen.
- [ ] Botón "Ver onboarding de nuevo" funcional.
- [ ] Navegación por teclado completa en todas las pantallas (tab order razonable).
- [ ] Contraste WCAG AA verificado.

### 2.10 Git hygiene

- [ ] No hay commits `WIP`, `fixup`, o mensajes con typos en historia de `main`.
- [ ] Tag `v1.0.0` apunta al commit final, firmado si es posible.
- [ ] Branch `release/v1.0.0` mergeada a `main`.
- [ ] `.gitignore` no incluye archivos críticos por error.
- [ ] `.git/` no contiene archivos > 10 MB (verificar con `git rev-list --objects --all | sort -k 2 | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | awk '{print $3, $4}' | sort -nr | head`).

---

## 3. Secuencia del día del lanzamiento

> Ventana sugerida: martes o miércoles AM (mejor tráfico de comunidades técnicas). Evitar viernes.

### 3.1 T-2 días

- [ ] Pre-flight checklist 100% verde.
- [ ] Release notes finales escritas.
- [ ] Screenshot set actualizado (dashboard, cache, debloat, registry, audit).
- [ ] GIF/video corto del flujo principal (≤30s, opcional pero impactante).

### 3.2 T-1 día

- [ ] Build final con `scripts/build-release.ps1`.
- [ ] Smoke test en 2 VMs frescas (una con Win11 22H2, otra 24H2).
- [ ] Borrador de release en GitHub (sin publicar todavía).
- [ ] Anuncios redactados (no enviados).

### 3.3 Día D

#### 3.3.1 Mañana — publicación

1. **9:00 — Tag y push:**
   ```bash
   git tag -s v1.0.0 -m "ClearTool 1.0.0"
   git push origin v1.0.0
   ```
2. **9:05 — Verificar workflows:**
   - `release.yml` corre, genera artifacts.
   - `publish-update.yml` actualiza el manifest server.
3. **9:30 — Publicar release en GitHub** (descheckear "draft"):
   - Adjuntar `.exe` + `.msi` + `.sha256.txt`.
   - Body de release con notas + checksum + instrucciones SmartScreen.
4. **9:45 — Verificación post-publish:**
   - Descargar `.exe` desde URL de release pública.
   - Verificar SHA256 contra publicado.
   - Instalar en VM virgen, ejecutar smoke test rápido (Dashboard pinta, sin errores en log).
5. **10:00 — Actualizar README principal:**
   - Badge de versión.
   - Link al release.
   - Screenshot final.

#### 3.3.2 Mañana — comunicación

6. **10:15 — README final commit & push.**
7. **10:30 — Post en comunidades** (orden sugerido):
   - [ ] r/Windows11 (subreddit principal).
   - [ ] r/software (segundo tier).
   - [ ] Hacker News "Show HN: ClearTool — open-source debloat + cleaner for Win11".
   - [ ] X/Twitter (si hay cuenta del proyecto).
   - [ ] LinkedIn (perfil del autor — networking técnico).
   - [ ] Foros de Windows enthusiasts (BetaArchive, MajorGeeks si aplica).

#### 3.3.3 Tarde — soporte

8. **A partir de las 12:00 — monitoreo:**
   - Refrescar issues del repo cada hora.
   - Refrescar comments de los posts.
   - Responder solo a problemas técnicos críticos. Marketing fluff (gracias, ¿soportará Y?) → respuesta en batch al día siguiente.
9. **Atender issues "P0" (crashes en instalación/primer lanzamiento):**
   - Reproducir.
   - Si es real: hotfix branch + parche v1.0.1 dentro de 24h.

### 3.4 D+1 a D+7

- [ ] Monitor diario de issues.
- [ ] Compilar feedback temprano en `docs/POST-MORTEM-v1.0.md`.
- [ ] Si hay > 3 reportes del mismo bug → priorizar para v1.0.1.
- [ ] Agradecimientos individuales a contribuidores tempranos.

---

## 4. Plantilla de release notes (rellenar para v1.0.0)

```markdown
# ClearTool 1.0.0 — First public release

**Fecha:** 2026-MM-DD

## Resumen

Primera versión pública de ClearTool: utilidad open-source para Windows 11 que combina limpieza profunda de caché, debloat agresivo del ecosistema Microsoft, gestión de servicios y tweaks de registro — todo con punto de restauración automático y log de auditoría reversible.

## Características

- **Dashboard 3D** en tiempo real con CPU, RAM, GPU, disco y procesos top.
- **Explorador de directorios** con árbol jerárquico, virtualización, y treemap (post v1.0).
- **Limpieza de caché** con 30+ ubicaciones curadas (Sistema, Usuario, Navegadores, Package managers).
- **Debloat** de 60+ apps preinstaladas (Cortana, Bing News, Spotify preinstall, etc.) con presets Mínimo/Recomendado/Total.
- **Servicios** con 58 entradas curadas tuneables, dependencias visualizadas.
- **Tweaks de registro** con 45 entradas curadas, diff viewer, batch con progress.
- **Restore points** integrados como red de seguridad obligatoria.
- **Audit log** JSONL con reverse recipes por operación.
- **Sin telemetría**, sin tracking, sin ads. Open-source MIT.
- **ES/EN** desde el día uno.

## Compatibilidad

- Windows 11 22H2+ (build 22621+).
- 200 MB disco libre, 4 GB RAM mínimo.
- WebView2 Runtime (preinstalado en Win11).

## Verificación de integridad

SHA-256 del instalador:
```
{{SHA256_NSIS}}
```

SHA-256 del MSI:
```
{{SHA256_MSI}}
```

## SmartScreen

Esta versión **no está firmada digitalmente**. Windows mostrará advertencia al primer lanzamiento. Click "More info" → "Run anyway". Consulta [INSTALL.md](docs/INSTALL.md) si necesitás guía paso a paso.

## Issues conocidos

Documentados en `docs/PLAN-FINAL/08-SEGURIDAD-FINAL.md §6`:

- `AutomaticDelayed` se reporta como `Automatic` (visual; el comportamiento del servicio es correcto).
- No hay detección de procesos que bloquean archivos del cache → archivos en uso van a reboot.
- No hay compresión gzip del audit log archivado.

Plan para v1.1: ver `docs/PLAN-FINAL/13-LANZAMIENTO.md §6`.

## Agradecimientos

A la comunidad open-source: tauri-apps, windows-rs, react, tanstack, framer-motion, lucide, shadcn/ui, y los curadores de listas como Sycnex/Windows10Debloater (referencia histórica para el catálogo de bloatware).

## Links

- [Documentación](https://github.com/cleartool/cleartool/blob/main/docs/)
- [Reportar bug](https://github.com/cleartool/cleartool/issues/new?template=bug.md)
- [Solicitar feature](https://github.com/cleartool/cleartool/issues/new?template=feature.md)
- [Discussions](https://github.com/cleartool/cleartool/discussions)
```

---

## 5. Anuncios — drafts por canal

### 5.1 Reddit (`r/Windows11`)

**Título:**
```
[Free, Open-Source] ClearTool 1.0 — Cache cleaner, debloat y registry tweaks para Win11, sin telemetría
```

**Body:**
```
Hola r/Windows11,

Acabo de publicar ClearTool 1.0, una utilidad de escritorio open-source que combina limpieza profunda de caché de Windows 11, debloat de apps preinstaladas (Cortana, Bing News, etc.) y tweaks de registro. Lo hice porque las opciones existentes son o muy viejas (CCleaner clásico), o sospechosas (telemetría escondida), o solo line-CLI (PowerShell scripts).

Diseño explícito:
- **Cero telemetría.** Cero llamadas salientes salvo update check opcional.
- **Restore point obligatorio** antes de cualquier operación destructiva.
- **Audit log local** en `%APPDATA%\ClearTool\audit.jsonl` con instrucciones de reversa por operación.
- **Open-source MIT** — el código está en GitHub.
- **No es Electron** — Tauri + Rust + React, peso ~10MB.

Stack: Tauri 2.x + Rust (backend) + React + TS (frontend).

[Screenshot del dashboard]
[Screenshot del debloat]

⚠ El installer no está firmado (sin recursos para EV cert aún). Windows SmartScreen mostrará warning — Click "More info" → "Run anyway". Hash SHA256 publicado en el release para verificación manual.

Link: github.com/cleartool/cleartool/releases/tag/v1.0.0

Cualquier feedback / bug report bienvenido. Cubre Win11 22H2 y 24H2.
```

### 5.2 Hacker News

**Title:** `Show HN: ClearTool – Open-source Win11 cleaner with restore points and audit log`

**Body:**
```
ClearTool is a Tauri + Rust + React desktop app for Windows 11 that combines cache cleanup, bloatware removal, registry tweaks and service management. Released v1.0 today after ~6 months of work.

Design constraints I imposed on myself:

- No telemetry. The only outbound call is an opt-in update check against a JSON manifest I host on GitHub Pages.
- Every destructive operation creates a Restore Point and writes an audit entry with a reverse recipe (JSON describing how to undo). The audit lives in %APPDATA%, not the cloud.
- Allowlists curated by hand: ~60 services, ~45 registry tweaks, ~60 Appx packages. Nothing outside the catalog can be touched.
- PowerShell scripts are include_str!'d as &'static str — no dynamic construction from user input. Args validated via regex before invocation.
- Manifest distinguishes between debug (asInvoker) and release (requireAdministrator) so cargo run doesn't hit "operation requires elevation" hell.

What it actually does:

[link to README screenshots]

What it explicitly doesn't:
- Touch Defender / security settings beyond user-controlled SmartScreen toggle.
- Send any usage data anywhere.
- "Force-fix" anything — every operation requires confirmation and shows exactly what it'll modify.

License: MIT. Source: github.com/cleartool/cleartool

Happy to answer questions about the architecture or the curation process for the catalogs.
```

### 5.3 X / Twitter

**Thread (5 tweets):**

1️⃣ ClearTool 1.0 is out. Open-source Windows 11 cleaner + debloat + registry tweaks. No telemetry. Restore point and audit log for every destructive operation. Tauri + Rust + React, ~10MB binary. [link]

2️⃣ Why I built this:
- CCleaner: ancient codebase, owned by Avast, telemetry.
- BleachBit: solid but UX from 2010.
- PowerShell debloat scripts: powerful but hostile to non-technical users.
- Nothing combined disk cleanup + debloat + tweaks under one transparent UX.

3️⃣ Hard constraints I imposed:
- Curated allowlists for everything (no "let user type any registry path").
- Each destructive op writes a JSON reverse recipe to a local audit log.
- PowerShell scripts are static strings; args validated with regex before invocation.

4️⃣ Stack rationale: Tauri (not Electron) for ~10MB binary instead of 200MB. Rust for FS/registry/services. React for UX. windows-rs for Win32 calls. WMI only where SCM doesn't cut it.

5️⃣ MIT licensed. No premium tier (yet). If you find a bug or have feedback, the GitHub issues are open. [repo link]

### 5.4 LinkedIn

**Post:**
```
Después de unos meses de trabajo, publiqué ClearTool 1.0 — una utilidad open-source de escritorio para Windows 11.

¿Qué problema resuelve?
Las opciones para limpiar/optimizar Win11 hoy son o muy viejas, o con telemetría sospechosa, o solo scripts de línea de comandos. ClearTool combina limpieza de caché, debloat de apps preinstaladas, gestión de servicios y tweaks de registro — todo con transparencia total y red de seguridad (restore points + audit log reversible).

Decisiones técnicas que se vuelven valor:
✔ Tauri en lugar de Electron → binario de ~10MB en vez de 200MB.
✔ Backend en Rust con windows-rs → acceso directo a Win32 APIs sin runtime pesado.
✔ Cero telemetría → única conexión saliente opcional es el check de actualizaciones.
✔ Allowlists curadas → la app no puede tocar nada fuera del catálogo revisado.
✔ Audit log JSONL local → cada operación destructiva tiene reverse recipe.

Open-source MIT. Sin freemium. Sin tracking.

Si trabajás con Windows y querés probarlo, repo en GitHub: [link]

Feedback técnico y bug reports más que bienvenidos.
```

---

## 6. Roadmap post-v1.0

> Esta lista informa lo que viene, NO es parte del DoD de v1.0.

### v1.0.x — parches de mantenimiento (semanas 1-4 post-release)

- Fixes de bugs reportados por la comunidad.
- Ajustes menores de catálogo según feedback.
- Ajustes UX si hay fricciones detectadas.

### v1.1.0 — características pendientes (mes 2-3)

- Detección de procesos que bloquean archivos del cache (Restart Manager API).
- `AutomaticDelayed` detectado correctamente.
- Compresión gzip de audit log archivado.
- Patches incrementales para auto-update (no full bundle).
- `.reg` backup automático antes de tweaks (defensa adicional).
- Treemap visual en el explorador.

### v1.2.0 — escala (mes 4-6)

- Code-signing con certificado OV (cuando descargas > 5k).
- Soporte para distribución vía winget.
- Catálogo de tweaks expandido (más de 70 entries).
- Comunidad: proceso de PRs para nuevos entries del catálogo.
- Modo "scheduler" — limpieza automática semanal opcional.

### v2.0 — visión (año 2+)

- ARM64 build (Windows on ARM).
- Catálogo dinámico via repo public, con curaduría comunitaria reviewada.
- Plugin system para extensiones de terceros.

---

## 7. Plantilla de post-mortem v1.0

**Archivo:** `docs/POST-MORTEM-v1.0.md` (a llenar después del lanzamiento)

```markdown
# Post-mortem ClearTool v1.0

**Fecha de release:** YYYY-MM-DD
**Días observados:** D+0 a D+30

## Métricas

| Métrica | Valor | Objetivo (opcional) |
|---|---|---|
| Descargas en GitHub (30 días) | ? | >2000 |
| Issues abiertos | ? | — |
| Issues cerrados como bugs | ? | — |
| Issues cerrados como features | ? | — |
| Estrellas en GitHub | ? | >500 |
| Forks | ? | >30 |
| Hits Hacker News (front page si aplica) | ? | — |

## Lo que funcionó

- ...

## Lo que NO funcionó / sorpresas

- ...

## Bugs críticos (P0) detectados

| # | Reporte | Fix en versión |
|---|---|---|

## Decisiones que cambiaríamos

- ...

## Top requests de la comunidad

1. ...
2. ...

## Plan ajustado para v1.0.1 / v1.1.0

- ...
```

---

## 8. Crisis playbook (si algo sale mal)

### 8.1 Bug crítico que rompe instalaciones

1. **Inmediato:** Editar release en GitHub, añadir banner ⚠ rojo con detalle del bug.
2. **<1h:** Issue pinned en el repo describiéndolo.
3. **<24h:** Hotfix `v1.0.1` con parche.
4. **Post-mortem público:** transparente sobre causa raíz.

### 8.2 Reporte de vulnerabilidad de seguridad

1. **Política:** archivo `SECURITY.md` en el repo con `security@cleartool.app` (o GitHub Security Advisory).
2. **Tiempo de respuesta:** 48h reconocimiento, 7 días patch en privado, disclosure coordinada.
3. **Reconocimiento:** entrada en `SECURITY.md` con el reporter (si consiente).

### 8.3 Reporte falso de malware (false positive de AV)

1. Verificar con `virustotal.com`.
2. Si genuinamente false positive: reportar al vendor del AV (AVG, Avast, Bitdefender, etc.).
3. Comunicar en el repo: "X AV vendor marca como sospechoso por ser exe sin firma. False positive reportado. Hash SHA256 disponible para verificación independiente."

---

## 9. Definition of Done

- [ ] Pre-flight checklist (§2) 100% verde.
- [ ] Secuencia del día del lanzamiento (§3) ejecutada.
- [ ] Release publicado en GitHub con todos los artifacts.
- [ ] README final commited con badges + screenshots + link al release.
- [ ] Posts en al menos 3 comunidades (Reddit + HN + uno más).
- [ ] Monitoreo D+1 hasta D+7 activo.
- [ ] `POST-MORTEM-v1.0.md` template creado (a rellenar en D+30).
- [ ] `SECURITY.md` en el repo con disclosure policy.
- [ ] `CONTRIBUTING.md` (cómo proponer nuevos catalog entries, etc.).
- [ ] Commit final `chore(release): ClearTool 1.0.0 publicado`.

---

## 10. Cierre del plan

Si llegaste hasta acá y todos los DoD desde [00-CATALOGOS](00-CATALOGOS.md) hasta este archivo están en verde:

**ClearTool 1.0.0 está vivo. Felicidades.**

El siguiente paso natural es escuchar a la comunidad y planificar v1.0.1 / v1.1 basándose en feedback real, no en suposiciones. Mantené el principio que guió todo: **transparencia > velocidad, seguridad > features**.

Buena suerte.

— Cierre del plan, 2026-05-21.
