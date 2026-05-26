# v2 · 00 — Visión para release público (v1.0)

## La pregunta que esta carpeta responde

> ¿Qué cambios hay que hacer en ClearTool entre el v0.1 actual y un v1.0 que un usuario desconocido pueda descargar de internet, instalar, y **fiarse**?

No es una pregunta de features. Es una pregunta de **confianza**, **calidad** y **diferenciación**.

## El problema que ClearTool intenta resolver

Windows 11 acumula:
- Bloatware preinstalado (consumer apps de Microsoft + apps OEM).
- Caché residual de actualizaciones, navegadores, apps UWP, telemetría.
- Servicios y tareas programadas que el usuario nunca pidió.
- Tweaks de registro que serían fáciles si la UI de Microsoft no los ocultara.

Las herramientas existentes que cubren parte de esto:

| Herramienta | Lo que hace bien | Lo que falla |
|-------------|------------------|--------------|
| **O&O ShutUp10** | Tweaks de privacidad masivos | Sólo registry, sin debloat, sin caché. UX vieja. |
| **Privatezilla** | Privacy tweaks open source | No toca cache, ni servicios, ni Appx provisionados. |
| **Bulk Crap Uninstaller** | Eliminar programas tradicionales | No conoce Appx ni servicios ni telemetría. |
| **Geek Uninstaller** | Limpio para uninstall | Sin debloat de provisionados ni cache cleaner. |
| **CCleaner** | Cache cleaner clásico | Caja negra, telemetría propia, owner sospechoso. UWP mal. |
| **WinDirStat** | Treemap de disco | Solo lectura. Sin acción. |
| **Disk Cleanup (built-in)** | Seguro y nativo | Cubre poco, sin debloat, sin UI moderna. |

**El espacio que ClearTool quiere ocupar**: una herramienta única que combine **cache cleaning + debloat + registry tweaks + service management + disk analysis** en una UI moderna, transparente, reversible, sin telemetría, open source.

## Quién es el usuario objetivo

Tres perfiles, en orden de prioridad:

### 1. Power user de Windows (P0 — primario)
- Conoce el registro, sabe lo que es un servicio, ha oído hablar de Appx.
- Le importa la privacidad. Probablemente desactivó Cortana y Copilot a mano alguna vez.
- Quiere algo que automatice lo que ya hace manualmente con PowerShell scripts dispersos.
- Le encanta el lema "muestra exactamente qué vas a tocar antes de tocarlo".

### 2. Técnico que repara PCs (P1 — secundario)
- Reinstala Windows 11 OEM, tiene que limpiar bloatware en 50 portátiles al mes.
- Necesita presets reutilizables, dry-run, batch operations.
- Restore points obligatorios para protegerse de quejas del cliente.

### 3. Gamer / streamer (P2 — terciario)
- Quiere recuperar disco y FPS.
- Le interesan: Game Bar, Xbox app, telemetría, hibernación, paging file.
- Probablemente no toca el registro pero sí ejecuta cleaners.

**Lo que NO somos**: una herramienta para abuelos. Si la UI necesita demasiados disclaimers para ser segura, ClearTool no es lo que quiere ese usuario; debería usar Disk Cleanup nativo.

## Lo que tiene que ser cierto para llamarse v1.0

Estas son las "non-negotiables" del release público. Si una falla, no se publica:

1. **El cache cleaner termina su trabajo.** Si un archivo está en uso, el usuario lo sabe y puede actuar (cerrar proceso, esperar reboot). Hoy hay errores `[object Object]` y un 30% de los archivos UWP no se borran ni se reprograman.
2. **Los errores son legibles.** Ningún `[object Object]`. Cada error dice qué pasó, en qué archivo/clave, y qué puede hacer el usuario.
3. **Los modules destructivos crean restore point** sin excepción (ya implementado).
4. **El audit log permite reversa fina.** El usuario puede deshacer una acción concreta sin restaurar todo el sistema.
5. **El binario está firmado** con certificado de organización válido. Sin esto, SmartScreen avisa y nadie lo instala.
6. **Existe un auto-updater** con releases firmadas. La gente no actualiza si tiene que ir a GitHub a descargar manualmente.
7. **Existe una página web** con propuesta clara, screenshots, y descarga directa.
8. **Existe un README serio** con installation, screenshots, FAQ, licencia.
9. **Los catálogos son cargables desde JSON externo** (además del embebido). Esto permite hotfixes sin recompilar.
10. **Hay un set mínimo de tests** que validan: catálogo carga, comandos IPC responden, restore point se crea+lista en una VM clean Win11.

## Lo que NO va a estar en v1.0 (y está bien)

Estas features son post-1.0. Documentarlas evita scope creep:

- Plugin/extensión system con APIs externas.
- Modo "wizard" guiado para usuarios noveles.
- Análisis predictivo ML de qué eliminar.
- Sincronización de preferencias entre PCs.
- Editor visual de tweaks (compilador de `.reg`).
- Driver cleanup.
- BCD/boot config editor.
- Diff de configuración entre dos PCs.

## Métricas de éxito post-launch

Cómo sabremos si v1.0 fue un éxito:

- **30 días post-launch**: 1000+ downloads desde GitHub Releases.
- **Reportes de bugs**: < 5 críticos / 100 instalaciones.
- **Rotura del sistema reportada**: 0 (objetivo absoluto; los restore points están para esto).
- **Issue de funcionalidad básica que falle ("no se borra X")**: < 10% de reportes.
- **Star count GitHub**: 200+ a los 60 días.

## Modelo de monetización (consciente: ninguno)

ClearTool v1.0 es **gratis** y **open source** (license MIT o Apache 2.0). No hay versión pro. No hay donations forzadas. La donation opcional aparece sólo en el About/Settings, nunca en el flujo normal.

**Por qué**: un cleaner que pide pasta antes de hacer su trabajo es indistinguible de adware. La confianza es nuestra moneda.

## Lo que sigue

Continúa por:
- `01-error-model-fix.md` — arreglar el `[object Object]` que sangra UX.
- `09-roadmap.md` — el calendario y orden de los milestones.
