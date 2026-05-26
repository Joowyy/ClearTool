# 10 — Distribución (instalador, manifest, iconos, metadatos)

> **Posición:** 10/14.
> **Dependencias:** módulos 00-07 cerrados + [08-SEGURIDAD-FINAL](08-SEGURIDAD-FINAL.md) en verde + [09-TESTS](09-TESTS.md) en verde.
> **Output:** `.exe` NSIS y `.msi` funcionales, manifest final, iconos en alta resolución, metadata coherente, estrategia SmartScreen documentada.

---

## 1. Resumen ejecutivo

ClearTool v1.0 se distribuye:

1. **NSIS `.exe` installer** — el formato preferido. Mejor UX, instala per-user.
2. **MSI** — para entornos corporativos / deployment vía GPO.
3. **Portable `.exe`** (opcional, post v1.0) — un solo ejecutable, sin instalación.

**Sin firma de código en v1.0.** SmartScreen mostrará warning. Estrategia: documentación clara + checksum verificable + reputación se construye con el tiempo. Plan documentado para certificado OV cuando haya tracción.

---

## 2. Diagnóstico del estado actual

| Item | Estado |
|---|---|
| `src-tauri/tauri.conf.json` configurado para release | 🟡 Existe pero metadatos pendientes |
| Iconos en `src-tauri/icons/` | 🟡 Genéricos del scaffold |
| Manifest `requireAdministrator` en release | ✅ |
| `productName`, `version`, `identifier` | 🟡 Defaults |
| Bundle config (NSIS, MSI) | ❌ Sin configurar |
| Licencia LICENSE en repo | ❌ Inexistente |
| Splash screen / Start Menu shortcut | ❌ |
| `Add/Remove Programs` entry coherente | 🟡 Default |

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| NSIS como bundle primario | Más ligero que MSI, mejor UX |
| Instalación **per-user** (no system-wide) | Evita UAC en install, simplifica updates |
| MSI complementario, NO primario | Solo para sysadmins |
| Sin firma en v1.0 | Coste alto (~$200/año mínimo), tracción aún no justificada |
| Iconos generados a partir de SVG master | Single source of truth |
| Versionado semver | Major.Minor.Patch — release notes claros |
| `windowsState` recordado entre sesiones | UX coherente |
| Tray icon: NO en v1.0 | Decisión: ClearTool es app de uso ocasional, no daemon |

---

## 4. Configuración Tauri

