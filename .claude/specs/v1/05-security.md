# 05 — Modelo de Seguridad

ClearTool ejecuta operaciones que pueden dejar un sistema Windows inutilizable si se equivoca. Esta spec define las amenazas que consideramos y las mitigaciones obligatorias por capa.

## Activos a proteger

1. **Integridad del SO del usuario** — que ClearTool no degrade arranque, networking, ni Defender.
2. **Datos del usuario** — `Documents`, `Pictures`, `Videos`, `Desktop`, `OneDrive` son intocables por diseño.
3. **Estado reversible** — todo cambio destructivo debe poder revertirse vía SRP o audit log.
4. **Cadena de suministro** — el binario que el usuario ejecuta debe corresponder al que se construyó desde el repo.

## Modelo de amenazas

### Atacante 1: bug propio

El código tiene un fallo lógico que borra/desinstala más de lo que el usuario aprobó.

**Mitigaciones:**

- Allowlist de paths (`cache-locations.json`) y de hives/keys (`PATH_ALLOWLIST` en `services::registry`). Cualquier intento fuera → `AppError::Permission`, abortar.
- Nunca `remove_dir_all` como primera primitiva: borrado archivo a archivo con validación de prefijo.
- `dry_run` obligatorio en cada comando destructivo; la UI lo expone como botón secundario.
- Tests "no destructivos" en CI: dado un fixture de FS, ningún comando borra fuera del root.
- `security-auditor` (subagente) revisa cada PR que toque `services::*` o catálogos.

### Atacante 2: input malicioso del usuario

El usuario escribe un path arbitrario en el explorer o un `id` inexistente al invocar.

**Mitigaciones:**

