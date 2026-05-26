# v2 · 07 — Distribución y Release (P1)

## Por qué este spec es P1

Sin un plan de distribución decente, ClearTool v1.0 se publica como un `.exe` en GitHub Releases que SmartScreen bloquea, Defender marca como "PUP", nadie firma, nadie actualiza, y el proyecto muere a las 3 semanas.

Este spec cubre:
- Code signing
- Installer (NSIS + MSIX para Store)
- Auto-updater
- Canales (stable / beta)
- Distribución multi-canal (GitHub, Microsoft Store, winget, web propia)
- Portable mode

## Code Signing

### Sin firma = SmartScreen bloquea
Windows 11 SmartScreen bloquea por defecto cualquier `.exe` descargado de internet sin firma de organización conocida. El usuario tiene que ir a "Más información → Ejecutar de todas formas". El 70% lo cierra ahí.

### Opciones de cert

| Cert tipo | Precio aprox. | SmartScreen | Notas |
|-----------|---------------|-------------|-------|
| Self-signed | $0 | ❌ bloqueado siempre | Sólo testing local |
| Standard Code Signing (OV) | ~$200/año | 🟡 reputation building (10-30 días) | Acumula reputación con downloads |
| EV Code Signing | ~$300/año | ✅ instant trust | Requiere HSM físico/cloud (YubiKey o Azure Key Vault) |
| **Azure Trusted Signing** | ~$10/mes | ✅ instant trust | Subscription Microsoft, fácil, **recomendado** |

**Decisión**: empezar con **Azure Trusted Signing** ($10/mes, sin certificado físico, integrable en GitHub Actions con un secret).

Setup:
1. Crear "Trusted Signing Account" en Azure.
2. Crear "Identity Validation" — Microsoft verifica que la entidad es real (1-3 días).
3. Crear "Certificate Profile" público.
4. En GitHub Actions, usar `azure/trusted-signing-action@v0` con secrets.

Workflow CI:
```yaml
- name: Sign with Azure Trusted Signing
  uses: azure/trusted-signing-action@v0
  with:
    azure-tenant-id: ${{ secrets.AZURE_TENANT_ID }}
    azure-client-id: ${{ secrets.AZURE_CLIENT_ID }}
    azure-client-secret: ${{ secrets.AZURE_CLIENT_SECRET }}
    endpoint: https://eus.codesigning.azure.net/
    trusted-signing-account-name: cleartool
    certificate-profile-name: cleartool-public
    files-folder: src-tauri/target/release/bundle/
    files-folder-filter: exe,msi
```

### Firmar qué
- `ClearTool.exe` (binario principal)
- `ClearTool-Setup-X.Y.Z.exe` (instalador NSIS)
- `ClearTool-X.Y.Z.msi` (instalador MSI alternativo, si se distribuye en winget)

## Installer NSIS

Tauri ya genera NSIS installer. Customizar:

**`src-tauri/tauri.conf.json`:**
```json
"bundle": {
  "windows": {
    "nsis": {
      "installerIcon": "icons/icon.ico",
      "installMode": "perMachine",
      "headerImage": "installer/header.bmp",
      "sidebarImage": "installer/sidebar.bmp",
      "license": "../LICENSE",
      "languages": ["English", "Spanish"],
      "displayLanguageSelector": true,
      "perMachine": true,
      "allowDowngrades": false,
      "createDesktopShortcut": true,
      "createStartMenuShortcut": true
    }
  }
}
```

### Pre/post install hooks
NSIS permite scripts custom. Para v1.0:
- **Pre-install**: comprobar Windows version (≥ 22000), abortar con mensaje claro si no.
- **Post-install**: ofrecer "Crear restore point ahora" como checkbox antes de cerrar.

