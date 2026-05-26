# Paso 05 — winget submission

**Área**: 12-distribution
**Tiempo estimado**: 2-3 horas activas + 1-2 días de review
**Dependencias**: Paso 02 (release firmado público en GitHub)

## Qué hacemos

Submitir ClearTool al repositorio público de winget para que cualquiera pueda hacer `winget install ClearTool.ClearTool`.

## Pasos

### 1. Pre-requisitos

- Release público v1.0.0 con .exe firmado en GitHub Releases.
- Repository público en `microsoft/winget-pkgs`.
- Conoce el SHA256 de tu installer.

```powershell
# Calcular SHA256
Get-FileHash "ClearTool-Setup-1.0.0-x64.exe" -Algorithm SHA256
```

### 2. Instalar wingetcreate (helper)

```powershell
winget install Microsoft.WingetCreate
```

### 3. Generar los manifests

```powershell
wingetcreate new https://github.com/joowy/cleartool/releases/download/v1.0.0/ClearTool-Setup-1.0.0-x64.exe
```

`wingetcreate` te guiará con prompts:
- **Package Identifier**: `ClearTool.ClearTool` (formato `Publisher.PackageName`).
- **Package Version**: `1.0.0`.
- **Publisher**: tu nombre legal o de empresa.
- **PublisherUrl**: `https://cleartool.app`.
- **PublisherSupportUrl**: `https://github.com/joowy/cleartool/issues`.
- **Author**: igual al publisher.
- **PackageName**: `ClearTool`.
- **PackageUrl**: `https://cleartool.app`.
- **License**: `MIT` (o `Apache-2.0`).
- **LicenseUrl**: `https://github.com/joowy/cleartool/blob/main/LICENSE`.
- **ShortDescription**: "Sistema de limpieza, debloat y tweaks para Windows 11."
- **Description**: la versión larga del README intro.
- **Tags**: `windows utility cleaner debloat privacy`.
- **InstallerType**: `nullsoft` (NSIS).
- **InstallerSwitches**: `Silent: /S`, `SilentWithProgress: /S /SD IDOK`.
- **InstallerScope**: `machine`.

### 4. Output

`wingetcreate` genera 3 archivos:
```
manifests/c/ClearTool/ClearTool/1.0.0/
├── ClearTool.ClearTool.installer.yaml
├── ClearTool.ClearTool.locale.en-US.yaml
└── ClearTool.ClearTool.yaml
```

Ejemplo del installer.yaml:

```yaml
PackageIdentifier: ClearTool.ClearTool
PackageVersion: 1.0.0
InstallerType: nullsoft
Scope: machine
InstallerSwitches:
  Silent: /S
  SilentWithProgress: /S /SD IDOK
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/joowy/cleartool/releases/download/v1.0.0/ClearTool-Setup-1.0.0-x64.exe
    InstallerSha256: ABCD1234...
ManifestType: installer
ManifestVersion: 1.6.0
```

### 5. Validar localmente

```powershell
wingetcreate test
# o
winget validate manifests/c/ClearTool/ClearTool/1.0.0/
```

Debe pasar todas las checks. Si no, fix los errores que indique.

### 6. Submit con `wingetcreate`

```powershell
wingetcreate submit
```

Esto:
1. Fork de `microsoft/winget-pkgs` a tu cuenta.
2. Crear branch.
3. Commit los manifests.
4. PR a `microsoft/winget-pkgs`.

Alternativamente, hazlo manualmente: PR a `microsoft/winget-pkgs` con los YAMLs en `manifests/c/ClearTool/ClearTool/1.0.0/`.

### 7. Review process

Microsoft ejecuta validación automática primero (~10 min):
- SmartScreen scan del installer.
- VirusTotal scan.
- Manifest schema validation.

Si pasa, reviewer humano lo aprueba en 1-2 días hábiles.

Posibles motivos de rechazo:
- Installer no firmado (no es tu caso, ya está firmado).
- SHA256 no coincide.
- URL no accesible públicamente.
- Manifest YAML inválido.

### 8. Tras aprobación

`winget install ClearTool.ClearTool` ya funciona globalmente.

### 9. Updates posteriores

Para cada versión nueva, simplificable con `wingetcreate update`:

```powershell
wingetcreate update ClearTool.ClearTool `
  --urls "https://github.com/joowy/cleartool/releases/download/v1.0.1/ClearTool-Setup-1.0.1-x64.exe" `
  --version 1.0.1 `
  --submit
```

Idealmente automatizar como GitHub Action que corre tras cada release:

```yaml
# .github/workflows/winget-update.yml
on:
  release:
    types: [released]
jobs:
  update:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          $version = "${{ github.event.release.tag_name }}".TrimStart("v")
          $url = "${{ github.event.release.html_url }}/download/ClearTool-Setup-$version-x64.exe"
          .\wingetcreate.exe update ClearTool.ClearTool --urls $url --version $version --token ${{ secrets.WINGET_TOKEN }} --submit
        env:
          WINGET_TOKEN: ${{ secrets.WINGET_TOKEN }}
```

`WINGET_TOKEN` = PAT de GitHub con permiso a hacer PRs al fork de winget-pkgs.

## Criterio de done

- [ ] `wingetcreate new` genera manifests válidos.
- [ ] `winget validate` pasa.
- [ ] PR a `microsoft/winget-pkgs` submitted.
- [ ] PR aprobado y merged.
- [ ] `winget search ClearTool` devuelve resultado.
- [ ] `winget install ClearTool.ClearTool` instala correctamente.
- [ ] Workflow CI auto-submitea updates en cada release nueva (opcional, post-v1.0).

## Referencias

- [winget-pkgs repo](https://github.com/microsoft/winget-pkgs)
- [Authoring manifests docs](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest)
- [wingetcreate](https://github.com/microsoft/winget-create)
