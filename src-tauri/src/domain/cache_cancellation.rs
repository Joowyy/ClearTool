// domain/cache_cancellation.rs — registro global de tokens de cancelación.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

static TOKENS: Lazy<Mutex<HashMap<String, CancellationToken>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Registra un token para `run_id`. Devuelve el token recién creado.
pub fn register(run_id: &str) -> CancellationToken {
    let token = CancellationToken::new();
    if let Ok(mut map) = TOKENS.lock() {
        map.insert(run_id.to_string(), token.clone());
    }
    token
}

/// Cancela el run activo con ese `run_id`. Devuelve `true` si se encontró y canceló.
pub fn cancel(run_id: &str) -> bool {
    if let Ok(map) = TOKENS.lock() {
        if let Some(token) = map.get(run_id) {
            token.cancel();
            return true;
        }
    }
    false
}

/// Elimina el token del registro (llamar al finalizar el run).
pub fn deregister(run_id: &str) {
    if let Ok(mut map) = TOKENS.lock() {
        map.remove(run_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_registry_returns_false_for_unknown_runid() {
        assert!(!cancel("nonexistent-run-id-xyz"));
    }

    #[test]
    fn cancellation_registry_cancels_active_token() {
        let run_id = "test-run-cancel-123";
        let token = register(run_id);
        assert!(!token.is_cancelled());
        assert!(cancel(run_id));
        assert!(token.is_cancelled());
        deregister(run_id);
    }
}
