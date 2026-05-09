// AppError — tipo de error unificado para todo el backend.
//
// Stub provisional: la implementación final usa `thiserror` (ver
// .claude/specs/02-backend-rust.md) pero para no añadir la dependencia hasta
// que la necesite algún módulo, mantengo un enum manual con `Display` y
// `Serialize` artesanales. Cuando metamos thiserror, se reescribe en una
// pasada — la API pública (variantes y `kind`) ya queda fijada aquí.

use serde::{Serialize, Serializer};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Registry(String),
    Powershell(String),
    Permission(String),
    NotElevated,
    Cancelled,
    RestoreUnavailable(String),
    External(String),
    NotImplemented,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "io: {}", e),
            AppError::Registry(m) => write!(f, "registry: {}", m),
            AppError::Powershell(m) => write!(f, "powershell: {}", m),
            AppError::Permission(m) => write!(f, "permission: {}", m),
            AppError::NotElevated => write!(f, "not elevated"),
            AppError::Cancelled => write!(f, "cancelled"),
            AppError::RestoreUnavailable(m) => write!(f, "restore unavailable: {}", m),
            AppError::External(m) => write!(f, "external: {}", m),
            AppError::NotImplemented => write!(f, "not implemented"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            AppError::Io(_) => "io",
            AppError::Registry(_) => "registry",
            AppError::Powershell(_) => "powershell",
            AppError::Permission(_) => "permission",
            AppError::NotElevated => "not-elevated",
            AppError::Cancelled => "cancelled",
            AppError::RestoreUnavailable(_) => "restore-unavailable",
            AppError::External(_) => "external",
            AppError::NotImplemented => "not-implemented",
        };
        let mut map = serde_json::Map::new();
        map.insert("kind".into(), kind.into());
        map.insert("message".into(), self.to_string().into());
        serde_json::Value::Object(map).serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;
