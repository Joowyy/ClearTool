// domain/catalog.rs — carga de catálogos JSON (allowlists).

use crate::core::{AppError, AppResult};
use crate::models::cache::{CacheCatalogFile, CacheLocation};
use crate::models::debloat::{BloatwareCatalogFile, BloatwareEntry};
use crate::models::registry::{RegistryCatalogFile, RegistryTweak};
use crate::models::service::{ServiceEntry, ServicesCatalogFile};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

const CACHE_LOCATIONS_JSON: &str = include_str!(
    "../../../.claude/skills/cache-scanner/RESOURCES/cache-locations.json"
);
const BLOATWARE_CATALOG_JSON: &str = include_str!(
    "../../../.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json"
);
const SERVICES_CATALOG_JSON: &str = include_str!(
    "../../../.claude/skills/powershell-debloat/RESOURCES/services-catalog.json"
);
const REGISTRY_TWEAKS_JSON: &str = include_str!(
    "../../../.claude/skills/windows-registry-ops/RESOURCES/registry-tweaks.json"
);

static CACHE_LOCATIONS: OnceLock<Mutex<Option<Vec<CacheLocation>>>> = OnceLock::new();
static BLOATWARE_CATALOG: OnceLock<Mutex<Option<Vec<BloatwareEntry>>>> = OnceLock::new();
static SERVICES_CATALOG: OnceLock<Mutex<Option<Vec<ServiceEntry>>>> = OnceLock::new();
static REGISTRY_TWEAKS: OnceLock<Mutex<Option<Vec<RegistryTweak>>>> = OnceLock::new();

static ALLOWED_CACHE_IDS: OnceLock<HashSet<String>> = OnceLock::new();
static ALLOWED_SERVICE_NAMES: OnceLock<HashSet<String>> = OnceLock::new();
static ALLOWED_REGISTRY_IDS: OnceLock<HashSet<String>> = OnceLock::new();

pub fn load_cache_locations() -> AppResult<Vec<CacheLocation>> {
    let lock = CACHE_LOCATIONS.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().map_err(|_| AppError::Catalog("lock poisoned".into()))?;
    if let Some(ref entries) = *guard {
        return Ok(entries.clone());
    }
    let envelope: CacheCatalogFile = serde_json::from_str(CACHE_LOCATIONS_JSON)
        .map_err(|e| AppError::Catalog(format!("cache-locations.json: {e}")))?;
    let entries = envelope.entries;
    *guard = Some(entries.clone());
    Ok(entries)
}

pub fn load_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    let lock = BLOATWARE_CATALOG.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().map_err(|_| AppError::Catalog("lock poisoned".into()))?;
    if let Some(ref entries) = *guard {
        return Ok(entries.clone());
    }
    let envelope: BloatwareCatalogFile = serde_json::from_str(BLOATWARE_CATALOG_JSON)
        .map_err(|e| AppError::Catalog(format!("bloatware-catalog.json: {e}")))?;
    let entries = envelope.entries;
    *guard = Some(entries.clone());
    Ok(entries)
}

pub fn load_services_catalog() -> AppResult<Vec<ServiceEntry>> {
    let lock = SERVICES_CATALOG.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().map_err(|_| AppError::Catalog("lock poisoned".into()))?;
    if let Some(ref entries) = *guard {
        return Ok(entries.clone());
    }
    let envelope: ServicesCatalogFile = serde_json::from_str(SERVICES_CATALOG_JSON)
        .map_err(|e| AppError::Catalog(format!("services-catalog.json: {e}")))?;
    let entries = envelope.entries;
    *guard = Some(entries.clone());
    Ok(entries)
}

pub fn load_registry_tweaks() -> AppResult<Vec<RegistryTweak>> {
    let lock = REGISTRY_TWEAKS.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().map_err(|_| AppError::Catalog("lock poisoned".into()))?;
    if let Some(ref entries) = *guard {
        return Ok(entries.clone());
    }
    let envelope: RegistryCatalogFile = serde_json::from_str(REGISTRY_TWEAKS_JSON)
        .map_err(|e| AppError::Catalog(format!("registry-tweaks.json: {e}")))?;
    let entries = envelope.entries;
    *guard = Some(entries.clone());
    Ok(entries)
}

pub fn is_cache_id_allowed(id: &str) -> bool {
    let allowed = ALLOWED_CACHE_IDS.get_or_init(|| {
        load_cache_locations()
            .map(|locs| locs.iter().map(|l| l.id.clone()).collect())
            .unwrap_or_default()
    });
    allowed.contains(id)
}

pub fn is_service_allowed(name: &str) -> bool {
    let allowed = ALLOWED_SERVICE_NAMES.get_or_init(|| {
        load_services_catalog()
            .map(|entries| entries.iter().map(|e| e.service_name.clone()).collect())
            .unwrap_or_default()
    });
    allowed.contains(name)
}

pub fn is_registry_tweak_allowed(id: &str) -> bool {
    let allowed = ALLOWED_REGISTRY_IDS.get_or_init(|| {
        load_registry_tweaks()
            .map(|tweaks| tweaks.iter().map(|t| t.id.clone()).collect())
            .unwrap_or_default()
    });
    allowed.contains(id)
}

pub fn validate_all() -> AppResult<()> {
    load_cache_locations()?;
    load_bloatware_catalog()?;
    load_services_catalog()?;
    load_registry_tweaks()?;

    let _ = is_cache_id_allowed("");
    let _ = is_service_allowed("");
    let _ = is_registry_tweak_allowed("");

    Ok(())
}
