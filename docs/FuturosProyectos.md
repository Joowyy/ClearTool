# Futuros Proyectos — Análisis estratégico para romper mercado

> Documento de brainstorming serio. Ideas filtradas por **potencial real de ingresos**, **mercado existente pero mal servido**, y **tecnologías que valen la pena aprender** desde el punto de partida actual (Tauri + Rust + React + ClearTool en marcha).

---

## 1. Framework para decidir qué proyecto vale la pena

Antes de elegir, todo proyecto se evalúa contra estas seis preguntas. Si no pasa al menos 4, no merece tu tiempo.

| Filtro | Pregunta dura |
|---|---|
| **Mercado real** | ¿Hay gente buscando esto ahora mismo en Google con intención clara? Volumen mensual > 10k búsquedas en alguna query relevante. |
| **Competencia floja** | ¿Los productos existentes son feos, lentos, abandonados, o caros sin justificación? Si Google y Microsoft ya lo hacen bien, huye. |
| **Monetización en >1 vía** | Idealmente ads + licencia + afiliados, no depender de un solo canal. |
| **Distribución posible solo** | ¿Podés llegar a usuarios sin equipo de ventas? SEO, comunidades (Reddit, Discord), boca a boca técnico. |
| **Defensibilidad** | ¿Por qué no te copia un fork en 2 semanas? Catálogos curados, marca, datos acumulados, integraciones. |
| **Compounding** | ¿Cada hora que invertís acumula valor (SEO, catálogo, base de usuarios) o se evapora? |

ClearTool ya pasa varios filtros (mercado real de utilities Windows, competencia floja salvo CCleaner viejo, defensibilidad por catálogos curados). Los siguientes proyectos están elegidos con esos mismos criterios.

---

## 2. Proyectos con potencial de romper mercado

### 2.1 — GameTrim (Tauri, adyacente a ClearTool)

**Idea:** Optimizador de Windows **específico para gaming**. Modo de juego que cierra procesos en background, libera RAM agresivamente, ajusta plan de energía, limpia shader caches (DirectX/Vulkan/NVIDIA), libera VRAM, deshabilita Game Bar/Xbox overlay, optimiza programación de CPU/GPU. Detección automática del juego que se lanza y aplicación del perfil.

**Mercado:** ~1.500 millones de gamers PC en el mundo. El nicho "PC optimization for gaming" tiene volumen brutal de búsqueda (Reddit r/pcgaming, r/buildapc, miles de tutoriales sueltos en YouTube).

**Competencia actual:**
- **Razer Cortex** — feo, intrusivo, bundle con software no deseado, opt-out de telemetría confuso.
- **Process Lasso** — potente pero UI de 2005, curva alta.
- **MSI Afterburner** — solo overclock/monitoring, no limpia nada.
- **Wagnardsoft DDU** — sólo drivers, nicho.

Hay espacio enorme para algo bonito, transparente y serio.

**Por qué te conviene:** reutilizás ~70% del backend de ClearTool (manejo de servicios, procesos, registro). Cambia el catálogo y el packaging. Es ClearTool con identidad de marca distinta y vertical concreto.

**Monetización:**
- Free + Pro ($14.99 one-time). Pro: perfiles por juego, auto-detección, benchmarks integrados.
- Afiliados de hardware (Amazon: RAM, SSDs NVMe, GPUs).
- Patrocinios de marcas gaming (NZXT, Corsair) cuando tengas 50k+ users.

**Tecnologías a aprender extra:** WMI avanzado (process injection, GPU monitoring), interfaz de NVIDIA NVAPI / AMD ADL para leer estado de GPU.

---

### 2.2 — Ecosistema SEO: cleartool.app + windowsoptimizer.guide

**Idea:** Sitio web complementario a ClearTool y a futuros productos. **No es la app, es el imán de tráfico**. Contenido:
- Tutoriales profundos: "Cómo desinstalar Edge en Windows 11 sin romper el sistema", "Lista de servicios que podés deshabilitar en Windows 11 (con explicación de cada uno)".
- Calculadoras interactivas: "¿Cuánto espacio ocupa la caché de Windows Update?", "Tu PC vs los requisitos de [juego]".
- Comparativas: "ClearTool vs CCleaner vs O&O ShutUp10".
- Wiki de servicios de Windows (cada servicio una página, lo que hace, si se puede deshabilitar, riesgos).

