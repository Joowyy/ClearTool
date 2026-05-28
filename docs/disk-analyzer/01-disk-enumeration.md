# 01 — Enumeración de discos del sistema

## Qué necesitamos exponer

El frontend pinta un `<DriveSelector>` que recibe una lista de objetos
`DriveListing`:

```ts
export interface DriveListing {
  letter: string;          // "C", "D", "E"...
  rootPath: string;        // "C:\\"
  label: string;           // "OS", "Datos", "" si no tiene etiqueta
  filesystem: string;      // "NTFS", "FAT32", "exFAT", "ReFS", ""
  driveType: DriveType;    // "fixed" | "removable" | "network" | "cdRom" | "ramDisk" | "unknown"
  totalBytes: number;      // 1_000_204_886_016
  freeBytes: number;       // 234_567_890
  isReady: boolean;        // false para unidades de CD vacías o de red offline
}
```

El `SystemSummary.drives` actual sólo tiene letra, total y free. Lo
extendemos sin romperlo: añadimos un **nuevo** comando `list_drives` que
devuelve el `DriveListing` completo, y dejamos `drives` en `SystemSummary`
como está (lo usa el home/dashboard).

## Backend — pasos

1. **Modelo nuevo** en `src-tauri/src/models/disk.rs`:

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]
   pub struct DriveListing {
       pub letter: String,
       pub root_path: String,
       pub label: String,
       pub filesystem: String,
       pub drive_type: DriveType,
       pub total_bytes: u64,
       pub free_bytes: u64,
       pub is_ready: bool,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]
   pub enum DriveType {
       Fixed,
       Removable,
       Network,
       CdRom,
       RamDisk,
       Unknown,
   }
   ```

2. **Implementación** en `src-tauri/src/domain/disk.rs`:

   ```rust
   pub fn list_drives() -> AppResult<Vec<DriveListing>> {
       #[cfg(target_os = "windows")]
       return crate::platform::filesystem::enumerate_drives();
       #[cfg(not(target_os = "windows"))]
       return Ok(Vec::new());
   }
   ```

3. **Plataforma** — añadir a `src-tauri/src/platform/filesystem.rs`:

   - `enumerate_drives()` que use `GetLogicalDrives` + por cada letra
     `GetDriveTypeW`, `GetVolumeInformationW`, `GetDiskFreeSpaceExW`.
   - Tolerar errores por letra: una unidad de red caída no debe romper la
     enumeración del resto. Cuando falle, marca `isReady: false`.

4. **IPC** — añadir `ipc::disk::list_drives` y registrarlo en `lib.rs`
   junto a `build_treemap_data`.

5. **Cliente TS** — exponer `listDrives()` en `src/api/client.ts` y el
   tipo en `src/api/types.ts`.

## APIs Win32 a usar

| API | Para qué |
|-----|----------|
| `GetLogicalDrives` | Bitmask con qué letras existen (A=bit0 .. Z=bit25). |
| `GetDriveTypeW` | Devuelve `DRIVE_FIXED`, `DRIVE_REMOVABLE`, etc. |
| `GetVolumeInformationW` | Label + filesystem. Puede fallar si no ready → tolerar. |
| `GetDiskFreeSpaceExW` | Total bytes y free bytes. |

Todas viven en `windows::Win32::Storage::FileSystem` (ya estamos usando
`GetDiskFreeSpaceExA` con la `A` ANSI; pasamos a `W` para soportar labels
con caracteres no-ASCII).

## UX del selector

- Componente `<DriveSelector>` (shadcn-style dropdown).
- Cada item muestra: icono (HD/USB/red/CD según `driveType`), `C:` en
  mono, label entre paréntesis, barra horizontal con `(total - free) /
  total`, y `XX.X GB libres / YY.Y GB`.
- Discos no listos aparecen tachados, no clicables.
- Persistir el último disco elegido en `localStorage` (`cleartool.diskAnalyzer.lastDrive`).

## Refresh

- Botón pequeño "↻" al lado del selector que reinvoca `listDrives()`.
- Listener opcional iter-2: `WM_DEVICECHANGE` para refrescar automáticamente
  al conectar/desconectar USB. Fuera del MVP.
