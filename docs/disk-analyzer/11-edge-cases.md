# 11 — Edge cases

## Reparse points / junctions / symlinks

- Detectar con `meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT`
  (en Windows: `windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT`).
- Default: NO seguir. Devolver el nodo con `kind: Symlink` o `Junction`,
  `size: 0`, `truncated: false`. Tooltip explica "símbolo no seguido".
- Activable desde el header. Al activar, llevar `HashSet<canonical_path>`
  para abortar bucles.
- Casos típicos en Windows que se ignoran por defecto:
  - `C:\Users\All Users` → `C:\ProgramData`.
  - `C:\Users\Default User` → `C:\Users\Default`.
  - `C:\Documents and Settings` → `C:\Users`.

## Permisos denegados

- Carpetas restringidas (`C:\System Volume Information`,
  `C:\$Recycle.Bin`, perfiles de otros usuarios) lanzan
  `ERROR_ACCESS_DENIED`.
- Política: capturar, contar como `size: 0`, añadir el path a
  `report.errors` con prefijo `"access denied: ..."`. NO interrumpir.
- Si la app corre con UAC, las áreas administrativas (Windows, Program Files)
  son legibles. Si no, gran parte se queda en negro. Mostrar el banner
  "Reinicia como admin para análisis completo" si > 1 % del disco es
  inaccesible.

## Paths > MAX_PATH (260 chars)

- En Windows el prefijo `\\?\C:\path...` permite paths hasta 32767.
- `std::fs::read_dir` en Rust moderno acepta paths largos.
- Si encontramos un fallo concreto, abrir con `OpenOptions` + path
  prefijado. Documentar el caso.

## Drives no listos (CD vacío, USB removida en medio del scan)

- `is_ready: false` en la enumeración → no aparece como seleccionable.
- Si la unidad se retira durante el scan, el walker recibirá errores en
  bucle: detectar `ERROR_NO_MEDIA` y abortar el scan completo con
  `AppError::Io("device disconnected")`.

## Recycle bin

- `C:\$Recycle.Bin` está restringido por SID y normalmente da access
  denied a la enumeración. Si tenemos elevación, sí lo vemos.
- Tratamiento: nodo separado con etiqueta "Papelera de reciclaje", el
  total real lo sacamos sumando.

## OneDrive offline / Files On-Demand

- Archivos con `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS` o
  `FILE_ATTRIBUTE_RECALL_ON_OPEN` tienen `meta.len()` = tamaño lógico
  pero ocupan 0 en disco.
- Detección + nodo etiquetado "OneDrive (cloud)". `size_on_disk` real
  sería `GetCompressedFileSizeW` — fuera del MVP.

## Volúmenes montados (mount points)

- Cuando una unidad se monta en una carpeta (ej. `C:\Mount\D` con la D
  física montada ahí), aparece como reparse point apuntando a otro
  volumen. Política: tratar igual que junction, no seguir por defecto.

## Compressed / Encrypted / Sparse files

- Atributo `FILE_ATTRIBUTE_COMPRESSED` / `_ENCRYPTED` / `_SPARSE_FILE`.
- En MVP: usar `metadata.len()` (lógico). Físico real exigiría
  `GetCompressedFileSizeW`, fuera del MVP.
- Mostrar el icono con un asterisco para señalar la diferencia (opcional).

## Discos de red

- Pueden ser lentos o caer. Política MVP: NO permitir analizar drives
  `driveType: network` — son demasiado impredecibles. UX: aparecen
  listados pero deshabilitados con tooltip "No soportado en esta versión".

## Caracteres exóticos en nombres

- Path con surrogate pairs UTF-16 mal-formados → `to_string_lossy()`
  los reemplaza por U+FFFD. Es aceptable; logarlo en `report.errors`
  para diagnóstico.

## Race con escritura activa

- Si el SO está escribiendo (instalando algo, descargando) durante el
  scan, los tamaños son del momento del walk, no consistentes globales.
  Es esperable; lo documentamos en el tooltip del botón "Analizar".

## Espacio "fantasma" (analizado < usado)

- `usado = drive_total - free`.
- `analizado = totalBytes` del report.
- Diferencia habitual: 1–10 GB (System Volume Information, Recycle.Bin
  de otros SIDs, hibernación, pagefile, shadow copies).
- En la donut mostramos `usado` (verdad oficial del SO) y `analizado` por
  debajo en gris.
