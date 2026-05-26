# Paso 06 — Web propia + README + LICENSE

**Área**: 12-distribution
**Tiempo estimado**: 6-8 horas
**Dependencias**: Paso 02 (release público), Paso 04 (endpoint updater)

## Qué hacemos

1. Web pública en `cleartool.app` con landing + docs + changelog + endpoint updater.
2. README profesional en el repo GitHub.
3. LICENSE + PRIVACY-POLICY + CONTRIBUTING.

## Stack web

- **Astro** (recomendado): SSG simple, MDX para artículos, súper rápido.
- Alternativa: Next.js, plain HTML, Hugo.

Hosting:
- **Cloudflare Pages** o **Vercel** (free tier, HTTPS auto, CDN global).

## Pasos

### 1. Dominio

Registrar `cleartool.app` (o variantes: `.io`, `.dev`, `.tool`). Coste ~$15/año.

Recomendación: usar Namecheap, Cloudflare Registrar (al coste mayorista), o Porkbun.

### 2. Setup repo web

```bash
# En la organización GitHub, repo nuevo: cleartool-web
mkdir cleartool-web && cd cleartool-web
npm create astro@latest
# Template: minimal con TypeScript
cd cleartool-web
npm i @astrojs/tailwind @astrojs/mdx
```

Estructura:
```
cleartool-web/
├── src/
│   ├── pages/
│   │   ├── index.astro              # landing
│   │   ├── download.astro           # selector de versión
│   │   ├── changelog.astro          # auto-gen from CHANGELOG.md
│   │   ├── privacy.astro            # privacy policy
│   │   └── docs/
│   │       ├── getting-started.md
│   │       ├── faq.md
│   │       └── troubleshooting.md
│   ├── layouts/
│   │   └── BaseLayout.astro
│   └── components/
│       ├── Header.astro
│       ├── Footer.astro
│       └── DownloadButton.astro
├── public/
│   ├── updates/
│   │   └── (handled by redirect)
│   └── screenshots/
└── astro.config.mjs
```

### 3. Landing (index.astro)

Estructura recomendada:

```
[ Hero ]
  ClearTool
  Limpieza, debloat y tweaks para Windows 11. Sin telemetría.
  [Descargar 1.0] [GitHub] [Docs]

[ Screenshots / hero image ]
  Mockup de la app en uso

[ Why ClearTool ]
  3 columnas:
  - Transparente: mostramos qué tocamos antes de tocarlo
  - Reversible: restore point + audit log
  - Local: cero telemetría

[ Modules grid ]
  Cards: Cache · Debloat · Services · Registry · Disk · Privacy

[ Quick install ]
  winget install ClearTool.ClearTool
  o descarga desde GitHub Releases

[ Testimonials / GitHub stars badge ]

[ Footer ]
  Privacy · License · GitHub · Issues
```

Mantener simple. Sin animaciones extra.

### 4. README.md del repo principal

Reemplazar el README actual de `cleartool` con uno serio:

```markdown
<div align="center">
  <img src="docs/assets/logo.png" width="128" alt="ClearTool" />
  <h1>ClearTool</h1>
  <p>Limpieza, debloat y tweaks para Windows 11. Open source, sin telemetría.</p>

  [![Release](https://img.shields.io/github/v/release/joowy/cleartool)](releases)
  [![License](https://img.shields.io/github/license/joowy/cleartool)](LICENSE)
  [![Stars](https://img.shields.io/github/stars/joowy/cleartool)](https://github.com/joowy/cleartool)
</div>

## Qué hace

ClearTool combina en una sola app:

- **Cache cleaner** con detección de procesos bloqueadores.
- **Debloat** de Microsoft consumer apps + OEM bloatware.
- **Registry tweaks** curados.
- **Service manager** con presets.
- **Restore points** automáticos antes de operaciones destructivas.
- **Audit log reversible**: revierte una operación concreta sin restaurar todo.
- **Disk analyzer** con treemap visual.
- **Privacy hardening** en 3 niveles.

## Screenshots

(insertar 4-5 screenshots: dashboard, debloat, cache, disk, privacy)

## Instalación

### winget (recomendado)

\`\`\`powershell
winget install ClearTool.ClearTool
\`\`\`

### Manual

Descarga el último `.exe` desde [Releases](https://github.com/joowy/cleartool/releases/latest) y ejecútalo.

### Modo portable

Descarga `ClearTool-portable-X.Y.Z.zip` desde Releases. Extrae y ejecuta `ClearTool.exe` sin instalar.

## Requisitos

- Windows 11 (build 22000 o superior).
- Permisos de administrador para operaciones destructivas.

## Privacidad

ClearTool **no envía nada a internet por defecto**. El único tráfico saliente es:
- Comprobar actualizaciones (configurable en Settings).
- Enlaces "Más info" al hacer click manualmente.

Ningún dato de uso, telemetría ni reportes de crash se envían sin tu consentimiento explícito.

Ver [PRIVACY-POLICY.md](PRIVACY-POLICY.md).

## FAQ

### ¿Es seguro?
Cada operación destructiva:
1. Crea un restore point del sistema antes.
2. Se registra en `%APPDATA%\ClearTool\audit.jsonl` con instrucciones de reversa.

Si algo va mal, puedes restaurar el sistema o revertir una operación concreta.

### ¿Por qué pide admin?
Modificar registros HKLM, servicios y eliminar paquetes Appx requiere elevation. La parte de **solo lectura** (escanear caché, listar bloatware) funciona sin admin (modo limitado).

### ¿Cómo lo desinstalo?
Apps & features → ClearTool → Uninstall. Te preguntará si quieres conservar settings.

## Contribuir

Ver [CONTRIBUTING.md](CONTRIBUTING.md). Especialmente bienvenidos:
- Entries de catálogo (debloat / cache / registry / privacy).
- Localizaciones.
- Reportes de bugs con repro claro.

## License

[MIT](LICENSE)

---

<sub>Construido con [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/) y [React](https://react.dev/).</sub>
```

