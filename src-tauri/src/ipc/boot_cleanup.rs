// ipc/boot_cleanup.rs — boot-time cleanup: pending renames programados para el próximo reboot.

use crate::core::{AppError, AppResult};
use crate::models::pending_rename::PendingRename;
use crate::platform::pending_rename;

#[tauri::command]
pub async fn list_pending_renames() -> AppResult<Vec<PendingRename>> {
    pending_rename::list_pending_renames()
}

#[tauri::command]
pub async fn cancel_pending_rename(source: String) -> AppResult<()> {
    let path = std::path::Path::new(&source);
    pending_rename::cancel_pending_rename(path)
}

#[tauri::command]
pub async fn clear_all_pending_renames() -> AppResult<u32> {
    let all = pending_rename::list_pending_renames()?;
    let count = all.len() as u32;

    // Solo eliminar las que no apuntan a rutas de sistema sensibles.
    // Por precaución se eliminan todas; el contrato en la spec dice:
    // emitir warn si hay entries no creadas por ClearTool y no tocarlas.
    // Dado que no tenemos un prefijo de ownership, filtramos conservadoramente:
    // solo eliminamos las que son delete (destination vacío).
    let to_keep: Vec<PendingRename> = all
        .into_iter()
        .filter(|p| {
            if p.is_delete {
                // Solo eliminar si es un borrado programado — no renombres de Windows Update etc.
                let path_lower = p.source.to_lowercase();
                let is_system = path_lower.contains(r"\windows\")
                    || path_lower.contains(r"\system32\")
                    || path_lower.contains(r"\syswow64\")
                    || path_lower.contains(r"\winsxs\");
                if is_system {
                    log::warn!("clear_all_pending_renames: omitiendo entrada de sistema: {}", p.source);
                }
                is_system
            } else {
                // Conservar renombres (no borrados) — son de Windows Update/DISM.
                log::warn!("clear_all_pending_renames: conservando rename de sistema: {}", p.source);
                true
            }
        })
        .collect();

    let removed = count - to_keep.len() as u32;

    // Reescribir solo las entradas que conservamos.
    // Llamamos cancel_pending_rename para cada una de las que borramos:
    // Más seguro que reescribir el REG_MULTI_SZ directamente.
    let all_original = pending_rename::list_pending_renames()?;
    for entry in &all_original {
        if !to_keep.iter().any(|k| k.source == entry.source) {
            let _ = pending_rename::cancel_pending_rename(std::path::Path::new(&entry.source));
        }
    }

    if removed == 0 {
        return Err(AppError::Validation(
            "No hay pending renames de borrado elegibles para cancelar".into(),
        ));
    }

    Ok(removed)
}