### Uninstaller
Tauri lo genera. Asegurarse de que:
- Elimina el binario y catálogos embebidos.
- NO elimina `%APPDATA%\ClearTool\` (logs + settings) sin confirmación.
- Pregunta: "¿Eliminar también configuración y audit log? (recomendado: No)".

## MSIX para Microsoft Store

### Por qué Store
- Visibilidad enorme para apps Windows.
- Auto-update gestionado por Microsoft.
- Instalación sin SmartScreen (Store apps están whitelisted).
- Sandboxing automático (limita lo que la app puede hacer — esto es un PROBLEMA para una app que toca registro y servicios).

### El problema: AppContainer sandbox
Apps MSIX corren en AppContainer. **No pueden** por defecto:
- Modificar HKLM.
- Detener/arrancar servicios.
- Eliminar archivos fuera de su sandbox.

Esto rompe el 80% de ClearTool.

### Soluciones

**A. Full Trust App** (sí, MSIX permite esto):
- Declarar `<rescap:Capability Name="runFullTrust" />` en AppxManifest.xml.
- Requiere submission especial al Store con justificación.
- Microsoft revisa y aprueba (o no). Histórico: aprueba apps utility legitimas (e.g., O&O ShutUp10 podría intentarlo).

**B. Distribuir vía MSIX externo (sin Store)**:
- Compilar MSIX firmado y servirlo desde nuestra web.
- Usuario hace doble click → Windows 11 instala vía App Installer.
- Sin Store review pero sin visibilidad Store.

**C. Side load only**:
- Distribuir MSIX como alternativa al .exe pero sin Store.
- Igual que (B) pero documentado como "instalación alternativa".

**Decisión v1.0**: Empezar con NSIS+MSI únicos (más control). Considerar Store en v1.1 cuando la app esté estable y la submission tenga sentido.

## winget

Microsoft maintain un repositorio público de paquetes: `winget-pkgs` en GitHub. Una vez que tengamos un MSI/EXE firmado y release público, submit un PR al repo:

```yaml
# manifests/c/ClearTool/ClearTool/1.0.0/ClearTool.installer.yaml
PackageIdentifier: ClearTool.ClearTool
PackageVersion: 1.0.0
InstallerType: nullsoft
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/joelsanchez/cleartool/releases/download/v1.0.0/ClearTool-Setup-1.0.0.exe
    InstallerSha256: <sha256>
    Scope: machine
```

Tras merge, el usuario instala con:
```bash
winget install ClearTool.ClearTool
```

Coste: gratis, sólo curaduría del PR.

## Auto-updater

Tauri tiene `tauri-plugin-updater`. Requiere:
1. Endpoint que sirve `latest.json` con info de la última versión.
2. Releases firmadas con `tauri signer` (clave privada nuestra).
3. App configurada con la public key correspondiente.

### Setup

**`src-tauri/tauri.conf.json`:**
```json
"plugins": {
  "updater": {
    "active": true,
    "endpoints": [
      "https://cleartool.app/updates/{{target}}/{{current_version}}",
      "https://github.com/joelsanchez/cleartool/releases/latest/download/latest.json"
    ],
    "dialog": true,
    "pubkey": "<public-key>"
  }
}
```

Endpoint principal: web propia (control sobre canal). Fallback: GitHub Releases (siempre disponible).

### Canales

| Canal | Audiencia | Frecuencia | Estabilidad |
|-------|-----------|-----------|-------------|
| `stable` | Default. Usuarios finales. | Cada 4-8 semanas | Alta. Tests completos. |
| `beta` | Early adopters. Opt-in. | Cada 1-2 semanas | Media. Tests automatizados pasan. |
| `nightly` | Devs. Opt-in via flag. | Cada commit a main | Baja. Sólo lint+build. |

Settings → Avanzado → "Update channel: ◉ stable ○ beta ○ nightly".

Cada canal sirve su propio `latest.json`:
```
https://cleartool.app/updates/stable/...
https://cleartool.app/updates/beta/...
https://cleartool.app/updates/nightly/...
```

### Cadencia de checks
- Al arrancar la app (si `behavior.checkUpdatesOnStart === true`, default true).
- Botón manual "Check for updates" en Settings.
- Background check cada 24h con la app abierta (silenciado, sólo dialog cuando hay update).

### UX update
```
┌────────────────────────────────────────────┐
│ Actualización disponible: v1.0.3           │
│                                            │
│ Novedades:                                 │
│ • Corregido bug Spotify lock               │
│ • Mejor detección de OEM bloatware         │
│ • 12 entradas nuevas en catálogo           │
│                                            │
│ [Notas completas] [Recordar después]       │
│ [Descargar e instalar]                     │
└────────────────────────────────────────────┘
```

Instalación: descarga MSI/EXE firmado → ejecuta → app se cierra → installer corre → app se relanza.

## Portable mode

Algunos usuarios prefieren `.exe` standalone sin instalar. Tauri build estándar ya genera `ClearTool.exe` sin installer. Documentar:

> **Modo portable**: descarga `ClearTool-portable-1.0.0.zip` desde Releases. Extrae y ejecuta `ClearTool.exe`. Los settings se guardan en `%APPDATA%\ClearTool\` igual que la versión instalada.

Limitaciones del modo portable:
- Sin auto-update (no hay forma de actualizar un .exe que se ejecuta a sí mismo).
- Sin shortcut en Inicio.
- UAC prompt cada arranque (igual que con instalador `perUser`, no `perMachine`).

## GitHub Releases

CI workflow `release.yml`:
```yaml
on:
  push:
    tags: ['v*.*.*']

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: npm ci
      - run: npm run tauri build -- --target x86_64-pc-windows-msvc

      - uses: azure/trusted-signing-action@v0
        with: { ... }

      - uses: tauri-apps/tauri-action@v0
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'ClearTool v__VERSION__'
          releaseBody: |
            **Changelog**: ver CHANGELOG.md

            **Descargas:**
            - `ClearTool-Setup-*.exe` — instalador recomendado
            - `ClearTool-portable-*.zip` — modo portable
          releaseDraft: true
          prerelease: false