- Validación + canonicalización (`dunce::canonicalize`) en cada comando.
- Allowlist de drives (`C:\`, `D:\`, …) detectados al arranque; rechazo de `\\?\GLOBALROOT`, `\\Device\\…`, `\\.\`.
- IDs de operaciones (cache, debloat, tweaks) se validan contra el catálogo cargado en memoria.

### Atacante 3: contenido inyectable en PowerShell

PowerShell scripts reciben argumentos. Si los args se construyen concatenando strings, cualquier `;` o `|` permite escalada de comandos.

**Mitigaciones:**

- Scripts PS son **archivos embebidos** (`include_str!`) con parámetros nombrados (`param([string]$Pattern, [string]$Scope)`). Nunca se construye PS dinámicamente desde input.
- `ExecutionPolicy Bypass` se pasa **únicamente** al proceso `powershell.exe` que invocamos, no se modifica la policy del sistema.
- Encoding del comando: `-EncodedCommand <base64(UTF-16LE)>` para evitar problemas de quoting con espacios y caracteres especiales en valores.
- Validación de args en Rust antes de pasarlos:
  - `Pattern` (Appx) → regex `^[A-Za-z0-9._\-\*]+$` (permite wildcards Appx).
  - `Scope` ∈ {`CurrentUser`, `AllUsers`, `Provisioned`}.
  - Cualquier valor con `;`, `|`, `&`, backtick, `$(`, `>` → rechazo `Permission`.
- Cada script tiene un ID interno (`appx_remove`, `appx_list`, …) y un test que ejercita un payload "malicioso" — el script debe producir error o ignorarlo.

### Atacante 4: race condition / TOCTOU

Entre validar un path y borrarlo, un atacante o app legítima reemplaza el archivo por un junction que apunta a `C:\Windows\System32`.

**Mitigaciones:**

- Re-canonicalizar inmediatamente antes de la llamada destructiva.
- Para directorios: abrir `HANDLE` con `FILE_FLAG_OPEN_REPARSE_POINT` y validar reparse data antes de descender.
- Para archivos: borrar usando handle abierto (`DeleteFileW` por path es vulnerable; preferimos `SetFileInformationByHandle(FileDispositionInfo, DeleteFile=TRUE)` con un handle ya validado).

### Atacante 5: privilege confusion

Operación que necesita admin se ejecuta sin admin y produce un estado parcial.

**Mitigaciones:**

- `is_elevated()` primero en cada comando destructivo. Sin elevation → `AppError::NotElevated` antes de cualquier side-effect.
- En `tauri.conf.json` el bundle pide `requireAdministrator` por manifest UAC. Si el usuario rechaza UAC, la app levanta en **modo limitado**: lecturas y escaneos OK, destructivas deshabilitadas.
- `ServiceGuard` y locks globales evitan que dos comandos toquen el mismo recurso (servicio, registry key) en paralelo.

### Atacante 6: webview comprometido

Algún script del frontend logra ejecutar JS arbitrario.

**Mitigaciones:**

- CSP estricto en `tauri.conf.json`:

  ```
  default-src 'self';
  img-src 'self' data: asset:;
  style-src 'self' 'unsafe-inline';
  script-src 'self';
  connect-src 'self' ipc: http://ipc.localhost
  ```

- No se cargan scripts de CDN externos. Bundling completo via Vite.
- `tauri-plugin-shell` en modo "open URL externo solo allowlist" — no permite `cmd /c` arbitrario.
- `tauri-plugin-fs` con scope explícito (read en `%TEMP%`, etc.). Sin escritura general.
- Capabilities (`capabilities/default.json`, `capabilities/elevated.json`) declaran qué permisos están disponibles para qué páginas. La UI no puede invocar `apply_registry_tweak` desde una página que no lo tenga listado.

### Atacante 7: cadena de suministro

Alguien sustituye `cleartool.msi` en el sitio de descarga.

**Mitigaciones (futuro, fuera de MVP):**

- Firmado del bundle MSI/NSIS con cert de code-signing.
- Hash SHA-256 publicado junto al binario.
- `Notarizing` no aplica en Windows; alternativa: SmartScreen reputation tras N descargas.
- Updater con verificación de firma del paquete.

## Privilegios mínimos por operación

| Operación | Necesita admin | Capability |
|---|---|---|
| `is_elevated`, `system_summary` | No | default |
| `scan_tree`, `compute_directory_size` | No | default |
| `list_cache_locations`, `scan_cache_locations` | No | default |
| `clean_cache_locations` | Depende de la entrada (algunas user-temp no requieren) | elevated cuando aplica |
| `list_bloatware_catalog`, `detect_installed_bloatware` | No | default |
| `remove_bloatware` | Sí | elevated |
| `list_services` | No | default |
| `set_service_state`, `apply_service_preset` | Sí | elevated |
| `list_registry_tweaks`, `read_registry_tweak_state` | Depende del hive (HKLM lectura no necesita admin pero a veces sí) | default |
| `apply_registry_tweak`, `revert_registry_tweak` | Sí (HKLM) | elevated |
| `create_restore_point`, `restore_to_point` | Sí | elevated |
| `list_audit_log` | No | default |
| `revert_audit_entry` | Depende de la receta | elevated cuando aplica |

## Validaciones obligatorias por capa

```
UI (validación blanda)
  └─> Comando Tauri (validación estricta)
        └─> Servicio (lógica de negocio, asume input válido)
              └─> APIs Win32 / PS (sin validación adicional, pero con allowlists previas)
```

La capa "Comando" rechaza cualquier input que no pase:

- IDs ⊆ catálogo cargado.
- Paths canonicalizables y dentro de drives detectados.
- Args de scripts PS regex-validados.
- Hives ∈ {HKLM, HKCU, HKCR, HKU, HKCC} y path ∈ allowlist.

## Logging y telemetría

- Telemetría a Microsoft / a nosotros: **cero**. ClearTool no llama a la red salvo en updates (no MVP).
- Logging local: `tracing` con `tauri-plugin-log`. Niveles INFO en prod, DEBUG en dev. Logs en `%APPDATA%\ClearTool\logs\app.log`.
- El log de aplicación **no** debe contener:
  - Contenido de archivos borrados.
  - Valores binarios largos del registro.
  - Tokens, API keys, secretos.
- El audit log (`audit.jsonl`) sí contiene snapshots de registro `before/after`. Es local, no se envía a ningún lado, pero queda documentado en el README que el usuario puede revisarlo o borrarlo.

## Reversibilidad como invariante

Para cada operación destructiva, antes de side-effects:

1. Crear restore point (o reportar `RestoreUnavailable` y preguntar).
2. Escribir audit entry "intent" (campo `payload_summary = "starting"`).
3. Ejecutar.
4. Cerrar audit entry con `payload_summary` final y `reverse_recipe`.

Si el binario muere entre 2 y 4, la próxima ejecución detecta entries `starting` huérfanos y los marca como `interrupted` para que la UI los muestre con "Estado desconocido — recomendamos restaurar al punto N".

## Hardening del binario

- `panic = "abort"` en release: evita unwinding tras un panic en código que ya hizo I/O destructivo a medias.
- LTO + strip + opt-level=3 → tamaño y dificultad de reverse engineering.
- Manifest UAC `asInvoker` no se usa: siempre `requireAdministrator`. Excepción: si usuario explícitamente lanza con `--no-elevation` (no implementado MVP).

## Auditabilidad

- Catálogos en JSON versionados en el repo, con `version` y `updatedAt`.
- Cada release guarda en `CHANGELOG.md` qué entradas del catálogo cambiaron.
- `security-auditor` aprueba cualquier modificación a:
  - `cache-locations.json`
  - `bloatware-catalog.json`
  - `services-catalog.json`
  - `registry-tweaks.json`
  - `services::registry::ALLOWED_PREFIXES`

## Checklist por PR

Cada PR que toque módulos destructivos debe llevar en la descripción:

- [ ] ¿Toca paths/keys/services nuevos? Listar.
- [ ] ¿Pasa por allowlist? Sí/no, dónde.
- [ ] ¿Tiene reverse_recipe completo?
- [ ] ¿Hay test de idempotencia?
- [ ] ¿Hay test de "no toca fuera del scope declarado"?
- [ ] ¿Hay actualización de `consequences` user-visible si aplica?
- [ ] ¿Aprobó `security-auditor`?