**Mercado:** búsquedas tipo "how to debloat windows 11", "windows services list" → cientos de miles al mes globalmente. Páginas como tenforums.com, elevenforum.com, howtogeek.com viven de este tráfico.

**Competencia:** la mayoría son foros viejos o blogs SEO genéricos con info desactualizada o incorrecta. Una marca **con app propia** que respalde el contenido tiene credibilidad técnica que esos no tienen.

**Monetización:**
- **Google AdSense / Ezoic / Mediavine** (este último paga 5–10× más que AdSense, requiere 50k visitas/mes).
- **Afiliados Amazon** (SSDs, RAM, antivirus, mantenimiento PC).
- **Lead magnet para ClearTool / GameTrim** (descargas, emails).
- **Patrocinios de software complementario** (VPNs, antivirus).

Con 200k visitas/mes orgánicas, un sitio así genera $2.000–6.000/mes de ads + afiliados, **pasivamente**, mientras el catálogo de artículos no envejezca.

**Tecnologías:** **Astro** o **Next.js** (SSG/SSR), **Tailwind**, MDX para artículos con componentes interactivos embebidos. Hosting en **Cloudflare Pages** o **Vercel** (free tier infinito para SSG). Analytics: **Plausible** o **Umami** (privacy-friendly, mejor reputación que GA4).

**Tiempo a primer dinero:** 4–8 meses de contenido sostenido antes de ver ingresos serios. Es el proyecto más lento pero el más **compounding**: cada artículo que escribís en 2026 te paga en 2028.

---

### 2.3 — Local AI Knowledge Hub (Tauri + IA local)

**Idea:** App de escritorio que **indexa todos tus archivos** (PDFs, docs, notas, código, emails exportados) con **embeddings 100% locales** y permite búsqueda semántica + chat conversacional sobre tus propios documentos. **Sin enviar nada al cloud.**

**Por qué ahora:** la ola de IA está en su pico, pero el 99% de las herramientas son cloud (privacidad mala) o requieren config técnica brutal (Ollama + LangChain + scripts). Hay un hueco gigante para algo de **un click**.

**Mercado:** abogados, médicos, investigadores, estudiantes de máster/doctorado, gente con compliance issues, gobierno, defensa. **Pagan** por privacy-first.

**Competencia:**
- **Notion AI / Mem AI** — cloud, no privacy.
- **Obsidian + plugins** — requiere armarlo a mano.
- **ChatPDF, AskYourPDF** — cloud, un solo PDF a la vez.
- **GPT4All, LM Studio** — chat con LLM local pero **sin indexación de archivos**.
- **AnythingLLM** — lo más cercano, pero UI cruda y abandonado a ratos.

**Monetización:**
- Free hasta 1.000 documentos. Pro $19/mes o $149/año: ilimitado, OCR, integración con cloud drives.
- Licencia empresarial (privacy compliance value): $500–2.000/seat/año.
- Marketplace de "knowledge packs" curados (legal, médico, ingeniería).

**Tecnologías a aprender:**
- **Embeddings locales**: `sentence-transformers` o `BGE-M3` corridos vía `candle-rs` (Rust nativo) o `llama.cpp` con bindings Rust.
- **Vector DB embebida**: `LanceDB` (Rust nativo, perfecto para Tauri) o `Qdrant` local.
- **LLM local**: `llama.cpp` con modelos quantizados (Llama 3.3 8B, Mistral, Phi-4 mini) que corren en CPU/GPU consumer.
- **Parsing**: `pdf-extract`, `mupdf`, `docx-rs`, OCR con `tesseract-rs`.
- **Frontend chat**: streaming UI con React + `react-markdown` + componentes de citas a fuente.

Este proyecto es **técnicamente el más ambicioso** de la lista pero el de mayor techo de mercado. Si funciona, es producto serio que puede llegar a $20k–100k MRR en 2–3 años.

---

### 2.4 — Idle/Incremental Web Game (modelo ads puro)

**Idea:** Juego idle/incremental para web. Mecánica simple, loop adictivo, monetización **100% ads** + microtransacciones opcionales. Ejemplos del género: Cookie Clicker, A Dark Room, Universal Paperclips, Melvor Idle.

**Por qué es interesante:**
- **Producción barata**: un dev solo puede sacar uno en 2–4 meses.
- **Retención brutal**: los idle games tienen DAU/MAU > 30% cuando el loop está bien.
- **Ads pagan bien**: banner + reward video (un usuario activo vale $0.10–0.50/mes en ads).
- **Si pega un mínimo (10k DAU), son $1.000–5.000/mes pasivos**.

