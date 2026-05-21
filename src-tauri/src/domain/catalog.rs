// domain/catalog.rs — carga de catálogos JSON (allowlists).
//
// Los catálogos se embeben en el binario con `include_str!` en tiempo de
// compilación. Cero dependencias de filesystem en runtime.
//
// Filosofía de errores:
//   - Si el JSON falla al deserializar → AppError::Catalog con mensaje claro.
//   - NUNCA devolver `[]` silencioso.

use crate::core::{AppError, AppResult};
use crate::models::cache::{CacheCatalogFile, CacheLocation};
use crate::models::debloat::BloatwareCatalogFile;

const CACHE_LOCATIONS_JSON: &str = include_str!(
    "../../../.claude/skills/cache-scanner/RESOURCES/cache-locations.json"
);
const BLOATWARE_CATALOG_JSON: &str = include_str!(
    "../../../.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json"
);

pub fn load_cache_locations() -> AppResult<Vec<CacheLocation>> {
    let envelope: CacheCatalogFile = serde_json::from_str(CACHE_LOCATIONS_JSON)
        .map_err(|e| AppError::Catalog(format!("cache-locations.json: {e}")))?;
    Ok(envelope.entries)
}

pub fn load_bloatware_catalog() -> AppResult<Vec<crate::models::debloat::BloatwareEntry>> {
    let envelope: BloatwareCatalogFile = serde_json::from_str(BLOATWARE_CATALOG_JSON)
        .map_err(|e| AppError::Catalog(format!("bloatware-catalog.json: {e}")))?;
    Ok(envelope.entries)
}

/// Valida que todos los catálogos embebidos sean parseables.
/// Se invoca una vez al arranque en `lib.rs::setup`.
pub fn validate_all() -> AppResult<()> {
    load_cache_locations()?;
    load_bloatware_catalog()?;
    Ok(())
}