### 4.1 `src-tauri/tauri.conf.json` — versión final

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "ClearTool",
  "version": "1.0.0",
  "identifier": "app.cleartool",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "ClearTool",
        "width": 1280,
        "height": 800,
        "minWidth": 1024,
        "minHeight": 640,
        "center": true,
        "resizable": true,
        "decorations": true,
        "transparent": false,
        "fullscreen": false,
        "visible": false,
        "useHttpsScheme": true
      }
    ],
    "security": {
      "csp": "default-src 'self' tauri:; img-src 'self' data: tauri:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ipc: tauri: https://updates.cleartool.app"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis", "msi"],
    "publisher": "ClearTool Project",
    "copyright": "© 2026 ClearTool — MIT License",
    "category": "Utility",
    "shortDescription": "Limpieza, debloat y tweaks para Windows 11.",
    "longDescription": "ClearTool es una utilidad open-source de escritorio para Windows 11 que combina exploración de directorios, limpieza profunda de caché del sistema, eliminación de bloatware (Appx y uninstallers), gestión de servicios y tweaks de registro. Todas las operaciones destructivas crean punto de restauración y quedan registradas con instrucciones de reversa.",
    "homepage": "https://github.com/cleartool/cleartool",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": ["resources/catalogs/**/*.json"],
    "fileAssociations": [],
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper" },
      "allowDowngrades": false,
      "wix": {
        "language": ["es-ES", "en-US"],
        "upgradeCode": "{REPLACE-WITH-GUID-V4-CONSTANT}"
      },
      "nsis": {
        "displayLanguageSelector": true,
        "languages": ["Spanish", "English"],
        "installMode": "perUser",
        "installerIcon": "icons/icon.ico",
        "headerImage": "icons/installer-header.bmp",
        "sidebarImage": "icons/installer-sidebar.bmp",
        "compression": "lzma",
        "license": "../LICENSE"
      }
    }
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://updates.cleartool.app/manifest.json"],
      "pubkey": "REEMPLAZAR_TRAS_GENERAR_KEY_TAURI_SIGN"
    }
  }
}
```

### 4.2 Manifest dinámico debug vs release

Ya implementado en [AUDIT-FALLOS-CRITICOS §2](../AUDIT-FALLOS-CRITICOS.md). Confirmar que `src-tauri/build.rs` distingue:

```rust
fn main() {
    println!("cargo:rerun-if-changed=ClearTool.exe.manifest");

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let manifest = if profile == "release" {
        include_str!("manifests/release.manifest")
    } else {
        include_str!("manifests/debug.manifest")
    };

    let out = std::env::var("OUT_DIR").unwrap();
    std::fs::write(format!("{}/embedded.manifest", out), manifest).unwrap();

    println!("cargo:rustc-link-arg-bin=cleartool=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-bin=cleartool=/MANIFESTINPUT:{}/embedded.manifest", out);

    tauri_build::build()
}
```

#### Paso 4.2.1 — `src-tauri/manifests/release.manifest`

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    type="win32"
    name="app.cleartool"
    version="1.0.0.0"
    processorArchitecture="amd64"/>
  <description>ClearTool</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>  <!-- Windows 10 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>  <!-- Windows 11 -->
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2,PerMonitor</dpiAwareness>
      <activeCodePage xmlns="http://schemas.microsoft.com/SMI/2019/WindowsSettings">UTF-8</activeCodePage>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>
```

#### Paso 4.2.2 — `src-tauri/manifests/debug.manifest`

Idéntico al anterior pero con `<requestedExecutionLevel level="asInvoker" uiAccess="false"/>`.

---

## 5. Iconos

### 5.1 Master SVG

**Archivo:** `src-tauri/icons/source/cleartool-icon.svg` (nuevo)

> Diseño sugerido (la IA puede iterar): círculo cyan con un símbolo de "escoba digital" sobreimpreso, o el monograma "CT" en font geometric sans. Mantener simple — funciona a 16x16 y a 512x512.

### 5.2 Generación de todos los tamaños

**Archivo:** `scripts/build-icons.ps1` (nuevo)

```powershell
# Genera todos los tamaños desde el SVG master.
# Requiere ImageMagick: choco install imagemagick

$ErrorActionPreference = "Stop"
$src = "src-tauri/icons/source/cleartool-icon.svg"
$dst = "src-tauri/icons"

$sizes = @(16, 24, 32, 48, 64, 128, 256, 512, 1024)

foreach ($s in $sizes) {
    magick convert -background none -resize "${s}x${s}" $src "$dst/${s}x${s}.png"
}

# @2x (alta densidad)
magick convert -background none -resize 256x256 $src "$dst/128x128@2x.png"

# .ico multi-resolución (Windows)
magick convert "$dst/16x16.png" "$dst/24x24.png" "$dst/32x32.png" "$dst/48x48.png" "$dst/256x256.png" "$dst/icon.ico"

# .icns (Mac — opcional)
# (Generación de icns requiere iconutil de macOS; saltear si compilando en Windows)

# Imágenes del installer NSIS (BMP 24-bit)
magick convert "$dst/512x512.png" -resize "150x57" -background "#0a0e1a" -flatten BMP3:"$dst/installer-header.bmp"
magick convert "$dst/512x512.png" -resize "164x314" -background "#0a0e1a" -flatten BMP3:"$dst/installer-sidebar.bmp"

Write-Host "OK: iconos generados en $dst"
```

### 5.3 Checklist visual