**Diferenciador necesario:** los idle genéricos están saturados. Necesitás un **tema único y memético**: idle ambientado en algo cultural específico, narrativa progresiva, mecánica nueva (combinatoria, social, multijugador asíncrono). El tema lo es todo en este género.

**Competencia actual:** Kongregate, Armor Games, miles en itch.io. Hay que destacar por arte/tema/gameplay loop.

**Monetización:**
- **AdSense + Google AdManager** (banner).
- **Reward video** (Rewarded Ads via Unity Ads / AdMob web).
- **Crypto cosmetics opcional** (skin packs $1–5).
- **Versión sin ads** ($4.99 one-time vía Stripe).

**Tecnologías:**
- **Phaser 3** (framework JS para 2D, muy maduro) o **Pixi.js** (más bajo nivel, más rendimiento).
- **Svelte / React** para UI alrededor del canvas.
- **Backend:** opcionalmente un Cloudflare Workers + D1 para leaderboards y saves (gratis hasta volumen serio).
- **Versión móvil**: el mismo juego empaquetado con **Capacitor** o **Tauri Mobile** para iOS/Android, donde los ads de móvil pagan mejor.

Este es el camino **menos predecible** (puede pegar fuerte o cero) pero también el de mayor upside relativo si pega: un juego viral genera más en un mes que ClearTool en un año.

---

### 2.5 — Mod Manager Universal para juegos

**Idea:** Gestor de mods cross-game que funcione bien. **Vortex** de Nexus es lento, **Mod Organizer 2** es solo para Bethesda, **r2modman** solo para Risk of Rain, **BG3 Mod Manager** solo para BG3. No hay nada universal moderno.

**Mercado:** decenas de millones de modders activos. Skyrim sigue moviendo millones de descargas mensuales **10 años después de salir**. BG3, Cyberpunk 2077, Stardew, Minecraft, Sims 4 — comunidades enormes.

**Por qué ahora:** Nexus Mods (el dueño de Vortex) tiene su sistema atado a su web. Hay espacio para un manager **agnóstico**, rápido, con perfiles, dependencias resueltas, snapshots y rollback.

**Competencia:**
- **Vortex** — lento, UX confusa, atado a Nexus.
- **MO2** — bueno pero solo Bethesda, mantenimiento voluntario.
- **r2modman** — bonito pero hardcoded a juegos específicos.

**Monetización:**
- Free + Pro $9.99/año (perfiles cloud sync, integración Discord, prioridad de descarga si hostean mirror).
- Patreon (comunidad modder paga bien por tooling que funciona).
- Partnerships con sitios de hosting (Nexus, ModDB, Thunderstore) por integración.

**Defensibilidad:** una vez que tenés 100k usuarios y soportás 50 juegos curados, copiarte cuesta un año de trabajo.

**Tecnologías:** Tauri + Rust ideal — manejo de FS, hashing, dependency resolution, parsers de mod metadata varían por juego. Frontend React. Posible integración con APIs de Nexus, CurseForge, Thunderstore.

---

### 2.6 — Subtitle Studio AI (creator economy + IA local)

**Idea:** App Tauri que toma un video, transcribe con **Whisper local**, traduce a N idiomas con un LLM local, **edita los subtítulos con timeline visual**, exporta SRT/VTT/ASS, hardcodea en el video con ffmpeg.

**Mercado:** YouTubers, TikTokers, podcasters, creadores de cursos online, traductores freelance. Las herramientas online (Rev, Descript, Kapwing) cobran $15–30/mes y mandan tu contenido al cloud.

**Competencia:**
- **Descript** — caro, cloud, lento.
- **Subtitle Edit** — gratis, feo, sin IA.
- **Whisper standalone** — requiere CLI, no para creadores normales.

**Monetización:**
- Free hasta 30 min/mes de transcripción. Pro $9.99/mes ilimitado.
- Pack idiomas premium (60 idiomas vs 12 free).
- Templates de estilo de subtítulos (TikTok-ready, YouTube-ready).

**Tecnologías:** `whisper.cpp` con bindings Rust, `ffmpeg-next` para video, LLM local pequeño (Phi-4 mini, Llama 3.2 3B) para traducción. Frontend: React con timeline editor (puede ser canvas o Konva.js).

