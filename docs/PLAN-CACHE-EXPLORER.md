# Plan — Caché y Explorador funcionales

> Plan operativo Parte 1. Documenta el bug que dejaba la UI vacía y cómo se arregla.
> Generado: 2026-05-20.

## 1. Diagnóstico final

La cadena de fallos era esta:

```
UI ───invoke("list_cache_locations")───▶ ipc::cache::list_cache_locations
                                            │
                                            ▼
                                  domain::cache::list_locations
                                            │
                                            ▼
                                  domain::catalog::load_cache_locations
                                            │
                                            ▼
                            std::fs::read_to_string(path)  → OK (sí encuentra el JSON)
                                            │
                                            ▼
                            serde_json::from_str::<Vec<CacheLocation>>(text)
                                            │
                                            ▼
                            ❌ Error en deserialize → fallback a "[]"
                                            │
                                            ▼
                                       []  ──▶ UI: "Sin ubicaciones"
```

### Por qué falla el deserialize

El **JSON real** (verdad del catálogo, no se va a tocar):

```json
{
  "id": "windows-update-downloads",
  "displayName": "Windows Update — Download cache",
  "path": "%WINDIR%\\SoftwareDistribution\\Download",
  "requiresAdmin": true,
  "averageSize": "1-5 GB",
  "preconditions": [...],
  "filters": [...]
}
```

El **struct Rust** lo lee con `serde` que por defecto espera el mismo casing que el campo Rust:

```rust
pub struct CacheLocation {
    pub id: String,
    pub display_name: String,    // serde busca "display_name", JSON da "displayName" → mismatch
    pub category: String,         // JSON no lo trae → falta campo → error
    pub path_template: String,    // serde busca "path_template", JSON da "path" → mismatch
    pub requires_admin: bool,     // "requires_admin" ≠ "requiresAdmin"
    pub risk: String,
    pub consequences: Vec<String>,
    pub average_size: Option<String>,
}
```

Tres problemas a la vez:
1. Casing diferente (snake_case vs camelCase).
2. Nombre del campo `path_template` vs `path`.
3. Campos del struct que el JSON no incluye (`category`, `risk`, `consequences`).

## 2. Estrategia de fix

**Principio:** la fuente de verdad son los catálogos JSON en [.claude/skills/](../.claude/skills/) porque ya están curados y revisados. Rust se adapta a ellos, no al revés.

### 2.1. Decoradores serde

Cada struct serializable cuyos datos vienen del JSON o cruzan a TS gana:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheLocation { ... }
```

Esto permite que el campo Rust `display_name` deserialice/serialice como `displayName` automáticamente, sin tener que renombrar a mano cada campo.

### 2.2. Renombre selectivo

El campo `path_template` se renombra a `path` (alineamos con el JSON), porque "template" implica que esperamos plantillas con `%VAR%` — y es exactamente lo que el JSON pone bajo `"path"`. La función `expand_path()` en [domain/cache.rs](../src-tauri/src/domain/cache.rs) sigue siendo la que evalúa `%VAR%`.

### 2.3. Campos opcionales

`category`, `risk`, `consequences`, `average_size` pasan a `Option<...>` o `Vec<...>` con `#[serde(default)]`. Así:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheLocation {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub category: Option<String>,
    pub path: String,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub consequences: Vec<String>,
    #[serde(default)]
    pub average_size: Option<String>,
}
```

### 2.4. Errores reales en vez de silencio

Antes [domain/catalog.rs](../src-tauri/src/domain/catalog.rs) caía en `[]` cuando el JSON no existía O cuando no parseaba — dos casos muy distintos. Ahora:

```rust
fn load_json<T: serde::de::DeserializeOwned>(candidates: &[&str]) -> AppResult<T> {
    for rel in candidates {
        let path = PathBuf::from(rel);
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            return serde_json::from_str::<T>(&text)
                .map_err(|e| AppError::Catalog(format!("{}: {}", rel, e)));
        }
    }
    // Sólo si NINGÚN candidato existe → devolvemos vacío.
    serde_json::from_str::<T>("[]").map_err(AppError::Json)
}
```

Si en runtime el JSON está roto, la UI mostrará el mensaje real ("`cache-locations.json`: missing field `id` at line 12") en lugar de "Sin ubicaciones".

## 3. Cambios concretos por archivo

### `src-tauri/src/models/cache.rs`

- Añadir `#[serde(rename_all = "camelCase")]` a los 5 structs serializables.
- Renombrar `path_template` → `path` en `CacheLocation`.
- Marcar opcionales con `Option<...>` y `#[serde(default)]`.

