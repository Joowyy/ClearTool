// platform/filesystem.rs — operaciones de bajo nivel sobre el FS.
//
// Devuelve datos crudos (tamaños, listados); la decisión de qué borrar o
// hasta qué profundidad recorrer es del `domain`, no de aquí.

use crate::core::AppResult;
use std::path::Path;
use walkdir::WalkDir;

// ── enumeración de drives (Disk Analyzer) ─────────────────────────────────

#[cfg(target_os = "windows")]
pub fn enumerate_drives() -> AppResult<Vec<crate::models::disk::DriveListing>> {
    use crate::models::disk::{DriveListing, DriveType};
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
    };

    // Constantes Win32 DRIVE_* (de WinBase.h). No están expuestas como
    // símbolos en windows-rs 0.58, las inlineamos aquí.
    const DRIVE_NO_ROOT_DIR: u32 = 1;
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_REMOTE: u32 = 4;
    const DRIVE_CDROM: u32 = 5;
    const DRIVE_RAMDISK: u32 = 6;

    let mut out = Vec::new();
    let mask = unsafe { GetLogicalDrives() };
    if mask == 0 {
        return Ok(out);
    }

    for i in 0u32..26 {
        if (mask & (1 << i)) == 0 {
            continue;
        }
        let letter_char = (b'A' + i as u8) as char;
        let root_path = format!("{}:\\", letter_char);
        let wide: Vec<u16> = root_path.encode_utf16().chain(std::iter::once(0)).collect();
        let pcwstr = PCWSTR(wide.as_ptr());

        let dt = unsafe { GetDriveTypeW(pcwstr) };
        let drive_type = match dt {
            DRIVE_FIXED => DriveType::Fixed,
            DRIVE_REMOVABLE => DriveType::Removable,
            DRIVE_REMOTE => DriveType::Network,
            DRIVE_CDROM => DriveType::CdRom,
            DRIVE_RAMDISK => DriveType::RamDisk,
            DRIVE_NO_ROOT_DIR => continue, // no existe la letra realmente
            _ => DriveType::Unknown,
        };

        // Volumen: label + filesystem.
        let mut label_buf = [0u16; 256];
        let mut fs_buf = [0u16; 64];
        let mut serial: u32 = 0;
        let mut max_component: u32 = 0;
        let mut fs_flags: u32 = 0;
        let vol_ok = unsafe {
            GetVolumeInformationW(
                pcwstr,
                Some(&mut label_buf),
                Some(&mut serial),
                Some(&mut max_component),
                Some(&mut fs_flags),
                Some(&mut fs_buf),
            )
        }
        .is_ok();

        let label = if vol_ok { wide_to_string(&label_buf) } else { String::new() };
        let filesystem = if vol_ok { wide_to_string(&fs_buf) } else { String::new() };

        // Tamaños.
        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let cap_ok = unsafe {
            GetDiskFreeSpaceExW(
                pcwstr,
                Some(&mut free_bytes_available),
                Some(&mut total_bytes),
                None,
            )
        }
        .is_ok();

        // is_ready: GetVolumeInformation + GetDiskFreeSpaceEx ambos OK.
        let is_ready = vol_ok && cap_ok;

        out.push(DriveListing {
            letter: letter_char.to_string(),
            root_path,
            label,
            filesystem,
            drive_type,
            total_bytes: if cap_ok { total_bytes } else { 0 },
            free_bytes: if cap_ok { free_bytes_available } else { 0 },
            is_ready,
        });
    }

    Ok(out)
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_drives() -> AppResult<Vec<crate::models::disk::DriveListing>> {
    Ok(Vec::new())
}

/// Devuelve (total_bytes, free_bytes) del drive que contiene `root`.
/// Best-effort: si falla, devuelve `None`.
#[cfg(target_os = "windows")]
pub fn drive_capacity_and_free(root: &str) -> Option<(u64, u64)> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    // Asegurarse de tener forma "X:\\".
    let normalized = if root.len() >= 2 && root.as_bytes()[1] == b':' {
        format!("{}:\\", &root[..1])
    } else {
        root.to_string()
    };
    let wide: Vec<u16> = normalized
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut free: u64 = 0;
    let mut total: u64 = 0;
    unsafe {
        GetDiskFreeSpaceExW(PCWSTR(wide.as_ptr()), Some(&mut free), Some(&mut total), None)
            .ok()?;
    }
    Some((total, free))
}

#[cfg(not(target_os = "windows"))]
pub fn drive_capacity_and_free(_root: &str) -> Option<(u64, u64)> {
    None
}

#[cfg(target_os = "windows")]
fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

