// Capa de modelos — DTOs serializables compartidos entre backend y frontend.
//
// Convenciones:
//   - Todos los structs derivan `Serialize` y `Deserialize`.
//   - `serde(rename_all = "camelCase")` para encajar con TS sin transformaciones.
//   - Cuando se incorpore `ts-rs`, se añade `#[ts(export)]` para emitir bindings.

pub mod cache;
pub mod debloat;
pub mod registry;
pub mod restore;
pub mod service;
pub mod system;
pub mod tree;
