# Dashboard de Colaboración — ClearTool × GameTrim

> **Propósito.** Guía completa para colaborar en el ecosistema ClearTool/GameTrim entre un desarrollador en Windows (tú) y un colaborador en Linux (tu amigo). Este documento cubre arquitectura, división de trabajo, sinergias de marca, flujo de desarrollo y planificación enfocada en gaming.
>
> **Generado:** 2026-05-21
> **Ramificación esperada:** `main` para ClearTool, `feature/gametim-*` para GameTrim
> **Target:** Ecosistema ClearTool 1.0 + GameTrim 0.1 (MVP gaming)

---

## 1. Qué es ClearTool (contexto obligatorio)

ClearTool es una aplicación de escritorio **exclusiva de Windows 11** que combina:

| Módulo | Qué hace | Estado |
|---|---|---|
| **Explorador de directorios** | Árbol de archivos con tamaños lazy, selector de discos, virtualización | 🟡 Funcional pero con bugs de expansión |
| **Limpieza de caché** | Escanea y borra 50+ ubicaciones de caché del sistema | ✅ Funcional (escaneo + clean) |
| **Debloat** | Desinstala apps preinstaladas (Appx) con catálogo curado | ❌ Stub |
| **Gestión de servicios** | Habilita/deshabilita servicios de Windows con presets | ❌ Stub |
| **Tweaks del registro** | 45+ tweaks reversibles del registro | ❌ Stub |
| **Restore Points** | Crea/lista/restaura puntos de restauración del sistema | ❌ Stub |
| **Dashboard 3D** | Telemetría en vivo (CPU, RAM, GPU, discos) con render R3F | ✅ Funcional |
| **Audit Log** | Registro de operaciones con recetas de reversa | ❌ Stub |

**Stack:** Tauri 2.x + Rust (backend) + React/TypeScript (frontend) + Tailwind + shadcn/ui
**Principios:** seguridad antes que velocidad, transparencia total, idempotencia, dry-run obligatorio, restore point antes de cualquier operación destructiva.

---

## 2. Qué es GameTrim (la sinergia gaming)

GameTrim es un **optimizador de Windows específico para gaming**, hermano de ClearTool. Reutiliza ~70% del backend de ClearTool pero con identidad, catálogo y UX orientados al gamer.

### 2.1 Features principales

| Feature | Descripción | Backend compartido con ClearTool |
|---|---|---|
| **Modo Juego** | Cierra procesos en background, libera RAM agresivamente, ajusta plan de energía | ✅ (servicios, procesos, registro) |
| **Limpieza de Shader Cache** | DirectX, Vulkan, NVIDIA, AMD shader caches | ✅ (motor de caché, nuevos catálogos) |
| **Liberación de VRAM** | Detecta y libera memoria de GPU no usada | 🟡 (GPU telemetry ya existe) |
| **Deshabilitar overlays** | Game Bar, Xbox overlay, Discord overlay, NVIDIA overlay | ✅ (servicios + registro) |
| **Perfiles por juego** | Configuración automática al detectar qué juego se lanza | ❌ (nuevo) |
| **Optimización CPU/GPU** | Programación de CPU, GPU scheduling, power plan gaming | 🟡 (registro tweaks) |
| **Benchmark integrado** | FPS antes/después de la optimización | ❌ (nuevo) |

### 2.2 Mercado

- ~1.500 millones de gamers PC en el mundo
- Nicho "PC optimization for gaming" con volumen brutal de búsqueda
- Competencia floja: Razer Cortex (feo, intrusivo), Process Lasso (UI de 2005), MSI Afterburner (solo monitoring)
- Espacio enorme para algo **bonito, transparente y serio**

### 2.3 Monetización

- Free + Pro ($14.99 one-time). Pro: perfiles por juego, auto-detección, benchmarks.
- Afiliados de hardware (Amazon: RAM, SSDs, GPUs)
- Patrocinios de marcas gaming (NZXT, Corsair) con 50k+ users

---

## 3. El problema: Linux vs Windows

**Tauri no compila binarios Windows en Linux.** Pero esto NO impide la colaboración. La división natural es:

| Capa | ¿Funciona en Linux? | ¿Quién la trabaja? |
|---|---|---|
| **Frontend React/TypeScript** | ✅ `npm run dev` funciona perfecto | Tu amigo (Linux) |
| **Rust — lógica agnóstica** | ✅ `cargo test`, `cargo check` funcionan | Ambos |
| **Rust — Win32 APIs** | ❌ Solo compila en Windows con MSVC | Tú (Windows) |
| **Catálogos JSON** | ✅ Archivos de texto, editables en cualquier lado | Ambos |
| **Branding / Diseño** | ✅ Figma, assets, paletas, mockups | Tu amigo (Linux) |
| **Documentación / Specs** | ✅ Markdown en el repo | Ambos |
| **Sitio web SEO** | ✅ Astro/Next.js, 100% cross-platform | Tu amigo (Linux) |
| **Instalador / Build final** | ❌ Solo en Windows | Tú (Windows) |

---

## 4. División de trabajo concreta

### 4.1 Tu amigo (Linux) — "Frontend + Branding + Web"

#### A. Branding de GameTrim
- **Logo:** diseño del logo de GameTrim (identidad gaming, distinta de ClearTool pero del mismo ecosistema)
- **Paleta de colores:** tema gaming (cyan/magenta/neon sobre oscuro) — tokens CSS en `globals.css`
- **Tipografía:** selección de fuentes para UI gaming (JetBrains Mono para números, Inter/Geist para texto)
- **Iconos:** set de iconos SVG para las features de GameTrim (shader cache, VRAM, modo juego, etc.)
- **Mockups:** pantallas completas de GameTrim en Figma antes de implementar

**Entregables:**
```
public/assets/gametim/
├── logo.svg
├── logo-dark.svg
├── icons/
│   ├── shader-cache.svg
│   ├── vram.svg
│   ├── game-mode.svg
│   ├── overlay-toggle.svg
│   └── benchmark.svg
└── brand-guidelines.md
```

#### B. Frontend React de GameTrim
Usando el mismo stack que ClearTool (Tailwind + shadcn/ui) pero con tema gaming:

| Pantalla | Descripción | Prioridad |
|---|---|---|
| `gametim-home` | Dashboard gaming con anillo 3D de carga global, FPS target, estado de optimización | Alta |
| `gametim-game-picker` | Selector/detector de juegos instalados (Steam, Epic, GOG, Xbox) | Alta |
| `gametim-profiles` | Gestión de perfiles por juego (qué cerrar, qué optimizar) | Alta |
| `gametim-optimizations` | Lista de optimizaciones aplicables con toggles y dry-run preview | Alta |
| `gametim-benchmark` | Panel de benchmark FPS antes/después | Media |
| `gametim-settings` | Configuración global, auto-detección, hotkey para modo juego | Media |

**Estructura de archivos:**
```
src/features/gametim/
├── home-page.tsx
├── game-picker.tsx
├── profiles-page.tsx
├── optimizations-page.tsx
├── benchmark-page.tsx
├── components/
│   ├── game-card.tsx
│   ├── profile-editor.tsx
│   ├── optimization-toggle.tsx
│   ├── fps-meter.tsx
│   └── game-mode-indicator.tsx
└── hooks/
    ├── use-game-detection.ts
    ├── use-profiles.ts
    └── use-benchmark.ts
```

#### C. Catálogos Gaming (JSON)
Curar las ubicaciones y configuraciones específicas de gaming:

| Catálogo | Ubicación | Contenido |
|---|---|---|
| `shader-cache-locations.json` | `.claude/skills/cache-scanner/RESOURCES/` | DirectX, Vulkan, NVIDIA, AMD shader cache paths |
| `gaming-services-catalog.json` | `.claude/skills/powershell-debloat/RESOURCES/` | Servicios que afectan gaming (Game Bar, Xbox, etc.) |
| `gaming-registry-tweaks.json` | `.claude/skills/windows-registry-ops/RESOURCES/` | Tweaks de rendimiento para gaming |
| `game-presets.json` | `src-tauri/resources/gametim/` | Perfiles predefinidos por tipo de juego (FPS, RPG, MMO, etc.) |

#### D. Sitio Web SEO
- **cleartool.app** — landing de ClearTool
- **windowsoptimizer.guide** — blog/tutorial con contenido que atrae tráfico orgánico
- Artículos como "Cómo optimizar Windows 11 para gaming en 2026", "Lista de servicios que puedes deshabilitar sin romper tu PC"
- Comparativas: "GameTrim vs Razer Cortex", "ClearTool vs CCleaner"

### 4.2 Tú (Windows) — "Backend + Win32 + Integración"

#### A. Comandos Tauri pendientes de ClearTool
Según el PLAN-FINAL, estos son los módulos que necesitas implementar:

| Módulo | Archivo | Prioridad |
|---|---|---|
| Restore Points | `domain/restore.rs` + `platform/restore_point.rs` | 🔴 Crítica (prerequisito de todo lo destructivo) |
| Audit Log | `domain/audit.rs` | 🔴 Crítica |
| Registry Tweaks | `domain/registry.rs` + `platform/registry.rs` | Alta |
| Services | `domain/services.rs` + `platform/services.rs` | Alta |
| Debloat | `domain/debloat.rs` + `platform/powershell.rs` | Alta |

#### B. Backend compartido con GameTrim
Módulos Rust que funcionan para ambos productos:

| Módulo | Qué hace | Reutilización |
|---|---|---|
| `telemetry.rs` | CPU, RAM, GPU en vivo | 100% compartido |
| `cache.rs` | Motor de escaneo y limpieza de caché | 90% (nuevos catálogos) |
| `services.rs` | Gestión de servicios del SCM | 80% (nuevos presets gaming) |
| `registry.rs` | Tweaks del registro | 70% (nuevos tweaks gaming) |
| `process_manager.rs` | Listar/matar procesos | 100% compartido |
| `power_plan.rs` | Cambiar plan de energía | Nuevo para GameTrim |

#### C. Integración y Build
- Unir el frontend de GameTrim con el backend Windows
- Validar que todo funciona en Windows real
- Build del instalador NSIS
- Testing en VM Windows 11 limpia

---

## 5. Sinergia de marca — El Ecosistema

### 5.1 Arquitectura de marca

```
                    ┌─────────────────────┐
                    │   ClearTool Brand   │  ← Marca paraguas
                    │   cleartool.app     │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
    ┌─────────▼──────┐  ┌─────▼──────┐  ┌──────▼─────────┐
    │   ClearTool    │  │  GameTrim  │  │  SEO Web Site  │
    │   (general)    │  │  (gaming)  │  │  windowsopt.   │
    │   $14.99 Pro   │  │  $14.99    │  │  guide (ads)   │
    └────────────────┘  └────────────┘  └────────────────┘
```

### 5.2 Principios de marca compartida

| Principio | ClearTool | GameTrim |
|---|---|---|
| **Look** | Profesional, cyan/violeta, glassmorphism | Gaming, cyan/magenta/neon, más agresivo |
| **Tono** | Serio, técnico, transparente | Enérgico, gamer, pero sin ser childish |
| **Valores** | Privacidad, control, rendimiento | Rendimiento máximo, FPS, sin bloat |
| **Logo** | Herramienta/limpieza abstracta | Rayo/velocidad/gamepad abstracto |
| **Tipografía** | Inter + JetBrains Mono | Misma, pero con acentos de color neón |

### 5.3 Cross-promoción

- ClearTool tiene un link "¿Gamer? Prueba GameTrim →" en Settings
- GameTrim tiene un link "¿Limpieza general? Prueba ClearTool →" en Settings
- Ambos linkean a `windowsoptimizer.guide` para contenido educativo
- Discord común para la comunidad
- Newsletter compartida

### 5.4 Proyección de ingresos (escenario realista a 24 meses)

| Fuente | Proyección |
|---|---|
| ClearTool Pro: 2.000 licencias × $14.99 | $30.000 |
| GameTrim Pro: 5.000 licencias × $14.99 | $75.000 |
| Sitio web (150k visitas/mes con Mediavine) | $42.000/año |
| Afiliados Amazon hardware | $6.000–18.000/año |
| **Total año 2** | **$120k–180k** |

---

## 6. Flujo de desarrollo

### 6.1 Setup del repositorio

```bash
# Tu amigo (Linux):
git clone <repo-url>
cd ClearTool
npm install
cargo check          # Verifica que Rust compila
npx tsc --noEmit     # Verifica que TypeScript tipa
npm run dev          # Dev server del frontend (Vite)

# Tú (Windows):
git clone <repo-url>
cd ClearTool
npm install
npm run tauri dev    # App completa con backend Tauri
```

### 6.2 Estrategia de ramas

```
main ──────────────────────────────────────────────── (ClearTool estable)
  ├── dev ─────────────────────────────────────────── (integración continua)
  │   ├── feature/clear-restore-points ────────────── (tú)
  │   ├── feature/clear-audit-log ─────────────────── (tú)
  │   ├── feature/clear-debloat ───────────────────── (tú)
  │   ├── feature/gametim-brand ───────────────────── (tu amigo)
  │   ├── feature/gametim-ui ──────────────────────── (tu amigo)
  │   └── feature/gametim-catalogs ────────────────── (ambos)
  └── release/1.0.0 ───────────────────────────────── (pre-release)
```