- [ ] `16x16.png` legible (toolbar minimum).
- [ ] `32x32.png` consistente con `16x16`.
- [ ] `48x48.png` (Windows Explorer thumbnail).
- [ ] `256x256.png` (Start Menu).
- [ ] `icon.ico` multi-resolución (Windows requiere mínimo 16, 32, 48, 256).
- [ ] `installer-header.bmp` 150x57 px 24-bit BMP (sin alpha).
- [ ] `installer-sidebar.bmp` 164x314 px 24-bit BMP.

---

## 6. LICENSE y archivos legales

### 6.1 `LICENSE` en raíz

**Archivo:** `LICENSE` (nuevo)

```
MIT License

Copyright (c) 2026 ClearTool Project

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

### 6.2 `THIRD-PARTY-NOTICES.md`

Lista de dependencias con su licencia. Generar con `cargo about` o `license-checker` para npm.

```bash
# Rust
cargo install cargo-about
cargo about generate --manifest-path src-tauri/Cargo.toml > THIRD-PARTY-RUST.html

# Node
npx license-checker --json > third-party-npm.json
```

Compilar ambos en `THIRD-PARTY-NOTICES.md` listando solo dependencias direct + licenses.

### 6.3 `PRIVACY.md`

```markdown
# Privacidad — ClearTool

ClearTool **no recolecta, almacena ni envía datos personales**.

## Tráfico de red

Las únicas conexiones salientes son:

1. **Comprobación de actualizaciones** (si está habilitada en Ajustes → Comportamiento).
   - Endpoint: `https://updates.cleartool.app/manifest.json`
   - Datos enviados: User-Agent `ClearTool/1.0.0` y la versión actual en el query string.
   - Sin identificadores únicos. Sin cookies.

2. **Apertura del navegador en links explícitos** (botones "Repo en GitHub", "Reportar issue", etc.). Esto lanza el navegador del sistema con la URL; ClearTool no intermedia.

## Datos locales

- **`%APPDATA%\ClearTool\settings.json`** — tus preferencias.
- **`%APPDATA%\ClearTool\audit.jsonl`** — log de operaciones que realizaste.
- **`%LOCALAPPDATA%\ClearTool\logs\app.log`** — log técnico.

Ninguno se transmite a ningún sitio. Podés borrarlos manualmente sin consecuencias.

## Contacto

Cualquier duda: abrí un issue en https://github.com/cleartool/cleartool/issues
```

---

## 7. SmartScreen y reputación sin firma

### 7.1 Qué pasará al primer lanzamiento

1. Usuario descarga `ClearTool_1.0.0_x64-setup.exe`.
2. Edge/Chrome muestra warning "este archivo no es comúnmente descargado".
3. Usuario fuerza descarga.
4. Doble-click → **SmartScreen blocks** con "Windows protected your PC".
5. Usuario debe click "More info" → "Run anyway".

### 7.2 Cómo minimizar la fricción

| Estrategia | Aporte |
|---|---|
| Documentación clara en README + sitio web | Usuario sabe qué esperar antes de descargar |
| Página `/install` con screenshots de los pasos | Reduce abandono |
| Hash SHA256 publicado para verificación manual | Power users confían |
| Mensaje del installer NSIS al primer run: "Esta es la primera vez que ejecutas ClearTool. Es normal que Windows pida confirmación adicional" | Tono |
| Tracking pasivo de descargas (counters en GitHub Releases) sin telemetría intrusiva | Reputación se construye con volumen |
| **NO** distribuir vía Microsoft Store (al menos no v1.0) | Store requiere telemetría y vetting que va contra el proyecto |

### 7.3 Estrategia de firma futura (v1.1+)

Cuando haya:
- 5k+ descargas acumuladas verificables.
- Comunidad establecida (issues, PRs, stars).

Adquirir **certificado OV (Organization Validation)**:
- Sectigo / DigiCert: ~$200-400/año.
- Verificación legal de "entidad ClearTool Project" (cuasi-empresa o persona física).
- Firmar con `signtool` post-build:
  ```powershell
  signtool sign /tr http://timestamp.sectigo.com /td sha256 /fd sha256 /n "ClearTool Project" ClearTool_1.0.1_x64-setup.exe
  ```
- Después de ~3k firmas, Microsoft "calienta" la reputación SmartScreen.

Aún mejor: **certificado EV (Extended Validation)** — ~$600/año, **reputación instantánea**. Solo cuando haya recurso para amortizar.

---

## 8. Build pipeline local

### 8.1 Script `scripts/build-release.ps1`

**Archivo:** `scripts/build-release.ps1` (nuevo)

```powershell
# Build de release completo, paso a paso, con verificaciones.
$ErrorActionPreference = "Stop"

