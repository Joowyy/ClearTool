// services/catalog.rs — carga y validación de los catálogos JSON.
//
// Catálogos esperados (allowlists):
//   - bloatware-catalog.json    (Appx packages + uninstaller IDs)
//   - cache-locations.json       (rutas FS conocidas a limpiar)
//   - services-catalog.json      (servicios Windows manipulables)
//   - registry-tweaks.json       (hives + path prefixes permitidos)
//
// Validación contra `*.schema.json` al cargar. Cualquier fallo de schema
// → la app arranca pero el módulo afectado queda deshabilitado (degradación
// gracil, no panic).
//
// Implementación pendiente.
