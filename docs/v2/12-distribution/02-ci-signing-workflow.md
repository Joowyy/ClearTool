# Paso 02 — CI workflow con firma automática

**Área**: 12-distribution
**Tiempo estimado**: 3-4 horas
**Dependencias**: Paso 01 (cert + secrets configurados)

## Qué hacemos

GitHub Actions workflow que:
1. Buildea ClearTool en Windows runner.
2. Firma binario + MSI con Azure Trusted Signing.
3. Crea GitHub Release con artefactos firmados.

## Archivos

- `.github/workflows/release.yml` (nuevo)
- `.github/workflows/build.yml` (nuevo, build sin firmar para PRs)

## Cómo

### 1. Workflow de build (para PRs y main)

```yaml
# .github/workflows/build.yml
name: Build

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri"

      - name: Install deps
        run: npm ci

      - name: Lint (TS)
        run: npm run typecheck && npm run lint

      - name: Cargo check
        run: cd src-tauri && cargo clippy -- -D warnings

      - name: Cargo test
        run: cd src-tauri && cargo test

      - name: Build (no sign)
        run: npm run tauri build -- --target x86_64-pc-windows-msvc

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: cleartool-unsigned-${{ github.sha }}
          path: |
            src-tauri/target/release/ClearTool.exe
            src-tauri/target/release/bundle/nsis/*.exe
          retention-days: 14
```

### 2. Workflow de release (firma + publica)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - "v*.*.*"
  workflow_dispatch:
    inputs:
      tag:
        description: "Tag (ej. v1.0.0)"
        required: true

permissions:
  contents: write   # crear release

jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # para changelog

      - uses: actions/setup-node@v4
        with: { node-version: "20", cache: "npm" }

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: "src-tauri" }

      - name: Install deps
        run: npm ci

      - name: Build
        run: npm run tauri build -- --target x86_64-pc-windows-msvc

      - name: Install AzureSignTool
        run: dotnet tool install --global AzureSignTool

      - name: Sign binaries
        env:
          AZURE_TENANT_ID: ${{ secrets.AZURE_TENANT_ID }}
          AZURE_CLIENT_ID: ${{ secrets.AZURE_CLIENT_ID }}
          AZURE_CLIENT_SECRET: ${{ secrets.AZURE_CLIENT_SECRET }}
          TS_ACCOUNT: ${{ secrets.TRUSTED_SIGNING_ACCOUNT }}
          TS_PROFILE: ${{ secrets.TRUSTED_SIGNING_PROFILE }}
          TS_ENDPOINT: ${{ secrets.TRUSTED_SIGNING_ENDPOINT }}
        run: |
          $files = @(
            "src-tauri\target\release\ClearTool.exe",
            (Get-ChildItem "src-tauri\target\release\bundle\nsis\*.exe" | Select-Object -ExpandProperty FullName)
          )
          foreach ($f in $files) {
            Write-Host "Firmando: $f"
            AzureSignTool sign `
              -tr http://timestamp.acs.microsoft.com `
              -kvu $env:TS_ENDPOINT `
              -kvi $env:AZURE_TENANT_ID `
              -kvs $env:AZURE_CLIENT_SECRET `
              -kva $env:AZURE_CLIENT_ID `
              -kvc $env:TS_PROFILE `
              -tmd $env:TS_ACCOUNT `
              -fd sha256 `
              -v `
              $f
            if ($LASTEXITCODE -ne 0) { throw "Sign failed para $f" }
          }

      - name: Verify signatures
        run: |
          Get-ChildItem "src-tauri\target\release\bundle\nsis\*.exe" | ForEach-Object {
            $sig = Get-AuthenticodeSignature $_.FullName
            if ($sig.Status -ne 'Valid') { throw "Firma inválida en $($_.Name): $($sig.Status)" }
            Write-Host "OK: $($_.Name) firmado por $($sig.SignerCertificate.Subject)"
          }

      - name: Generate changelog
        id: changelog
        shell: pwsh
        run: |
          $tag = "${{ github.ref_name }}"
          $prevTag = git describe --tags --abbrev=0 "$tag^" 2>$null
          if (-not $prevTag) { $prevTag = "" }
          $log = if ($prevTag) {
            git log --pretty=format:"- %s" "$prevTag..$tag"
          } else {
            git log --pretty=format:"- %s" "$tag"
          }
          $body = @"
          ## Changelog desde $prevTag

          $log

          ## Descargas

          - **Instalador**: ``ClearTool-Setup-*.exe`` (recomendado)
          - **Portable**: ``ClearTool-portable-*.zip``

          Firmado con Azure Trusted Signing — SmartScreen confía automáticamente.
          "@
          $body | Out-File changelog.md -Encoding utf8
          Write-Host "Changelog generado"

      - name: Create Portable ZIP
        run: |
          $version = "${{ github.ref_name }}".TrimStart("v")
          Compress-Archive -Path "src-tauri\target\release\ClearTool.exe" `
            -DestinationPath "ClearTool-portable-$version.zip"

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          name: "ClearTool ${{ github.ref_name }}"
          body_path: changelog.md
          draft: true   # revisar antes de publicar
          files: |
            src-tauri/target/release/bundle/nsis/*.exe
            ClearTool-portable-*.zip
```

### 3. Probar el workflow

#### a) Push to PR — build sin firmar

Hacer un PR cualquiera y verificar que `build.yml` corre OK (sin secretos de signing necesarios).

#### b) Tag de prueba — release firmado

```bash
git tag v0.5.0-test
git push origin v0.5.0-test
```

En GitHub Actions, ver el workflow `release.yml`. Tras ~10 minutos debería crear el release en draft mode.

Verificar:
1. Artefactos descargables.
2. Firma válida (`Get-AuthenticodeSignature` localmente sobre el .exe descargado).
3. SmartScreen NO bloquea al ejecutar (sin advertencia "publisher unknown").

Si todo OK, publicar el release manualmente desde GitHub UI.

### 4. Caveat sobre cost

Cada firma cuesta ~$0.005. Una release típica firma ~3 archivos. Con 10 releases/año → ~$0.15/año en firmas + $120 de base.

Si haces muchas builds (nightly canal), considerar firmar solo en main / tag, no en cada commit.

## Criterio de done

- [ ] `.github/workflows/build.yml` corre en cada PR.
- [ ] `.github/workflows/release.yml` corre en tag `v*.*.*`.
- [ ] Tag de prueba produce release draft con artefactos firmados.
- [ ] `Get-AuthenticodeSignature` sobre el .exe descargado devuelve `Valid`.
- [ ] SmartScreen NO bloquea ejecución del .exe firmado.
- [ ] Portable ZIP incluido en release.
- [ ] Changelog auto-generado en el body del release.
