// domain/restore.rs — System Restore Points orquestados.

use crate::core::AppResult;
use crate::models::restore::{CreateRestorePointInput, RestorePoint, RestoreReport};
use crate::platform;
use chrono::Utc;

pub fn ensure_enabled() -> AppResult<bool> {
    platform::restore_point::is_enabled()
}

pub fn enable_for_system_drive() -> AppResult<()> {
    platform::restore_point::enable_for_system_drive()
}

pub fn create(input: &CreateRestorePointInput) -> AppResult<RestoreReport> {
    let restore_type = input.restore_type.unwrap_or(12); // MODIFY_SETTINGS
    let seq = platform::restore_point::create(
        &input.description,
        restore_type,
        input.bypass_throttle,
    )?;
    Ok(RestoreReport {
        sequence_number: seq,
        created_at: Utc::now().to_rfc3339(),
        description: input.description.clone(),
        bypassed_throttle: input.bypass_throttle,
    })
}

pub fn list() -> AppResult<Vec<RestorePoint>> {
    platform::restore_point::list()
}

pub fn restore_to(sequence_number: u32) -> AppResult<()> {
    platform::restore_point::restore_to(sequence_number)
}

/// Helper reusable: garantiza que existe (o crea) un Restore Point antes de una
/// operación destructiva. Los demás módulos lo llaman al inicio de clean(),
/// apply(), remove(), etc.
///
/// Si System Protection está OFF, devuelve `Ok(None)` y deja que el caller decida
/// si abortar o continuar sin punto.
pub fn ensure_or_create(description: &str) -> AppResult<Option<u32>> {
    if !platform::restore_point::is_enabled()? {
        log::warn!("System Protection OFF — operación sin restore point");
        return Ok(None);
    }
    let report = create(&CreateRestorePointInput {
        description: description.to_string(),
        restore_type: Some(0), // APPLICATION_INSTALL
        bypass_throttle: true,
    })?;
    Ok(Some(report.sequence_number))
}

/// Crea un restore point de forma simple. Si System Protection está OFF,
/// loggea un warning pero no falla (operaciones de red pueden continuar).
pub fn create_restore_point(description: &str) -> AppResult<()> {
    if !platform::restore_point::is_enabled()? {
        log::warn!("System Protection OFF — create_restore_point skipped: {}", description);
        return Ok(());
    }
    let _ = create(&CreateRestorePointInput {
        description: description.to_string(),
        restore_type: Some(0),
        bypass_throttle: true,
    })?;
    Ok(())
}