# 1. Validar entorno
$node = node --version
$rust = rustc --version
if ($null -eq $node) { throw "Node no instalado" }
if ($null -eq $rust) { throw "Rust no instalado" }

# 2. Sincronizar catálogos
& "$PSScriptRoot\sync-catalogs.ps1"

# 3. Generar iconos
& "$PSScriptRoot\build-icons.ps1"

# 4. Validar tests
Push-Location src-tauri
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --release
Pop-Location

# 5. Verificar TS
npx tsc --noEmit

# 6. Build
$env:CT_RELEASE_BUILD = "1"
npm run tauri build

# 7. Verificar artifacts
$nsis = Get-ChildItem -Recurse -Filter "ClearTool_*_x64-setup.exe" | Select-Object -First 1
$msi  = Get-ChildItem -Recurse -Filter "ClearTool_*_x64_en-US.msi" | Select-Object -First 1

if ($null -eq $nsis) { throw "NSIS .exe no generado" }
if ($null -eq $msi)  { Write-Warning "MSI no generado (no bloquea)" }

Write-Host "NSIS: $($nsis.FullName)" -ForegroundColor Green
Write-Host "Tamaño: $([math]::Round($nsis.Length/1MB, 2)) MB"

# 8. Generar hashes
$sha256 = (Get-FileHash $nsis.FullName -Algorithm SHA256).Hash
$sha256 | Out-File "$($nsis.DirectoryName)\$($nsis.BaseName).sha256.txt"
Write-Host "SHA256: $sha256"

Write-Host ""
Write-Host "Release build OK. Next: test manual en VM, publicar release en GitHub." -ForegroundColor Cyan
```

---

## 9. GitHub Release flow

### 9.1 Tag + release notes

```bash
# Tras commit final
git tag -a v1.0.0 -m "ClearTool 1.0.0 — Initial public release"
git push origin v1.0.0
```

### 9.2 Plantilla de release notes

**Archivo:** `docs/RELEASE-NOTES-TEMPLATE.md` (nuevo)

```markdown
# ClearTool {{VERSION}}

**Fecha:** {{DATE}}

## Resumen

Una sentencia describiendo qué trae esta versión.

## Novedades

- ...

## Correcciones

- ...

## Compatibilidad

- Windows 11 22H2+ (build 22621+).
- Recomendado: 4 GB RAM, 200 MB disco.

## Verificación de integridad

SHA-256 del instalador:
```
{{SHA256}}
```

## SmartScreen

Esta versión **no está firmada digitalmente**. Windows mostrará advertencia al primer lanzamiento. Click "More info" → "Run anyway". Consultá [docs/INSTALL.md](INSTALL.md) si necesitás guía.

## Cambios técnicos

- ...

## Reconocimientos

- ...
```

### 9.3 Asset checklist

Al subir el release en GitHub:

- [ ] `ClearTool_1.0.0_x64-setup.exe` (NSIS).
- [ ] `ClearTool_1.0.0_x64.msi` (MSI).
- [ ] `ClearTool_1.0.0_x64-setup.exe.sha256.txt`.
- [ ] `ClearTool_1.0.0_x64.msi.sha256.txt`.
- [ ] Release notes pegadas en el cuerpo del release.

---

## 10. Página `/install` del repo

**Archivo:** `docs/INSTALL.md` (nuevo)

```markdown
# Instalación de ClearTool

## Requisitos