/// Devuelve (bytes_lógicos, archivos, directorios) recorriendo `root`.
/// No sigue symlinks ni junctions por defecto.
pub fn directory_stats(root: &Path, follow_reparse_points: bool) -> AppResult<(u64, u64, u64)> {
    if !root.exists() {
        return Ok((0, 0, 0));
    }

    let mut bytes: u64 = 0;
    let mut files: u64 = 0;
    let mut dirs: u64 = 0;

    let walker = WalkDir::new(root)
        .follow_links(follow_reparse_points)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        let ft = entry.file_type();
        if ft.is_file() {
            files += 1;
            if let Ok(meta) = entry.metadata() {
                bytes = bytes.saturating_add(meta.len());
            }
        } else if ft.is_dir() {
            dirs += 1;
        }
    }
    Ok((bytes, files, dirs))
}

/// Lista nivel a nivel (children directos). Para archivos devuelve el tamaño
/// del metadata (operación O(1)). Para directorios devuelve 0 — el cómputo
/// recursivo es caro (puede tardar minutos en un disco grande) y se hace
/// on-demand vía `directory_stats` cuando el usuario expanda el directorio.
pub fn list_children_with_sizes(
    root: &Path,
    _follow_reparse_points: bool,
) -> AppResult<Vec<(String, String, bool, u64)>> {
    let mut out = Vec::new();
    let rd = match std::fs::read_dir(root) {
        Ok(rd) => rd,
        Err(_) => return Ok(out),
    };

    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let is_dir = ft.is_dir();
        let size = if is_dir {
            0
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };
        out.push((path.to_string_lossy().to_string(), name, is_dir, size));
    }
    out.sort_by(|a, b| match (a.2, b.2) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
    });
    Ok(out)
}

/// Resultado de un intento de borrado recursivo robusto.
pub struct DeleteResult {
    pub bytes_freed: u64,
    pub files_deleted: u64,
    pub errors: Vec<String>,
    pub pending_reboot: Vec<String>,
}

/// Borrado recursivo robusto.
///
/// 1. Recorre recursivamente todos los subdirectorios.
/// 2. Intenta eliminar cada archivo individualmente, capturando errores.
/// 3. Maneja archivos en uso:
///    - Primero: intenta borrar normalmente.
///    - Si falla: marca para borrado en el próximo reboot (pending_reboot).
/// 4. Elimina directorios vacíos tras borrar su contenido (de dentro hacia fuera).
/// 5. No falla silenciosamente: cada error se registra.
pub fn delete_recursive_robust(root: &Path) -> DeleteResult {
    let mut result = DeleteResult {
        bytes_freed: 0,
        files_deleted: 0,
        errors: Vec::new(),
        pending_reboot: Vec::new(),
    };

    if !root.exists() {
        result.errors.push(format!("{}: path does not exist", root.display()));
        return result;
    }

    // Contar antes de borrar.
    if let Ok((b, f, _)) = directory_stats(root, false) {
        result.bytes_freed = b;
        result.files_deleted = f;
    }

    // Recopilar todos los archivos primero (hojas primero).
    let mut all_entries: Vec<(std::path::PathBuf, bool)> = Vec::new();
    collect_entries(root, &mut all_entries, &mut result);

    // Borrar archivos primero, luego directorios (orden inverso = de dentro hacia fuera).
    for (entry_path, is_dir) in all_entries.into_iter().rev() {
        if is_dir {
            match std::fs::remove_dir(&entry_path) {
                Ok(()) => {}
                Err(e) => {
                    // Si el directorio no está vacío o está en uso, lo reportamos.
                    if e.raw_os_error() == Some(145) || e.raw_os_error() == Some(32) {
                        result.pending_reboot.push(entry_path.to_string_lossy().to_string());
                    } else {
                        result.errors.push(format!("{}: {}", entry_path.display(), e));
                    }
                }
            }
        } else {
            match std::fs::remove_file(&entry_path) {
                Ok(()) => {}
                Err(e) => {
                    // Error 32 = ERROR_SHARING_VIOLATION (archivo en uso)
                    // Error 5 = ERROR_ACCESS_DENIED
                    if e.raw_os_error() == Some(32) {
                        #[cfg(windows)]
                        {
                            mark_for_reboot_deletion(&entry_path, &mut result);
                        }
                        #[cfg(not(windows))]
                        {
                            result.pending_reboot.push(entry_path.to_string_lossy().to_string());
                        }
                    } else {
                        result.errors.push(format!("{}: {}", entry_path.display(), e));
                    }
                }
            }
        }
    }

    // Intentar borrar el root mismo.
    if root.is_dir() {
        if let Err(e) = std::fs::remove_dir(root) {
            if e.raw_os_error() != Some(145) {
                result.errors.push(format!("root {}: {}", root.display(), e));
            }
        }
    } else {
        if let Err(e) = std::fs::remove_file(root) {
            result.errors.push(format!("root {}: {}", root.display(), e));
        }
    }

    result
}

fn collect_entries(
    root: &Path,
    entries: &mut Vec<(std::path::PathBuf, bool)>,
    result: &mut DeleteResult,
) {
    let rd = match std::fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            result.errors.push(format!("{}: {}", root.display(), e));
            return;
        }
    };

    for entry in rd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                result.errors.push(format!("{}: {}", root.display(), e));
                continue;
            }
        };

        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                result.errors.push(format!("{}: {}", path.display(), e));
                continue;
            }
        };

        if ft.is_dir() {
            collect_entries(&path, entries, result);
            entries.push((path, true));
        } else {
            entries.push((path, false));
        }
    }
}