### `src-tauri/src/domain/cache.rs`

- En `scan()`, sustituir `loc.path_template` por `loc.path`.
- En `clean()`, sustituir `loc.path_template` por `loc.path`.
- Sin cambios en la lógica de `expand_path()`.

### `src-tauri/src/domain/catalog.rs`

- Cambiar `load_json` para que propague el error real con la ruta y el mensaje del parser.
- Añadir un tercer candidato: `{exe_dir}/../../../.claude/skills/cache-scanner/RESOURCES/cache-locations.json` para cubrir el caso en que `cwd != src-tauri/`.

### `src/api/types.ts`

- Renombrar campos a camelCase (`displayName`, `path`, `requiresAdmin`, etc.) en `CacheLocation`, `CacheScanReport`, `CleanCacheInput`, `PerLocationResult`, `CleanReport`.
- Marcar como `?` los opcionales (`category?`, `risk?`, `consequences?`, `averageSize?`).

### `src/features/cache-cleaner/cache-page.tsx`

- Sustituir `loc.display_name` por `loc.displayName`.
- Sustituir `loc.path_template` por `loc.path`.
- Manejar `loc.category` y `loc.risk` cuando sean `undefined`.
- Sustituir lecturas de `report.bytes` y `bytes_after_filters` (siguen siendo válidos con `rename_all`).

## 4. Optimización del Explorador

[domain/explorer.rs::list_top_level](../src-tauri/src/domain/explorer.rs) hoy recorre **recursivamente** cada subdirectorio para calcular su tamaño. Lanzar el scan sobre `C:\` puede tardar **varios minutos** en un disco con 500GB de datos.

### Solución

Modificar [platform/filesystem.rs::list_children_with_sizes](../src-tauri/src/platform/filesystem.rs):

```rust
pub fn list_children_with_sizes(
    root: &Path,
    follow_reparse_points: bool,
) -> AppResult<Vec<(String, String, bool, u64)>> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(root)?;
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();
        // Para directorios, NO recursar — tamaño 0 hasta que el usuario haga doble click.
        // Para archivos, tamaño directo del metadata (gratis).
        let size = if is_dir {
            0
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };
        out.push((path.to_string_lossy().to_string(), name, is_dir, size));
    }
    Ok(out)
}
```

El cómputo de tamaño recursivo queda exclusivamente para `compute_directory_size`, que el frontend invoca on-demand al expandir un directorio (próxima iteración).

## 5. Plan de verificación

```bash
# 1. Backend compila sin warnings
cd src-tauri && cargo check

# 2. Frontend tipa sin errores
cd .. && npx tsc --noEmit

# 3. Dev
npm run tauri dev
```

En la app:
- [ ] Sidebar → "Caché". Espera ~10-30 entradas listadas. Cada fila muestra nombre + path expandido + categoría/riesgo si los hay.
- [ ] Marcar 2-3 ubicaciones → botón "Escanear". La columna "Detectado" muestra tamaños reales.
- [ ] Botón "Limpiar (X MB)" funciona en seco (dry_run) si activamos el flag.
- [ ] Sidebar → "Explorador". Path "C:\\" → "Escanear". Lista de hijos en <2s. Para archivos, tamaño correcto. Para carpetas, "0 B" (se cargará al expandir).

## 6. Lo que NO entra en esta parte (próximo bloque)

- Doble click en un directorio → expansión y cómputo recursivo de tamaño (`compute_directory_size`).
- Limpieza de caché con creación previa de restore point (depende de `domain::restore` que sigue stub).
- Persistencia del último path explorado.
- Filtros por tipo de archivo / fecha en el explorador.
- Tabla virtualizada con `react-virtual` (necesaria cuando un directorio tiene >5000 entradas).