- Windows 11 22H2+ (build 22621+).
- 200 MB disco libre.
- WebView2 Runtime (preinstalado en Win11; instalador lo verifica).

## Pasos

### 1. Descargar

Ir a https://github.com/cleartool/cleartool/releases/latest

Descargar `ClearTool_{VERSION}_x64-setup.exe`.

### 2. Verificar checksum (opcional pero recomendado)

```powershell
Get-FileHash .\ClearTool_1.0.0_x64-setup.exe -Algorithm SHA256
```

Comparar con el `.sha256.txt` adjunto al release.

### 3. Ejecutar el installer

Doble-click. Windows mostrará:

> Windows protected your PC

Esto es normal — ClearTool aún no está firmado digitalmente. Click "More info" → "Run anyway".

### 4. UAC

ClearTool requiere admin para tweaks de HKLM, servicios y restore points. Acepta el prompt UAC.

### 5. Primer lanzamiento

La app se abre. Si System Protection está OFF, verás un banner amarillo. Recomendamos activarlo antes de cualquier operación destructiva.

## Desinstalación

Panel de Control → Aplicaciones → ClearTool → Desinstalar.

**Nota:** `%APPDATA%\ClearTool\` (audit log, settings) NO se borra automáticamente. Borralo manualmente si querés.
```

---

## 11. Tests del bundle

| Test | Cómo |
|---|---|
| `.exe` instala correctamente | Snapshot VM → instalar → verificar Add/Remove → desinstalar limpio |
| `.msi` con `msiexec /i ClearTool.msi /quiet` instala silente | Test en VM |
| `.exe` recordó tamaño de ventana | Lanzar, redimensionar, cerrar, relanzar |
| Add/Remove muestra "ClearTool" + versión + publisher | Visualmente |
| Start Menu shortcut funciona | Click → lanza |
| Uninstall borra el `.exe` pero no `%APPDATA%\ClearTool\` | Manual |
| Reinstall sobre versión vieja respeta settings | Manual |

---

## 12. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| SmartScreen rechaza incluso "Run anyway" en políticas empresariales restrictivas | Documentar MSI alternativo para deployment GPO |
| WebView2 ausente en Win11 N edition | Bundle con `embedBootstrapper` lo instala on-demand |
| Per-user install en máquina multi-usuario | Cada usuario tiene su instalación; OK para target home |
| Iconos vacíos / placeholder en release | DoD bloquea release sin iconos verificados |
| Versión hard-coded en múltiples sitios desincronizada | Source of truth: `tauri.conf.json`. Script verifica match con `Cargo.toml` y `package.json` |
| Binario sin metadatos (Description, Company en propiedades del exe) | `tauri.conf.json` → `bundle.copyright`, `productName`, `publisher` los rellenan |

---

## 13. Definition of Done

- [ ] `tauri.conf.json` con versión final y metadatos completos.
- [ ] `src-tauri/manifests/release.manifest` y `debug.manifest` separados.
- [ ] `build.rs` selecciona manifest según `PROFILE`.
- [ ] `icons/` con todos los tamaños + `.ico` multi-res + BMPs del installer.
- [ ] `LICENSE` en raíz (MIT).
- [ ] `docs/PRIVACY.md` y `docs/INSTALL.md` creados.
- [ ] `THIRD-PARTY-NOTICES.md` generado.
- [ ] `scripts/build-release.ps1` ejecuta end-to-end sin errores.
- [ ] NSIS `.exe` instala en VM virgen + `Add/Remove Programs` correcto.
- [ ] MSI instala con `msiexec /quiet` en VM virgen.
- [ ] SHA256 del installer publicado.
- [ ] `docs/RELEASE-NOTES-TEMPLATE.md` listo.
- [ ] Versión sincronizada en `Cargo.toml`, `package.json`, `tauri.conf.json` (verificado por script).
- [ ] Commit `chore(release): pipeline de distribución v1.0`.

---

## 14. Próximo archivo

→ [11-AUTO-UPDATE.md](11-AUTO-UPDATE.md) — Tauri Updater + manifest server + canales estable/beta.