#[cfg(windows)]
fn mark_for_reboot_deletion(path: &std::path::Path, result: &mut DeleteResult) {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        let res = MoveFileExW(
            windows::core::PCWSTR(wide.as_ptr()),
            None,
            MOVEFILE_DELAY_UNTIL_REBOOT,
        );
        if res.is_ok() {
            result.pending_reboot.push(path.to_string_lossy().to_string());
        } else {
            result.errors.push(format!(
                "{}: file in use, could not schedule for reboot deletion",
                path.display()
            ));
        }
    }
}

/// Legacy alias — mantiene compatibilidad con código existente.
pub fn delete_recursive(root: &Path) -> (u64, u64, Vec<String>) {
    let result = delete_recursive_robust(root);
    (result.bytes_freed, result.files_deleted, result.errors)
}

/// Devuelve (bytes, files) aplicando un filtro `keep` a cada archivo.
pub fn directory_stats_filtered<F>(
    root: &Path,
    _follow_reparse_points: bool,
    mut keep: F,
) -> AppResult<(u64, u64)>
where
    F: FnMut(&Path, &std::fs::Metadata) -> bool,
{
    if !root.exists() {
        return Ok((0, 0));
    }

    let mut bytes: u64 = 0;
    let mut files: u64 = 0;

    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        let ft = entry.file_type();
        if ft.is_file() {
            if let Ok(meta) = entry.metadata() {
                if keep(entry.path(), &meta) {
                    files += 1;
                    bytes = bytes.saturating_add(meta.len());
                }
            }
        }
    }
    Ok((bytes, files))
}

/// Borrado recursivo con filtro. Respeta la misma lógica que delete_recursive_robust
/// pero solo borra archivos que pasan el filtro `keep`.
pub fn delete_recursive_filtered<F>(root: &Path, mut keep: F) -> DeleteResult
where
    F: FnMut(&Path, &std::fs::Metadata) -> bool,
{
    let mut result = DeleteResult {
        bytes_freed: 0,
        files_deleted: 0,
        errors: Vec::new(),
        pending_reboot: Vec::new(),
    };

    if !root.exists() {
        result
            .errors
            .push(format!("{}: path does not exist", root.display()));
        return result;
    }

    let mut all_entries: Vec<(std::path::PathBuf, bool)> = Vec::new();
    collect_entries_filtered(root, &mut all_entries, &mut result, &mut keep);

    for (entry_path, is_dir) in all_entries.into_iter().rev() {
        if is_dir {
            match std::fs::remove_dir(&entry_path) {
                Ok(()) => {}
                Err(e) => {
                    if e.raw_os_error() == Some(145) || e.raw_os_error() == Some(32) {
                        result
                            .pending_reboot
                            .push(entry_path.to_string_lossy().to_string());
                    }
                }
            }
        } else {
            match std::fs::remove_file(&entry_path) {
                Ok(()) => {
                    if let Ok(meta) = entry_path.metadata() {
                        result.bytes_freed = result.bytes_freed.saturating_add(meta.len());
                    }
                    result.files_deleted += 1;
                }
                Err(e) => {
                    if e.raw_os_error() == Some(32) {
                        #[cfg(windows)]
                        {
                            mark_for_reboot_deletion(&entry_path, &mut result);
                        }
                        #[cfg(not(windows))]
                        {
                            result
                                .pending_reboot
                                .push(entry_path.to_string_lossy().to_string());
                        }
                    } else {
                        result
                            .errors
                            .push(format!("{}: {}", entry_path.display(), e));
                    }
                }
            }
        }
    }

    if root.is_dir() {
        if let Err(e) = std::fs::remove_dir(root) {
            if e.raw_os_error() != Some(145) {
                result
                    .errors
                    .push(format!("root {}: {}", root.display(), e));
            }
        }
    } else {
        if let Err(e) = std::fs::remove_file(root) {
            result
                .errors
                .push(format!("root {}: {}", root.display(), e));
        }
    }

    result
}

fn collect_entries_filtered<F>(
    root: &Path,
    entries: &mut Vec<(std::path::PathBuf, bool)>,
    result: &mut DeleteResult,
    keep: &mut F,
) where
    F: FnMut(&Path, &std::fs::Metadata) -> bool,
{
    let rd = match std::fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            result.errors.push(format!("{}: {}", root.display(), e));
            return;
        }
    };

    for entry in rd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                result.errors.push(format!("{}: {}", root.display(), e));
                continue;
            }
        };

        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                result.errors.push(format!("{}: {}", path.display(), e));
                continue;
            }
        };

        if ft.is_dir() {
            collect_entries_filtered(&path, entries, result, keep);
            entries.push((path, true));
        } else if let Ok(meta) = entry.metadata() {
            if keep(&path, &meta) {
                entries.push((path, false));
            }
        }
    }
}
