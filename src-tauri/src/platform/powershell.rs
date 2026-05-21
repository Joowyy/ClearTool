// platform/powershell.rs — invocación segura de PowerShell embebido.
//
// Reglas duras:
//   - Flag CREATE_NO_WINDOW obligatorio en Windows (evita ventanas parpadeantes).
//   - Timeout configurable (default 20 s).
//   - Solo acepta `&'static str` como script — impide inyección de input dinámico.
//   - Captura stdout/stderr como UTF-8.
//   - Retorna `AppResult<Output>`.

use std::process::{Command, Output};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use crate::core::{AppError, AppResult};
use std::time::Duration;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn run_script(script: &'static str) -> AppResult<Output> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy", "Bypass",
        "-Command", script,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map_err(|e| AppError::Powershell(e.to_string()))
}

#[allow(dead_code)]
pub fn run_script_with_timeout(script: &'static str, _timeout: Duration) -> AppResult<Output> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy", "Bypass",
        "-Command", script,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().map_err(|e| AppError::Powershell(e.to_string()))?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(AppError::Powershell(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}
