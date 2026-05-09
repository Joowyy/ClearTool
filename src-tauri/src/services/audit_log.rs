// services/audit_log.rs — escritura/lectura de %APPDATA%\ClearTool\audit.jsonl.
//
// Cada operación destructiva escribe un `AuditEntry` con `reverse_recipe`
// que permite la reversa fina (un solo tweak, un solo servicio). La reversa
// total se hace contra restore points. Diseño en .claude/specs/05-security.md.
//
// Implementación pendiente (escritura append-only, formato JSONL).