**Reglas:**
- `main` solo recibe merges de `dev` cuando está estable
- Cada feature va en su propia rama
- PRs requieren `cargo check` + `tsc --noEmit` + `npm run build` pasando
- Commits siguen Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`)

### 6.3 Comunicación

| Canal | Uso |
|---|---|
| **GitHub Issues** | Bugs, features, discusión técnica |
| **GitHub Projects** | Kanban board con tareas asignadas |
| **Discord** | Comunicación diaria, screenshots, dudas rápidas |
| **Pull Requests** | Review de código, discusión de implementaciones |
| **Docs en repo** | Specs, planes, decisiones arquitectónicas |

### 6.4 Ciclo de trabajo semanal

| Día | Tú (Windows) | Tu amigo (Linux) |
|---|---|---|
| **Lunes** | Planning semanal, revisar PRs pendientes | Diseño de la semana, mockups nuevos |
| **Mar-Mié** | Implementar backend (restore, audit, registry) | Implementar frontend React, componentes |
| **Jueves** | Testing en Windows, fix de bugs Win32 | Catálogos JSON, artículos SEO |
| **Viernes** | Code review, merge de PRs, demo | Code review, ajustes de UI/UX |
| **Sáb-Dom** | Opcional: investigación, lectura | Opcional: diseño libre, brainstorming |

---

## 7. Plan de acción — Primeras 4 semanas

### Semana 1 — Setup y Branding

| Quién | Tarea | Entregable |
|---|---|---|
| Ambos | Configurar repo, ramas, Discord, GitHub Project | Repo funcional para los dos |
| Tu amigo | Diseñar logo de GameTrim, paleta de colores, iconos | Assets en `public/assets/gametim/` |
| Tu amigo | Explorar estructura del frontend de ClearTool | Familiarización con el código |
| Tú | Implementar `domain/restore.rs` (restore points) | Módulo funcional en Windows |
| Tú | Implementar `domain/audit.rs` (audit log) | Módulo funcional |

### Semana 2 — Frontend GameTrim + Backend ClearTool

| Quién | Tarea | Entregable |
|---|---|---|
| Tu amigo | Crear `gametim-home-page.tsx` (dashboard gaming) | Pantalla funcional en `npm run dev` |
| Tu amigo | Crear `gametim-game-picker.tsx` (selector de juegos) | Componente UI |
| Tu amigo | Curar `shader-cache-locations.json` | Catálogo con 15+ entradas |
| Tú | Implementar `domain/registry.rs` (tweaks del registro) | Módulo funcional |
| Tú | Implementar `domain/services.rs` (gestión de servicios) | Módulo funcional |

### Semana 3 — Catálogos Gaming + Debloat

| Quién | Tarea | Entregable |
|---|---|---|
| Tu amigo | Crear `gametim-profiles-page.tsx` | Pantalla de perfiles |
| Tu amigo | Crear `gametim-optimizations-page.tsx` | Pantalla de optimizaciones |
| Tu amigo | Curar `gaming-services-catalog.json` | Servicios gaming |
| Tu amigo | Curar `gaming-registry-tweaks.json` | Tweaks gaming |
| Tú | Implementar `domain/debloat.rs` (Appx + PowerShell) | Módulo funcional |
| Ambos | Alinear contratos Rust ↔ TypeScript de GameTrim | IPC commands definidos |

### Semana 4 — Integración y Primer MVP

| Quién | Tarea | Entregable |
|---|---|---|
| Tú | Integrar frontend GameTrim con backend Windows | App GameTrim funcional |
| Tú | Implementar `power_plan.rs` (cambio de plan de energía) | Comando Tauri nuevo |
| Tu amigo | Primer artículo SEO: "Cómo optimizar Windows 11 para gaming" | Publicado en windowsoptimizer.guide |
| Tu amigo | Ajustar UI/UX de GameTrim según feedback | Iteración de diseño |
| Ambos | Demo del MVP, planificación de Semana 5-8 | Decidir próximos pasos |

---

## 8. Mapa del repositorio (lo que tu amigo necesita conocer)

```
ClearTool/
├── CLAUDE.md                              ← Instrucciones del proyecto (leer primero)
├── docs/
│   ├── HISTORIAL-SESIONES.md              ← Bitácora de trabajo
│   ├── FuturosProyectos.md                ← Plan estratégico (GameTrim está aquí)
│   ├── AUDIT-FALLOS-CRITICOS.md           ← Bugs conocidos y estado
│   ├── PLAN-DASHBOARD-3D.md               ← Plan del dashboard con 3D
│   ├── PLAN-EXPLORER-REDISENIO.md         ← Plan del explorador de archivos
│   ├── PLAN-CACHE-EXPLORER.md             ← Plan de caché
│   ├── dashboard-colaboracion.md          ← ESTE ARCHIVO
│   └── PLAN-FINAL/                        ← Plan completo para v1.0
│       ├── README.md                      ← Índice del plan
│       ├── 00-CATALOGOS.md                ← Catálogos y schemas
│       ├── 01-RESTORE-POINTS.md           ← Sistema de restore points
│       ├── 02-AUDIT-LOG.md                ← Audit log con reverse recipes
│       ├── 03-REGISTRY.md                 ← Tweaks del registro
│       ├── 04-SERVICES.md                 ← Gestión de servicios
│       ├── 05-DEBLOAT.md                  ← Motor de debloat
│       ├── 06-CACHE-FINAL.md              ← Limpieza de caché
│       ├── 07-SETTINGS.md                 ← Pantalla de settings
│       └── 08-SEGURIDAD-FINAL.md          ← Auditoría de seguridad (GATE)
│
├── src/                                   ← Frontend React (FUNCIONA EN LINUX)
│   ├── api/                               ← Capa de IPC con Tauri
│   │   ├── client.ts                      ← Comandos Tauri tipados
│   │   ├── events.ts                      ← Eventos tipados
│   │   └── types.ts                       ← Tipos TypeScript
│   ├── features/                          ← Pantallas por módulo
│   │   ├── home/                          ← Dashboard 3D
│   │   ├── explorer/                      ← Explorador de archivos
│   │   ├── cache-cleaner/                 ← Limpieza de caché
│   │   ├── debloat/                       ← Debloat
│   │   ├── services/                      ← Servicios
│   │   ├── registry-tweaks/               ← Tweaks del registro
│   │   ├── restore-points/                ← Restore points
│   │   └── settings/                      ← Configuración
│   ├── components/                        ← Componentes reutilizables
│   ├── hooks/                             ← Custom hooks
│   ├── styles/globals.css                 ← Tokens CSS y paleta
│   └── App.tsx, router.tsx, main.tsx
│
├── src-tauri/                             ← Backend Rust (SOLO COMPILE EN WINDOWS)
│   ├── src/
│   │   ├── lib.rs                         ← Bootstrap de Tauri
│   │   ├── core/                          ← Error handling, config
│   │   ├── platform/                      ← Win32 APIs (elevation, filesystem, registry...)
│   │   ├── domain/                        ← Lógica de negocio (cache, explorer, telemetry...)
│   │   ├── ipc/                           ← Comandos Tauri (wrappers de domain)
│   │   └── models/                        ← DTOs serializables
│   ├── resources/                         ← JSON embebidos en binario
│   ├── capabilities/default.json          ← Permisos de plugins
│   └── Cargo.toml
│
├── .claude/                               ← Skills y agents de Claude Code
│   ├── skills/
│   │   ├── cache-scanner/RESOURCES/cache-locations.json
│   │   ├── powershell-debloat/RESOURCES/bloatware-catalog.json
│   │   ├── powershell-debloat/RESOURCES/services-catalog.json
│   │   └── windows-registry-ops/RESOURCES/registry-tweaks.json
│   └── agents/
│
├── package.json
├── tsconfig.json
└── tailwind.config.js
```

---

## 9. Convenciones de desarrollo

### 9.1 Idioma y casing

| Contexto | Convención | Ejemplo |
|---|---|---|
| Comentarios y docs | Español | `// Escanea la carpeta de caché` |
| Identificadores Rust | `snake_case` | `scan_cache_locations` |
| Identificadores TS | `camelCase` | `scanCacheLocations` |
| JSON (catálogos) | `camelCase` | `{"displayName": "Windows Update"}` |
| Commits | Conventional Commits | `feat(cache): add shader cache locations` |
| Nombres de ramas | kebab-case | `feature/gametim-brand` |

