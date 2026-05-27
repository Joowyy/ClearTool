// platform/powershell.rs — invocación segura de PowerShell embebido.

use std::process::{Command, Output};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use crate::core::{AppError, AppResult};
use std::time::Duration;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn run_script(script: &'static str) -> AppResult<Output> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-Command", script,
        ]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().map_err(|e| AppError::Powershell(e.to_string()))
    }

    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("pwsh");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-Command", script,
        ]);
        cmd.output().map_err(|e| {
            AppError::Powershell(format!(
                "PowerShell no disponible: {}",
                e
            ))
        })
    }
}

/// Variante que acepta `&str` no `&'static str` — solo para scripts seguros
/// parametrizados por enteros validados (ej: sequence_number de restore point).
pub fn run_script_owned(script: &str) -> AppResult<Output> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-Command", script,
        ]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().map_err(|e| AppError::Powershell(e.to_string()))
    }

    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("pwsh");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-Command", script,
        ]);
        cmd.output().map_err(|e| {
            AppError::Powershell(format!("PowerShell no disponible: {}", e))
        })
    }
}

pub async fn run_script_with_timeout(script: &'static str, timeout: Duration) -> AppResult<Output> {
    let secs = timeout.as_secs();
    tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(move || run_script(script)),
    )
    .await
    .map_err(|_| AppError::Powershell(format!("timeout {}s", secs)))?
    .map_err(|e| AppError::External(e.to_string()))?
}
