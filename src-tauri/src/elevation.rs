// Helpers de elevación de privilegios.
//
// La implementación real usa `windows-rs` con las features
// Win32_Security y Win32_System_Threading para llamar a
// OpenProcessToken + GetTokenInformation con TOKEN_ELEVATION
// (ver .claude/specs/02-backend-rust.md). Mientras no se añade la
// dependencia, este stub devuelve `false` para no bloquear la compilación.
//
// TODO(elevation): sustituir por la versión Win32 cuando se incorpore
// `windows = "0.58"` con features Win32_Foundation/Security/Threading.

pub fn is_elevated() -> bool {
    false
}
