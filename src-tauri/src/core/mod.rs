// Capa `core` — fundamentos transversales del backend.
//
// Aquí vive lo que el resto de capas asume disponible:
//   - `error`  : tipo unificado `AppError` y alias `AppResult<T>`.
//   - `config` : paths estándar de la app (%APPDATA%\ClearTool, etc.)
//                y constantes globales.
//
// `core` NO depende de `platform`, `domain` ni `ipc`. Es el "qué siempre es cierto".

pub mod config;
pub mod error;
pub mod settings;

pub use error::{AppError, AppResult};
