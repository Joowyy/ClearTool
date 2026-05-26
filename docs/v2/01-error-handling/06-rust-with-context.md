# Paso 06 — Backend: `AppError::with_context` para enriquecer mensajes

**Área**: 01-error-handling
**Tiempo estimado**: 2 horas
**Dependencias**: ninguna (independiente del trabajo frontend)

## Qué hacemos

Añadir un helper `with_context` a `AppError` que permita aumentar el mensaje con el contexto de la operación. Ej: en vez de "Access denied (os error 5)" devolver "eliminando C:\foo\bar: Access denied (os error 5)".

## Por qué

Hoy los errores Rust llegan al frontend con mensajes técnicos sin contexto:
- `"file in use, could not schedule for reboot deletion"` ← ¿qué archivo?
- `"Access denied. (os error 5)"` ← ¿en qué operación?

Con `with_context`, el mensaje siempre dice **qué se intentaba hacer** + **qué pasó**. Hace muchísimo más legibles los toasts.

## Archivos que tocamos

- `src-tauri/src/core/error.rs` (modificado)
- Múltiples sitios donde se generan errores I/O y de PowerShell (modificado)

## Cómo

### 1. Añadir el método `with_context`

Edita `src-tauri/src/core/error.rs`. Busca la definición de `AppError`:

```rust
// src-tauri/src/core/error.rs

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "message", rename_all = "kebab-case")]
pub enum AppError {
    #[error("io: {0}")]
    Io(String),
    #[error("registry: {0}")]
    Registry(String),
    #[error("powershell: {0}")]
    Powershell(String),
    #[error("permission: {0}")]
    Permission(String),
    #[error("not elevated: {0}")]
    NotElevated(String),
    // ... otros variantes
}

impl AppError {
    /// Añade contexto al mensaje sin cambiar el `kind`.
    ///
    /// # Ejemplo
    /// ```ignore
    /// fs::remove_file(&path)
    ///   .map_err(|e| AppError::Io(e.to_string()).with_context(format!("eliminando {}", path.display())))?;
    /// ```
    pub fn with_context<C: std::fmt::Display>(self, ctx: C) -> Self {
        match self {
            AppError::Io(msg)            => AppError::Io(format!("{}: {}", ctx, msg)),
            AppError::Registry(msg)      => AppError::Registry(format!("{}: {}", ctx, msg)),
            AppError::Powershell(msg)    => AppError::Powershell(format!("{}: {}", ctx, msg)),
            AppError::Permission(msg)    => AppError::Permission(format!("{}: {}", ctx, msg)),
            AppError::NotElevated(msg)   => AppError::NotElevated(format!("{}: {}", ctx, msg)),
            AppError::Validation(msg)    => AppError::Validation(format!("{}: {}", ctx, msg)),
            AppError::Services(msg)      => AppError::Services(format!("{}: {}", ctx, msg)),
            AppError::Parse(msg)         => AppError::Parse(format!("{}: {}", ctx, msg)),
            AppError::RestorePoint(msg)  => AppError::RestorePoint(format!("{}: {}", ctx, msg)),
            // Variantes sin payload de string se devuelven como están
            other => other,
        }
    }
}
```

Ajusta el match según los variantes reales de tu enum (puede que haya más que estos).

### 2. Trait helper para `Result<T, AppError>`

Para que la sintaxis sea más ergonómica, añade un trait extension:

```rust
// src-tauri/src/core/error.rs (al final)

pub trait AppResultContext<T> {
    /// Atajo: `result.with_context("eliminando X")?` en vez de `.map_err(|e| e.with_context(...))?`.
    fn with_context<F, C>(self, ctx_fn: F) -> Result<T, AppError>
    where
        F: FnOnce() -> C,
        C: std::fmt::Display;
}

impl<T> AppResultContext<T> for Result<T, AppError> {
    fn with_context<F, C>(self, ctx_fn: F) -> Result<T, AppError>
    where
        F: FnOnce() -> C,
        C: std::fmt::Display,
    {
        self.map_err(|e| e.with_context(ctx_fn()))
    }
}
```

Asegúrate de exportar el trait desde `core/mod.rs`:

```rust
// src-tauri/src/core/mod.rs
pub use error::{AppError, AppResult, AppResultContext};
```

### 3. Usar en los 3 sitios críticos (mínimo)

#### Cache cleaner

```rust
// src-tauri/src/domain/cache.rs (o donde esté)
use crate::core::AppResultContext;

fs::remove_file(&path)
    .map_err(|e| AppError::Io(e.to_string()))
    .with_context(|| format!("eliminando {}", path.display()))?;
```

#### Restore points

```rust
// src-tauri/src/platform/restore_point.rs
fn create_inner(description: &str, restore_type: u32) -> AppResult<u32> {
    // ...
    if !ok.as_bool() {
        return Err(AppError::RestorePoint(format!(
            "SRSetRestorePointW failed: nStatus={}",
            status.nStatus.0
        ))
        .with_context(format!("creando restore point \"{}\"", description)));
    }
    // ...
}
```

#### Debloat detect

```rust
// src-tauri/src/domain/debloat.rs
pub fn detect_installed() -> AppResult<Vec<DetectedPackage>> {
    let cat = catalog::load_bloatware_catalog()?;
    let (users, provs) = platform::debloat::list_appx()
        .with_context(|| "listando paquetes Appx instalados")?;
    // ...
}
```

### 4. Verificar que compila

```bash
cd src-tauri
cargo check 2>&1 | tail -20
```

### 5. Verificar en el frontend

Lanza la app y provoca un error donde aplicaste `with_context`. El toast debe mostrar:

ANTES: `[io] Access denied (os error 5)`
DESPUÉS: `[io] eliminando C:\Users\joels\AppData\Local\Packages\...\file.lock: Access denied (os error 5)`

## Criterio de done

- [ ] `AppError::with_context` implementado en `core/error.rs`.
- [ ] Trait `AppResultContext` exportado desde `core/mod.rs`.
- [ ] Aplicado en mínimo 3 sitios críticos (cache, restore, debloat).
- [ ] `cargo check` pasa sin warnings nuevos.
- [ ] Errores en frontend muestran contexto enriquecido visible en el toast description.
- [ ] El `kind` original del error se preserva (no es "io" antes y "validation" después por el wrap).

## Bonus: aplicar gradualmente

No tienes que tocar TODOS los sitios de Rust en este paso. Aplica los críticos (3 sitios) y deja una nota en `docs/v2/CONTRIBUTING.md` (si existe) para que los siguientes módulos lo usen como default cuando los toques.
