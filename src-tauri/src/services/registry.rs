// services/registry.rs — lectura/escritura segura de HKLM y HKCU.
//
// Toda escritura pasa por la allowlist `registry-tweaks.json` (hives + path
// prefixes). Cualquier intento fuera de allowlist → AppError::Permission.
// Diseño en .claude/specs/04-modules/registry-tweaks.md.
//
// Implementación pendiente (usará `winreg` y/o `windows-rs` Win32_System_Registry).
