---
name: tauri-release-signing
description: Build, firma y distribución de ClearTool (Tauri 2) para Windows — NSIS/MSI bundling, Authenticode con Azure Trusted Signing en GitHub Actions, firma del updater con claves tauri-signer (TAURI_SIGNING_PRIVATE_KEY) y latest.json, canales stable/beta/nightly, pipeline release.yml por tag semver, y submission a winget. Usar al configurar CI de release, code signing, auto-updater o cualquier tarea del milestone M5 (Distribución). Distingue las DOS firmas necesarias: Authenticode (SmartScreen) y la firma del updater (integridad del bundle).
---

# Skill: tauri-release-signing

Todo lo de M5 ("lista para gente desconocida"). El error más común aquí es confundir las dos firmas: son distintas y ambas son necesarias.

## Las DOS firmas (no son lo mismo)

1. **Authenticode / code signing** (Azure Trusted Signing): hace que Windows SmartScreen confíe en el `.exe`/`.msi`. Sin esto, SmartScreen bloquea y ~70% de usuarios cierra. Firma el binario y los instaladores.
2. **Firma del updater** (`tauri signer`, claves Ed25519 propias): garantiza al auto-updater que el bundle descargado es nuestro y no fue alterado. **No se puede desactivar** en el plugin updater. La clave pública va en `tauri.conf.json`; la privada solo en CI.

Una no reemplaza a la otra. El release necesita ambas.

## Code signing — Azure Trusted Signing (decisión del proyecto)

~$10/mes, sin certificado físico (HSM), integrable en CI con secrets. Alternativa OV (~$200/año, construye reputación 10-30 días) o EV (~$300/año, trust instantáneo pero requiere HSM).

Setup (la validación de identidad tarda 1-3 días → **empezar en M3, no en M5**):
1. Crear "Trusted Signing Account" en Azure.
2. "Identity Validation" (Microsoft verifica la entidad).
3. "Certificate Profile" público.
4. Secrets en GitHub: `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`.

```yaml
- name: Sign with Azure Trusted Signing
  uses: azure/trusted-signing-action@v0
  with:
    azure-tenant-id:     ${{ secrets.AZURE_TENANT_ID }}
    azure-client-id:     ${{ secrets.AZURE_CLIENT_ID }}
    azure-client-secret: ${{ secrets.AZURE_CLIENT_SECRET }}
    endpoint: https://eus.codesigning.azure.net/
    trusted-signing-account-name: cleartool
    certificate-profile-name: cleartool-public
    files-folder: src-tauri/target/release/bundle/
    files-folder-filter: exe,msi
```

Firmar: `ClearTool.exe`, `ClearTool-Setup-X.Y.Z.exe` (NSIS), `ClearTool-X.Y.Z.msi`.

## Firma del updater — claves tauri-signer

```bash
# generar una vez; el password va en CI, la privada NUNCA al repo
npm run tauri signer generate -- -w ~/.tauri/cleartool.key
```

- `TAURI_SIGNING_PRIVATE_KEY` y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` como secrets de CI. Si se pierde la privada, **no se pueden publicar más updates** a usuarios ya instalados → guardarla con respaldo seguro.
- La **clave pública** va en `tauri.conf.json > plugins.updater.pubkey` (es seguro commitearla).
- Al hacer `tauri build` con esas env vars, Tauri genera los bundles **y** sus `.sig`.

## tauri.conf.json — updater + NSIS

```jsonc
{
  "plugins": {
    "updater": {
      "active": true,
      "dialog": true,
      "pubkey": "<clave-publica-base64>",
      "endpoints": [
        "https://cleartool.app/updates/{{target}}/{{current_version}}",
        "https://github.com/joelsanchez/cleartool/releases/latest/download/latest.json"
      ]
    }
  },
  "bundle": {
    "windows": {
      "nsis": {
        "installerIcon": "icons/icon.ico",
        "installMode": "perMachine",
        "license": "../LICENSE",
        "languages": ["English", "Spanish"],
        "displayLanguageSelector": true,
        "allowDowngrades": false,
        "createDesktopShortcut": true,
        "createStartMenuShortcut": true
      }
    }
  }
}
```

Endpoint principal = web propia (control del canal); fallback = GitHub Releases (siempre disponible). HTTPS obligatorio en producción.

### latest.json (lo que sirve el endpoint)

```jsonc
{
  "version": "1.0.3",
  "notes": "Corregido lock de Spotify; +12 entradas de catálogo",
  "pub_date": "2026-05-27T10:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<contenido del .sig>",
      "url": "https://github.com/.../ClearTool_1.0.3_x64-setup.exe"
    }
  }
}
```

### Canales

| Canal | Audiencia | Frecuencia | Endpoint |
|-------|-----------|-----------|----------|
| stable | usuarios finales (default) | 4-8 sem | `/updates/stable/...` |
| beta | early adopters (opt-in) | 1-2 sem | `/updates/beta/...` |
| nightly | devs (flag) | cada commit a main | `/updates/nightly/...` |

Cada canal sirve su propio `latest.json`. Settings → Avanzado expone el selector.

## Pipeline release.yml (trigger por tag semver)

```yaml
on:
  push:
    tags: ['v*.*.*']         # v1.0.0, v1.0.1-beta.1, v1.1.0-rc.1
jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: npm ci
      - name: Build (firmado por updater vía env)
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        run: npm run tauri build -- --target x86_64-pc-windows-msvc
      - uses: azure/trusted-signing-action@v0       # Authenticode
        with: { endpoint: https://eus.codesigning.azure.net/, ... }
      - uses: tauri-apps/tauri-action@v0
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'ClearTool v__VERSION__'
          releaseDraft: true
          prerelease: ${{ contains(github.ref_name, '-') }}   # pre-release si el tag tiene -beta/-rc
```

Orden correcto: build (genera bundle + `.sig` del updater) → Authenticode → publicar release. Firmar Authenticode **después** del build pero el `.sig` del updater debe corresponder al binario ya firmado por Authenticode → en la práctica: firmar Authenticode primero los bundles y luego generar/adjuntar los `.sig` del updater sobre el binario final. Verificar en pruebas locales (ver checklist).

## winget (post-launch, día 1)

PR a `microsoft/winget-pkgs` con el manifest apuntando a la release firmada:

```yaml
PackageIdentifier: ClearTool.ClearTool
PackageVersion: 1.0.0
Installers:
  - Architecture: x64
    InstallerType: nullsoft
    InstallerUrl: https://github.com/joelsanchez/cleartool/releases/download/v1.0.0/ClearTool-Setup-1.0.0.exe
    InstallerSha256: <sha256>
    Scope: machine
```

Tras merge: `winget install ClearTool.ClearTool`.

## NSIS hooks (v1.0)

- Pre-install: comprobar build Windows ≥ 22000; abortar con mensaje claro si no.
- Uninstaller: NO borrar `%APPDATA%\ClearTool\` (logs+settings) sin preguntar ("¿Eliminar también configuración y audit log? recomendado: No").

## Portable mode

`tauri build` ya genera `ClearTool.exe` standalone → empaquetar como `ClearTool-portable-X.Y.Z.zip`. Limitaciones documentadas: sin auto-update, sin shortcut, UAC cada arranque. Settings se guardan igual en `%APPDATA%\ClearTool\`.

## Verificación local antes de etiquetar v1.0

- [ ] `tauri signer` keys generadas; pubkey en config; secrets en CI.
- [ ] Build local con env de firma genera `.sig` junto a cada bundle.
- [ ] Simular update v1.0.0 → v1.0.1: servir `latest.json` local, confirmar que la app detecta, descarga, valida firma e instala.
- [ ] `.exe`/`.msi` firmados por Authenticode → `signtool verify /pa` OK y SmartScreen no bloquea.
- [ ] Instalador probado en 3 VMs (22H2, 23H2, 24H2).
- [ ] CHANGELOG, LICENSE (MIT/Apache-2.0), PRIVACY-POLICY ("cero telemetría, cero networking salvo updater").

## Coste anual de referencia

Azure Trusted Signing ~$120 + dominio ~$15 + hosting $0 (free tier) + GitHub Releases $0 ≈ **$135/año**.