### 9.2 Errores

- **Backend Rust:** tipos `thiserror` en `core::error::AppError`. Nunca `unwrap()` en lib.
- **Frontend TS:** `try { invoke }` con boundary del componente, mostrar error al usuario.

### 9.3 Eventos Tauri

- Naming: `<modulo>:<evento>` (kebab-case)
- Todo evento de operación lleva `runId`/`scanId` para descartar zombies
- Ejemplo: `cache:progress`, `explorer:node`, `gametim:optimization-done`

### 9.4 Comandos Tauri

- Async siempre (`#[tauri::command] async fn`)
- Validación de input contra allowlist en la primera línea
- Dry-run obligatorio en cualquier comando destructivo

### 9.5 Principios de seguridad (NO NEGOCIABLES)

1. **Restore point antes de cualquier operación destructiva** — obligatorio, no configurable
2. **Allowlists obligatorias** — nada fuera del catálogo se puede tocar
3. **Transparencia total** — la UI muestra exactamente qué se va a hacer antes de hacerlo
4. **Idempotencia** — ejecutar la misma acción dos veces no rompe nada
5. **Privilegios mínimos** — solo escalar a admin cuando sea necesario

---

## 10. Herramientas recomendadas para tu amigo (Linux)

| Herramienta | Para qué | Alternativa |
|---|---|---|
| **VS Code** | Editor principal con rust-analyzer, ESLint | Neovim, Zed |
| **Figma** | Diseño de UI, mockups, branding | Penpot (open source) |
| **Node.js LTS** | `npm run dev` para frontend | pnpm, bun |
| **Rust (rustup)** | `cargo check`, `cargo test` | — |
| **Git** | Control de versiones | — |
| **Discord** | Comunicación diaria | Slack, Matrix |
| **Inkscape/GIMP** | Edición de assets SVG/PNG | — |
| **Obsidian** | Notas, brainstorming | Notion, Logseq |

