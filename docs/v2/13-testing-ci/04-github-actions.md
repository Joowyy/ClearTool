# Paso 04 — GitHub Actions: build + release pipelines

**Área**: 13-testing-ci
**Tiempo estimado**: 2-3 horas
**Dependencias**: ninguna (puede ir en M1)

## Qué hacemos

Workflows GitHub Actions completos:
1. `build.yml` — corre en cada PR y push a main (sin firmar).
2. `validate-catalogs.yml` — valida JSON contra schemas.
3. `release.yml` — corre en tags `v*.*.*` (firmado y publicado).
4. `winget-update.yml` — auto-submit a winget tras release.

## Archivos

- `.github/workflows/build.yml` (nuevo)
- `.github/workflows/validate-catalogs.yml` (nuevo)
- `.github/workflows/release.yml` (ya en `12-distribution/02`, integrar)
- `.github/workflows/winget-update.yml` (ya en `12-distribution/05`)
- `.github/dependabot.yml` (opcional)

## Cómo

### 1. `build.yml` — CI en cada PR

Ver detalle completo en `12-distribution/02-ci-signing-workflow.md`. Resumen:

```yaml
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
      - uses: actions/setup-node@v4
        with: { node-version: "20", cache: "npm" }
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: "src-tauri" }

      - run: npm ci

      - name: TypeScript check
        run: npx tsc --noEmit

      - name: ESLint
        run: npm run lint

      - name: Rust clippy
        run: cd src-tauri && cargo clippy -- -D warnings

      - name: Rust tests
        run: cd src-tauri && cargo test

      - name: Frontend tests
        run: npm test

      - name: Build (no sign)
        run: npm run tauri build -- --target x86_64-pc-windows-msvc

      - uses: actions/upload-artifact@v4
        with:
          name: cleartool-unsigned-${{ github.sha }}
          path: |
            src-tauri/target/release/ClearTool.exe
            src-tauri/target/release/bundle/nsis/*.exe
          retention-days: 14
```

### 2. `validate-catalogs.yml`

```yaml
name: Validate Catalogs

on:
  push:
    paths:
      - ".claude/skills/**/*.json"
      - ".claude/skills/**/*.schema.json"
  pull_request:
    paths:
      - ".claude/skills/**/*.json"
      - ".claude/skills/**/*.schema.json"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: "20" }

      - name: Install ajv
        run: npm i -g ajv-cli ajv-formats

      - name: Validate bloatware catalog
        run: ajv validate
              -s .claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.schema.json
              -d .claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json
              --spec=draft7 -c ajv-formats --strict=false

      - name: Validate cache locations
        run: ajv validate
              -s .claude/skills/cache-scanner/RESOURCES/cache-locations.schema.json
              -d .claude/skills/cache-scanner/RESOURCES/cache-locations.json
              --spec=draft7 -c ajv-formats --strict=false

      - name: Validate registry tweaks
        run: ajv validate
              -s .claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.schema.json
              -d .claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.json
              --spec=draft7 -c ajv-formats --strict=false

      - name: Validate privacy presets
        run: ajv validate
              -s .claude/skills/windows-registry-ops/RESOURCES/privacy-presets.schema.json
              -d .claude/skills/windows-registry-ops/RESOURCES/privacy-presets.json
              --spec=draft7 -c ajv-formats --strict=false
```

### 3. `release.yml` (ya escrito en 12-distribution/02)

Aquí solo confirmar:
- Triggers en `push: tags: ['v*.*.*']`.
- Build + firmar + generar latest.json + upload a Release.
- Crea draft (no publica auto — review humano).

### 4. `winget-update.yml`

```yaml
name: Update winget manifest

on:
  release:
    types: [released]

jobs:
  winget:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download wingetcreate
        run: |
          iwr https://aka.ms/wingetcreate/latest -OutFile wingetcreate.exe

      - name: Update manifest
        run: |
          $version = "${{ github.event.release.tag_name }}".TrimStart("v")
          $url = "${{ github.event.release.html_url }}/download/ClearTool-Setup-$version-x64.exe"
          .\wingetcreate.exe update ClearTool.ClearTool `
            --urls $url `
            --version $version `
            --token ${{ secrets.WINGET_GITHUB_TOKEN }} `
            --submit
```

`WINGET_GITHUB_TOKEN` = PAT con permiso a hacer PRs al fork del usuario en `microsoft/winget-pkgs`.

### 5. `dependabot.yml` (opcional)

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
  - package-ecosystem: "cargo"
    directory: "/src-tauri"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
```

### 6. Badges en README

Tras el primer build verde:

```markdown
[![Build](https://github.com/joowy/cleartool/actions/workflows/build.yml/badge.svg)](https://github.com/joowy/cleartool/actions/workflows/build.yml)
[![Catalogs](https://github.com/joowy/cleartool/actions/workflows/validate-catalogs.yml/badge.svg)](https://github.com/joowy/cleartool/actions/workflows/validate-catalogs.yml)
[![Release](https://img.shields.io/github/v/release/joowy/cleartool)](https://github.com/joowy/cleartool/releases/latest)
```

### 7. Branch protection

GitHub repo → Settings → Branches → Add rule para `main`:
- Require status checks: `Build`, `Validate Catalogs`.
- Require PRs to be reviewed (si trabajas con colaboradores).
- No allow force pushes.

### 8. Caveat de tiempo en CI

Cada build en `windows-latest` tarda:
- Cold cache: ~12-15 minutos.
- Warm cache (Swatinem/rust-cache): ~4-6 minutos.
- Sólo TypeScript/Rust tests: ~2 minutos.

Si la build se vuelve lenta:
- Splittear en jobs paralelos (build / tests / lints).
- Considera `windows-2022` específico si `windows-latest` cambia y rompe algo.

## Criterio de done

- [ ] `build.yml` corre verde en main + PR.
- [ ] `validate-catalogs.yml` corre verde tras cualquier cambio en JSON.
- [ ] `release.yml` produce releases firmados en tags.
- [ ] `winget-update.yml` actualiza winget tras cada release.
- [ ] Badges en README muestran estado.
- [ ] Branch protection activa.
- [ ] Caches Rust + npm hits >80% en CI tras estabilizar.

## Métricas de éxito CI

| Métrica | Target |
|---------|--------|
| Tiempo build (warm) | <6 min |
| Tiempo test (Rust + TS) | <2 min |
| Tiempo total PR | <8 min |
| Tasa de fallos por flakiness | <5% |
| % de PRs que pasan al primer intento | >80% |
