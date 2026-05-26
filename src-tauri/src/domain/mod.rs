// Capa `domain` — lógica de negocio del backend.
//
// `domain` orquesta `platform` y aplica reglas: allowlists, dry-run,
// validación, restore points previos, escritura de audit log. NO conoce
// Tauri (eso lo hace `ipc`) ni el FFI (lo hace `platform`).

pub mod audit;
pub mod cache;
pub mod catalog;
pub mod debloat;
pub mod disk;
pub mod explorer;
pub mod registry;
pub mod restore;
pub mod services;
pub mod startup;
pub mod system_info;
pub mod telemetry;
