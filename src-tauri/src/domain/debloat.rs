// domain/debloat.rs — orquestación de eliminación de bloatware.

use crate::core::{AppError, AppResult};
use crate::domain::catalog;
use crate::models::debloat::{BloatwareEntry, DetectedPackage, RemoveBloatwareInput, RemoveReport};

pub fn list_catalog() -> AppResult<Vec<BloatwareEntry>> {
    catalog::load_bloatware_catalog()
}

pub fn detect_installed() -> AppResult<Vec<DetectedPackage>> {
    Err(AppError::NotImplemented)
}

pub fn remove(_input: &RemoveBloatwareInput) -> AppResult<RemoveReport> {
    Err(AppError::NotImplemented)
}
