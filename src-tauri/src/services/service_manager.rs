// services/service_manager.rs — gestión de servicios de Windows (SCM).
//
// Allowlist obligatoria desde `services-catalog.json`. Operaciones soportadas:
// query state, change start type, start/stop. Diseño en
// .claude/specs/04-modules/service-manager.md.
//
// Implementación pendiente (usará `windows-service` y/o Win32_System_Services).