---

### 2.7 — DevDash / WSL Manager Pro (nicho dev, pagan caro)

**Idea:** Panel local para developers. Combina:
- Vista unificada de **Docker containers** corriendo (sin abrir Docker Desktop, que es pesado).
- **Procesos** + puertos abiertos + qué los abrió.
- **Estado de WSL** (distros, recursos asignados, snapshots).
- **Git status** agregado de todos los repos en tu máquina.
- **Logs centralizados** de servicios locales.
- **Cleanup**: caches de npm/yarn/pnpm/cargo/pip/maven/go.

**Mercado:** ~30M de devs en el mundo. Los que pagan tools (JetBrains, Tower, GitKraken) pagan fácil $50–150/año por algo que les ahorre tiempo.

**Competencia:**
- **Docker Desktop** — pesado, lento, ahora requiere licencia empresarial.
- **Lazydocker** — TUI, solo Docker.
- **GitKraken** — solo git, $50/año.
- Nada que combine todo en un dashboard local moderno.

**Monetización:**
- Free + Pro $39/año o $99 lifetime.
- Licencia equipo: $20/dev/año.

**Tecnologías:** Tauri (otra vez), Rust (procesos vía `sysinfo`, Docker vía `bollard`, WSL vía wsl.exe wrappers), React. Posibilidad de plugin system para que la comunidad añada widgets.

---

## 3. Combos sinérgicos (lo que multiplica los ingresos)

Los proyectos no se eligen aislados. Los combos correctos hacen que **el segundo proyecto cueste la mitad** y el tercero sea casi gratis.

### Combo "Ecosistema Windows Utilities"
**ClearTool + GameTrim + cleartool.app/windowsoptimizer.guide**

- Compartís backend (services, registry, restore points, audit log).
- El sitio web manda tráfico a las dos apps.
- Las apps mencionan el sitio en su menú "Learn more".
- Newsletter común con ~30k subscribers en 18 meses = lista vendible para sponsors.
- Una sola identidad de marca, un solo cert de firma, una sola infra de updater.

**Ingresos potenciales a 24 meses (escenario realista, no optimista):**
- ClearTool Pro: 2.000 licencias × $14.99 = $30.000.
- GameTrim Pro: 5.000 licencias × $14.99 = $75.000.
- Sitio web (150k visitas/mes con Mediavine): ~$3.500/mes × 12 = $42.000/año.
- Afiliados Amazon hardware: $500–1.500/mes.

Total razonable año 2 si ejecutás bien: **$120k–180k**.

### Combo "AI Local Suite"
**Local AI Knowledge Hub + Subtitle Studio AI**

- Compartís pipeline de modelos (Whisper, LLM local, embeddings).
- Compartís infra de bundling de modelos (que es lo complicado).
- Marca "AI sin cloud" defendible y con narrativa fuerte.
- Las dos apps se cross-promocionan dentro de la suite.

Mercado más caro de construir, techo más alto.

### Combo "Gaming Vertical"
**GameTrim + Mod Manager Universal + Idle game**

- Audiencia compartida (gamers PC).
- Las apps mencionan el juego, el juego linkea las apps.
- Discord común, comunidad común.
- Si una pega, jala a las otras.

---

## 4. Camino recomendado (orden de batalla)

No empezar todo a la vez. Secuencia con justificación:

### Fase 0 — Ahora (3–6 meses)
**Terminar ClearTool a calidad profesional + lanzarlo bien.**
Sin esto no hay nada. ClearTool es tu **portfolio + primera fuente de ingresos + manual de instrucciones de Tauri vivido en producción**.

### Fase 1 — Mes 4–6 (paralelo al final de ClearTool)
**Arrancar cleartool.app / windowsoptimizer.guide.**
SEO tarda. Cuanto antes empieces a publicar contenido, antes empieza a compoundear. Una hora al día escribiendo artículos en paralelo a coding.

### Fase 2 — Mes 6–10
**GameTrim**. Reutilizás 70% del código de ClearTool, sacás un producto nuevo en 3–4 meses con vertical y marca distinta. **Multiplica ingresos sin multiplicar trabajo.**

### Fase 3 — Mes 10–14
**Elegí UNA de estas tres según tu energía y datos del mercado en ese momento:**