```

Tags semver: `v1.0.0`, `v1.0.1-beta.1`, `v1.1.0-rc.1`.

## Web propia

Dominio: `cleartool.app` o similar (~$15/año).

Stack mínimo:
- Sitio estático Next.js / Astro / pura HTML+CSS.
- Hosting: Vercel/Netlify/Cloudflare Pages free tier.
- Páginas:
  - `/` — landing con propuesta de valor, screenshots, descarga.
  - `/changelog` — auto-generado desde CHANGELOG.md.
  - `/docs` — getting started, FAQ, troubleshooting.
  - `/download` — selector de versión + arquitectura.
  - `/updates/{target}/{version}` — endpoint del auto-updater (devuelve JSON).

CDN para descargas: GitHub Releases es suficiente (alta velocidad, gratis).

## Distribuciones secundarias

| Plataforma | Estado v1.0 | Esfuerzo |
|------------|-------------|----------|
| GitHub Releases | ✅ Día 1 | Bajo |
| winget | ✅ Día 1 (post-launch) | Bajo (un PR) |
| Chocolatey | 🟡 Día 30 | Medio (mantiene paquete) |
| Scoop | 🟡 Día 30 | Bajo |
| Microsoft Store | ⏭ v1.1 o v1.2 | Alto (review process) |

## Métricas pre-launch

Antes de marcar v1.0:
- [ ] Cert válido (Trusted Signing activo).
- [ ] CI firma automáticamente.
- [ ] Installer testeado en 3 VMs limpias (22H2, 23H2, 24H2).
- [ ] Auto-updater probado: v1.0.0 → v1.0.1 simulado.
- [ ] Web up con landing + docs básicos.
- [ ] Changelog completo.
- [ ] README serio en GitHub.
- [ ] License (MIT o Apache 2.0).
- [ ] Privacy policy (corta: "no telemetría, no networking").

## Coste operativo anual

| Item | Coste |
|------|-------|
| Azure Trusted Signing | $120/año |
| Dominio cleartool.app | $15/año |
| Hosting web | $0 (free tier) |
| GitHub Releases | $0 (free) |
| Code signing review | $0 |
| **Total** | **~$135/año** |

Cubrible con donations opcionales o de bolsillo.

## Lo que NO está en v1.0

- macOS / Linux builds (Tauri lo soporta pero ClearTool es Win-only).
- Enterprise MSI con políticas de grupo.
- Versión "Pro" con features extra.
- Telemetría de uso (intencional: cero networking por default).

## Criterio de "done"

- [ ] Azure Trusted Signing activo y firmando en CI.
- [ ] NSIS instalador customizado y testeado en 3 builds Windows.
- [ ] Auto-updater funcional con canal stable.
- [ ] GitHub Releases pipeline configurado con tag trigger.
- [ ] winget PR submitted y aprobado.
- [ ] Web mínima (`cleartool.app` o equivalente) operativa.
- [ ] Privacy Policy y License documentados.
- [ ] CHANGELOG.md completo desde v0.1 → v1.0.