---

## 11. Checklist de onboarding para tu amigo

- [ ] Clonar el repositorio
- [ ] Instalar Node.js LTS, Rust (rustup), Git
- [ ] Ejecutar `npm install` y verificar que no hay errores
- [ ] Ejecutar `cargo check` y verificar que compila
- [ ] Ejecutar `npx tsc --noEmit` y verificar que tipa
- [ ] Ejecutar `npm run dev` y ver el frontend en el navegador
- [ ] Leer `CLAUDE.md` para entender el proyecto
- [ ] Leer `docs/FuturosProyectos.md` sección 2.1 (GameTrim)
- [ ] Leer `docs/PLAN-FINAL/README.md` para el estado actual
- [ ] Unirse al Discord y presentarse
- [ ] Elegir la primera tarea del GitHub Project y empezar

---

## 12. Roadmap a largo plazo (ecosistema completo)

```
2026 Q2-Q3          2026 Q3-Q4          2027 Q1-Q2          2027 Q3+
─────────────────   ─────────────────   ─────────────────   ─────────────────
ClearTool 1.0       GameTrim 0.1        Mod Manager         AI Local Suite
├─ MVP completo     ├─ MVP gaming       ├─ Universal mods   ├─ Knowledge Hub
├─ Open-source MIT  ├─ Perfiles juego   ├─ 10 juegos        ├─ Subtitle Studio
├─ 500 users        ├─ 2.000 users      ├─ 50k users        ├─ Enterprise
└─ Feedback         └─ Pro license      └─ Partnerships     └─ $100k MRR

                    Sitio web SEO
                    ├─ 50 artículos
                    ├─ 150k visitas/mes
                    ├─ $3.5k/mes ads
                    └─ Lead magnet apps
```

---

## 13. Recordatorios finales

1. **El producto técnico bueno NO basta.** Distribución y marketing son el 60% del juego.
2. **Comunidad > marketing pagado.** Reddit, Discord, Twitter/X técnico, HN.
3. **Una marca, una audiencia, varios productos.** Es 10× más fácil vender el segundo producto a alguien que ya te conoce.
4. **Documentá todo en abierto** (devlog en blog, Twitter de progreso). El "build in public" genera tracción gratis.
5. **No optimices ingresos en el mes 1.** Optimizá retention y boca a boca los primeros 6 meses.
6. **Linux NO es un impedimento.** El frontend, branding, catálogos, docs y web son el 60% del trabajo y funcionan perfecto en Linux.
7. **Cada semana, demo.** Aunque sea de 5 minutos. Ver el producto avanzar motiva más que cualquier plan.

---

> **Próximo paso:** compartir este documento con tu amigo, configurar el repo en GitHub, y empezar por la Semana 1 del plan de acción.
