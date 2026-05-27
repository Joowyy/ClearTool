// domain/cache_background.rs — análisis perezoso del catálogo de cachés en background.
//
// Se dispara al arrancar la app (via spawn_initial_scan en setup) y guarda el
// CleanPlan resultante en una RwLock global. El frontend puede obtenerlo
// inmediatamente vía `get_cache_plan_warm`, evitando que el usuario tenga que
// pulsar "Analizar" antes de ver resultados.

use crate::core::{AppError, AppResult};
use crate::models::cache::CleanPlan;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CachedPlan {
    pub plan: CleanPlan,
    pub computed_at: Instant,
    pub all_ids: Vec<String>,
}

static CACHED: Lazy<RwLock<Option<CachedPlan>>> = Lazy::new(|| RwLock::new(None));

/// El plan se considera obsoleto tras este tiempo y se recalcula en el siguiente request.
const TTL: Duration = Duration::from_secs(90);

/// Lanza el análisis inicial en background al arrancar la app.
/// No bloquea el hilo de UI ni el splash de Tauri.
///
/// Usa `tauri::async_runtime::spawn` (no `tokio::spawn` directo) porque
/// `setup()` corre antes de que el runtime Tokio quede registrado como
/// "current" para el thread principal — `tokio::spawn` panica ahí con
/// "there is no reactor running". Tauri provee su propio handle que sí
/// funciona desde setup.
pub fn spawn_initial_scan() {
    tauri::async_runtime::spawn(async move {
        log::info!("cache_background: iniciando primer análisis perezoso");
        if let Err(e) = recompute_and_store().await {
            log::warn!("cache_background: primer análisis falló: {}", e);
        }
    });
}

/// Devuelve el plan cacheado si existe y está dentro del TTL.
pub fn get_cached() -> Option<CachedPlan> {
    let guard = CACHED.read().ok()?;
    let cached = guard.as_ref()?.clone();
    if cached.computed_at.elapsed() > TTL {
        return None;
    }
    Some(cached)
}

/// Devuelve el plan cacheado independientemente del TTL (para mostrar datos
/// aunque sean algo viejos mientras se recalcula en background).
pub fn get_cached_stale_ok() -> Option<CachedPlan> {
    CACHED.read().ok()?.as_ref().cloned()
}

/// Recalcula el plan con todos los IDs del catálogo y lo almacena en cache.
/// Usa spawn_blocking para no bloquear el executor de tokio con I/O síncrono.
pub async fn recompute_and_store() -> AppResult<CleanPlan> {
    let (plan, ids) = tokio::task::spawn_blocking(|| -> AppResult<(CleanPlan, Vec<String>)> {
        let catalog = crate::domain::catalog::load_cache_locations()?;
        let ids: Vec<String> = catalog.iter().map(|l| l.id.clone()).collect();
        let plan = crate::domain::cache::analyze_locations(&ids)?;
        Ok((plan, ids))
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::other(format!("join error en cache_background: {}", e))))??;

    let cached = CachedPlan {
        plan: plan.clone(),
        computed_at: Instant::now(),
        all_ids: ids,
    };
    if let Ok(mut guard) = CACHED.write() {
        *guard = Some(cached);
    }
    Ok(plan)
}

/// Invalida el plan cacheado. Llamar tras cualquier limpieza real (no dry-run).
pub fn invalidate() {
    if let Ok(mut guard) = CACHED.write() {
        *guard = None;
    }
}
