// Capa `ipc` — superficie expuesta al frontend vía Tauri.
//
// Cada submódulo contiene `#[tauri::command]` que son envoltorios delgados:
// validan inputs, delegan a `domain`, traducen `AppError` a la representación
// que el frontend consume. No hay lógica de negocio aquí.

pub mod audit;
pub mod boot_cleanup;
pub mod cache;
pub mod debloat;
pub mod diagnostics;
pub mod disk;
pub mod inventory;
pub mod network;
pub mod privacy;
pub mod processes;
pub mod registry;
pub mod restore;
pub mod services;
pub mod settings;
pub mod startup;
pub mod system_info;
pub mod telemetry;
