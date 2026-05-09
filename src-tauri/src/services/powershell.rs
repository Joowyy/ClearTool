// services/powershell.rs — invocación segura de scripts PowerShell embebidos.
//
// Reglas duras (ver .claude/specs/05-security.md):
//   - Scripts cargados con `include_str!`, NUNCA construidos en runtime.
//   - Argumentos validados con regex en Rust antes de pasar a PS.
//   - Captura estructurada de stdout/stderr/exit_code.
//
// Implementación pendiente.
