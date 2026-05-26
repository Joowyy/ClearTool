# Paso 03 — NSIS installer polish

**Área**: 12-distribution
**Tiempo estimado**: 3-4 horas
**Dependencias**: ninguna (puede ir en paralelo a 01)

## Qué hacemos

Mejorar el instalador NSIS que genera Tauri: branding, idiomas, license display, post-install offer "crear restore point".

## Archivos

- `src-tauri/tauri.conf.json` (modificar bundle.windows.nsis)
- `src-tauri/installer/header.bmp` (nuevo asset, 150x57)
- `src-tauri/installer/sidebar.bmp` (nuevo asset, 164x314)
- `src-tauri/installer/preinstall.nsh` (nuevo NSIS hook)
- `src-tauri/installer/postinstall.nsh` (nuevo NSIS hook)
- `LICENSE` (asegurar que existe en raíz)

## Cómo

### 1. Configurar tauri.conf.json

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "",
      "nsis": {
        "installerIcon": "icons/icon.ico",
        "installMode": "perMachine",
        "headerImage": "installer/header.bmp",
        "sidebarImage": "installer/sidebar.bmp",
        "license": "../LICENSE",
        "languages": ["English", "Spanish"],
        "displayLanguageSelector": true,
        "compression": "lzma",
        "allowDowngrades": false,
        "createDesktopShortcut": true,
        "createStartMenuShortcut": true,
        "installerHooks": "installer/hooks.nsh"
      }
    }
  }
}
```

### 2. Crear assets BMP

Las dimensiones son fijas en NSIS Modern UI:
- `header.bmp`: 150x57 (logo arriba del wizard).
- `sidebar.bmp`: 164x314 (banner lateral en página de welcome).

Crear en cualquier editor (Photoshop/GIMP/Figma → Export BMP).

Estilo recomendado:
- Fondo: oscuro cyan (`#0a0a1f`).
- Logo ClearTool centrado.
- Texto: "ClearTool" en blanco, 24pt en sidebar; sólo logo en header.

### 3. NSIS hooks

```nsis
; src-tauri/installer/hooks.nsh

; Preinstall — verificar Windows version
!macro NSIS_HOOK_PREINSTALL
  ; Check Windows 11 (build >= 22000)
  ${If} ${AtLeastBuild} 22000
    ; OK
  ${Else}
    MessageBox MB_OK|MB_ICONSTOP "ClearTool requiere Windows 11 (build 22000+).$\nTu sistema no es compatible."
    Quit
  ${EndIf}
!macroend

; Postinstall — ofrecer crear restore point
!macro NSIS_HOOK_POSTINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION "¿Crear un punto de restauración del sistema antes de usar ClearTool?$\n(Recomendado para usuarios nuevos)" /SD IDYES IDNO skip_restore
    nsExec::Exec 'powershell -NoProfile -Command "Checkpoint-Computer -Description \"ClearTool — pre-uso\" -RestorePointType MODIFY_SETTINGS"'
  skip_restore:
!macroend
```

### 4. Verifica build local

```bash
npm run tauri build
```

El instalador se genera en `src-tauri/target/release/bundle/nsis/ClearTool_X.Y.Z_x64-setup.exe`.

Ejecutarlo en una VM limpia y verificar:
- Selector de idioma aparece (ES / EN).
- Header BMP visible en cada página del wizard.
- Sidebar BMP en la página de welcome.
- License page muestra contenido del LICENSE.
- Post-install pregunta por restore point.
- Si Windows < 11, el instalador aborta con mensaje claro.

### 5. Uninstaller

Tauri genera uninstaller automáticamente. Verificar:
- Aparece en "Apps & features" como "ClearTool".
- Al desinstalar, NO borra `%APPDATA%\ClearTool\` sin preguntar.

Para preguntar antes de borrar config, añadir hook:

```nsis
; src-tauri/installer/hooks.nsh (continuación)

!macro NSIS_HOOK_PREUNINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION "¿Eliminar también configuración y audit log?$\n(Si dices No, podrás reinstalar y conservar settings)" /SD IDNO IDYES delete_config IDNO skip_delete_config
  delete_config:
    RMDir /r "$APPDATA\ClearTool"
  skip_delete_config:
!macroend
```

### 6. Caveat sobre perMachine vs perUser

`perMachine` requiere UAC al instalar. Pro: shortcuts para todos los usuarios. Con: UAC prompt.

`perUser` no requiere UAC al instalar pero los shortcuts son solo del usuario actual.

ClearTool elige `perMachine` porque la app necesita admin para funcionar de todas formas — pedirlo en el install es honesto.

## Criterio de done

- [ ] `header.bmp` (150x57) y `sidebar.bmp` (164x314) creados con branding.
- [ ] `tauri.conf.json` actualizado con licencia + idiomas + hooks.
- [ ] Instalador local muestra selector de idioma.
- [ ] License page muestra contenido del LICENSE.
- [ ] Postinstall ofrece crear restore point.
- [ ] Preuninstall pregunta por borrar config.
- [ ] Test en VM Win11: install OK, uninstall OK.
- [ ] Test en VM Win 10 (build < 22000): instalador aborta con mensaje claro.
