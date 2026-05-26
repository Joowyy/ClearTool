// core/error.rs — tipo de error unificado para todo el backend.
//
// Reglas:
//   - Toda función pública devuelve `AppResult<T>` salvo casos triviales.
//   - El `Serialize` manual proyecta a `{kind, message}` para el frontend,
//     que lo consume como discriminated union en TS.

use serde::{ser::SerializeMap, Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("registry: {0}")]
    Registry(String),
    #[error("powershell: {0}")]
    Powershell(String),
    #[error("permission: {0}")]
    Permission(String),
    #[error("not elevated")]
    NotElevated,
    #[error("cancelled")]
    Cancelled,
    #[error("restore point: {0}")]
    RestorePoint(String),
    #[error("restore unavailable: {0}")]
    RestoreUnavailable(String),
    #[error("services: {0}")]
    Services(String),
    #[error("audit: {0}")]
    Audit(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("external: {0}")]
    External(String),
    #[error("not implemented")]
    NotImplemented,
    #[error("catalog: {0}")]
    Catalog(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
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
            AppError::RestorePoint(_) => "restore-point",
            AppError::RestoreUnavailable(_) => "restore-unavailable",
            AppError::Services(_) => "services",
            AppError::Audit(_) => "audit",
            AppError::Validation(_) => "validation",
            AppError::Parse(_) => "parse",
            AppError::External(_) => "external",
            AppError::NotImplemented => "not-implemented",
            AppError::Catalog(_) => "catalog",
            AppError::Json(_) => "json",
        };
        let mut map = s.serialize_map(Some(2))?;
        map.serialize_entry("kind", kind)?;
        map.serialize_entry("message", &self.to_string())?;
        map.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Añade contexto a un error existente, preservando el tipo original.
    ///
    /// # Ejemplo
    /// ```ignore
    /// read_catalog().map_err(|e| e.with_context("al cargar catálogo de bloatware"))
    /// ```
    pub fn with_context(self, context: &str) -> Self {
        match self {
            AppError::Io(msg) => AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{context}: {msg}"),
            )),
            AppError::Registry(msg) => AppError::Registry(format!("{context}: {msg}")),
            AppError::Powershell(msg) => AppError::Powershell(format!("{context}: {msg}")),
            AppError::Permission(msg) => AppError::Permission(format!("{context}: {msg}")),
            AppError::NotElevated => AppError::NotElevated,
            AppError::Cancelled => AppError::Cancelled,
            AppError::RestorePoint(msg) => AppError::RestorePoint(format!("{context}: {msg}")),
            AppError::RestoreUnavailable(msg) => AppError::RestoreUnavailable(format!("{context}: {msg}")),
            AppError::Services(msg) => AppError::Services(format!("{context}: {msg}")),
            AppError::Audit(msg) => AppError::Audit(format!("{context}: {msg}")),
            AppError::Validation(msg) => AppError::Validation(format!("{context}: {msg}")),
            AppError::Parse(msg) => AppError::Parse(format!("{context}: {msg}")),
            AppError::External(msg) => AppError::External(format!("{context}: {msg}")),
            AppError::NotImplemented => AppError::NotImplemented,
            AppError::Catalog(msg) => AppError::Catalog(format!("{context}: {msg}")),
            AppError::Json(msg) => AppError::Json(msg),
        }
    }
}
