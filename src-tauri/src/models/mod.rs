// Capa de modelos — DTOs serializables compartidos entre backend y frontend.
//
// Convenciones:
//   - Todos los structs derivan `Serialize` y `Deserialize`.
//   - `serde(rename_all = "camelCase")` para encajar con TS sin transformaciones.
//   - Cuando se incorpore `ts-rs`, se añade `#[ts(export)]` para emitir bindings.

pub mod cache;
pub mod debloat;
pub mod disk;
pub mod inventory;
pub mod pending_rename;
pub mod privacy;
pub mod process;
pub mod registry;
pub mod restore;
pub mod service;
pub mod settings;
pub mod startup;
pub mod system;
pub mod telemetry;
pub mod tree;
