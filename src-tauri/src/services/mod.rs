// Capa de servicios — lógica de negocio del backend.
//
// Los `commands` de Tauri son envoltorios delgados; toda la lógica real
// (acceso a FS, registro, PowerShell, audit log, restore points, carga
// de catálogos) vive aquí, sin saber nada de Tauri ni de IPC.

pub mod audit_log;
pub mod catalog;
pub mod filesystem;
pub mod powershell;
pub mod registry;
pub mod restore_point;
pub mod service_manager;
