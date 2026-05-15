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
    #[error("restore unavailable: {0}")]
    RestoreUnavailable(String),
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
            AppError::RestoreUnavailable(_) => "restore-unavailable",
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