- **Si ClearTool/GameTrim pegan fuerte** → Mod Manager Universal (aprovechás reputación en utilities + entrás en gaming serio).
- **Si querés un golpe asimétrico (alto riesgo, alto upside)** → Idle game.
- **Si querés algo con techo gigante de ingresos** → Local AI Knowledge Hub (es el más complejo y el más caro de construir, pero el único que puede llegar a $100k MRR).

### Fase 4 — Año 2+
Doblá la apuesta en el ganador. No empieces el cuarto proyecto hasta que los tres anteriores estén autopagándose o claramente muertos.

---

## 5. Tecnologías a estudiar (por camino elegido)

### Camino "Utilities + Web SEO"
1. Tauri 2.x avanzado (capabilities, plugins propios, updater).
2. Windows APIs avanzadas (WMI, COM, WinRT).
3. Astro o Next.js (SSG).
4. Tailwind + shadcn/ui (mismo stack que ClearTool ya usa).
5. SEO técnico: schema.org, Core Web Vitals, search intent research.
6. Copywriting técnico: cómo escribir un artículo que rankee Y convierta.

### Camino "AI Local"
1. **Candle** (framework ML en Rust puro de HuggingFace) o **llama.cpp** con bindings Rust.
2. **LanceDB** o **Qdrant** embebido — bases vectoriales.
3. Embeddings: `BGE-M3`, `nomic-embed-text`, comprensión de chunking strategies.
4. Quantization de modelos (GGUF, AWQ) — cómo elegir el modelo correcto para el hardware del usuario.
5. RAG patterns avanzados (hybrid search, reranking, query expansion).
6. Whisper.cpp + WhisperX para timestamps precisos.
7. ffmpeg avanzado (filters, hardware accel).

### Camino "Gaming + Web Games"
1. **Phaser 3** o **Pixi.js** + TypeScript.
2. **Cloudflare Workers + D1** para backend serverless gratis hasta volumen serio.
3. **Capacitor** o **Tauri Mobile** para llevar el mismo juego a iOS/Android.
4. Diseño de monetización en juegos free-to-play (Ahrefs blog, GameAnalytics).
5. Engagement / retention loops (Hooked, de Nir Eyal).
6. Si te ponés serio: **Godot 4** (gratis, potente, GDScript fácil) o **Bevy** (engine Rust, ECS, en alpha pero crece rápido).

### Camino "Dev Tools"
1. **Docker API** (`bollard` crate en Rust).
2. **Tauri** + dashboards con `recharts` o `visx`.
3. Plugin systems en Rust (`libloading`, `wasm-bindgen` para sandboxed plugins).
4. UX para developers (estudiar Linear, Raycast, Warp).

---

## 6. Lo que NO recomendaría

- **App SaaS B2B genérica** (CRM otro más, project management otro más). Mercado saturado, marketing brutal, ciclos de venta largos.
- **Cripto / Web3 wallet / DeFi tool.** Mercado tóxico, regulación impredecible, comunidad volátil. Hay gente ganando dinero ahí pero el coste reputacional y mental es alto.
- **NFT marketplace.** Muerto comercialmente desde 2023.
- **App social nueva.** Sin red existente, churn brutal.
- **Otro launcher de Windows.** Capa de visibilidad bajísima, mercado micro.
- **Editor de texto / IDE nuevo.** Mercado dominado por VS Code/JetBrains/Zed; no hay grieta defendible.

---

## 7. Recordatorios finales

1. **El producto técnico bueno NO basta.** Distribución y marketing son el 60% del juego. Reservá tiempo para esto desde el día 1.
2. **Comunidad > marketing pagado.** Reddit, Discord, Twitter/X técnico, HN. Empezá a estar presente con cara antes de tener producto que vender.
3. **Una marca, una audiencia, varios productos.** Es 10× más fácil vender el segundo producto a alguien que ya te conoce que conseguir un cliente nuevo.
4. **Documentá todo en abierto** (devlog en blog, Twitter de progreso). El "build in public" genera tracción gratis y enseña marketing por hacer.
5. **No optimices ingresos en el mes 1.** Optimizá **retention y boca a boca** en los primeros 6 meses. Si la gente vuelve y te recomienda, el dinero llega solo.

---

> Próximo paso sugerido: cuando termines ClearTool, releer este doc, reevaluar el mercado en ese momento (los huecos cambian rápido en 2026–2027), y elegir el siguiente según los datos reales de cómo respondió el público a ClearTool.