### 5. LICENSE

Si no existe, crear con texto MIT:

```
MIT License

Copyright (c) 2026 Joel Sánchez Fernández

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### 6. PRIVACY-POLICY.md

```markdown
# Privacy Policy

**Última actualización: 2026-XX-XX**

ClearTool no recoge, almacena ni transmite información personal o telemetría.

## Lo que NO hacemos

- No enviamos datos de uso ni analytics.
- No recogemos información sobre tu sistema.
- No usamos servicios de tracking de terceros.
- No tenemos servidores que reciban datos tuyos.

## Lo que SÍ pasa a internet

1. **Comprobación de actualizaciones** (opt-out en Settings):
   - Petición HTTP GET a `https://cleartool.app/updates/...` para saber si hay versión nueva.
   - No incluye identificador único, IP queda en logs del CDN durante 24h (eliminada después).

2. **Enlaces "Más info"** que tú haces click:
   - Abren tu navegador con URLs de documentación externa.

## Datos locales

ClearTool guarda en tu PC, sin enviarlos a ningún sitio:

- `%APPDATA%\ClearTool\settings.json` — tus preferencias de UI.
- `%APPDATA%\ClearTool\audit.jsonl` — log de operaciones destructivas + reversa.
- `%APPDATA%\ClearTool\catalogs\` — catálogos custom que tú hayas añadido.

Todo esto se puede borrar manualmente o al desinstalar.

## Cambios futuros

Si en algún momento se añade telemetría opt-in, será **explícita y bien señalizada**, con toggle en Settings → Privacy.

## Contacto

Preguntas sobre privacidad: open an issue en GitHub o email `privacy@cleartool.app`.
```

### 7. CONTRIBUTING.md

```markdown
# Contribuir a ClearTool

Gracias por considerar contribuir. Lee esto antes.

## Cómo contribuir

### Reportar bugs

Abrir issue con:
- Versión de ClearTool (`Settings → About`).
- Versión de Windows (build number — `winver`).
- Pasos para reproducir.
- Logs si están disponibles (`Settings → Export diagnostic report`).

### Entries de catálogo (la mejor contribución)

ClearTool depende de catálogos curados de:
- Bloatware (`bloatware-catalog.json`)
- Cache locations (`cache-locations.json`)
- Registry tweaks (`registry-tweaks.json`)
- Privacy presets (`privacy-presets.json`)

Para añadir una entry:

1. Verifica en VM que tu entry funciona (detect + apply + revert).
2. Edita el JSON correspondiente en `.claude/skills/.../RESOURCES/`.
3. Asegura que `npm run validate-catalogs` pasa.
4. PR con un commit por entry, mensaje formato:
   `feat(debloat): añadir entry HP Wolf Security`.

### Código

- Conventional Commits: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`.
- Rust: `cargo fmt` + `cargo clippy -- -D warnings`.
- TypeScript: `npm run lint`.
- Tests para módulos destructivos (ver `docs/v2/13-testing-ci/`).

### Traducciones

Añadir locale en `src/locales/<lang>/`. Mantener mismo set de keys que `en/`.

## Setup dev

\`\`\`bash
git clone https://github.com/joowy/cleartool
cd cleartool
npm i
npm run tauri dev
\`\`\`

## Licencia

Al contribuir, aceptas que tu código se licencia bajo MIT (la misma del repo).
```

### 8. Astro web — config deploy

Cloudflare Pages:

1. Conecta el repo `cleartool-web` en dash.cloudflare.com → Pages.
2. Build command: `npm run build`.
3. Build output: `dist`.
4. Variables: ninguna por ahora.
5. Custom domain: `cleartool.app` (configurar DNS para CNAME).

Deploy automático en cada push a `main`.

### 9. Endpoint updater desde la web

En `cleartool-web`, añadir:

```ts
// src/pages/updates/[...path].astro
---
export async function getStaticPaths() {
  return [
    { params: { path: undefined } },
  ];
}

const tag = "v1.0.0"; // o leer dinámico desde GitHub Releases API
return Astro.redirect(
  `https://github.com/joowy/cleartool/releases/latest/download/latest.json`,
  302
);
---
```

(O usar Cloudflare Workers para algo más sofisticado si se llega.)

## Criterio de done

- [ ] Dominio `cleartool.app` apunta a Cloudflare/Vercel Pages.
- [ ] Landing con descarga visible.
- [ ] Docs página con getting-started + FAQ.
- [ ] Changelog page muestra histórico de releases.
- [ ] Privacy policy publicada.
- [ ] README del repo es serio (no el default).
- [ ] LICENSE + PRIVACY-POLICY + CONTRIBUTING en raíz.
- [ ] Endpoint `/updates/...` redirige a GitHub Releases.
- [ ] Mobile-responsive (no esencial pero deseable).
