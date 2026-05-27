# 15 — Los ~400 MB residuales que nunca se borran

> **Severidad:** 🟡 P1 — comportamiento confuso reportado por el usuario.
> _"La cache siempre se queda como con 400MB ahí parados, y nunca los
> borra, quiero que investigues esto porque no borran o si tiene
> solución, quiero que si no se pueden borrar directamente los quites
> para que salgan"._
> **Modelo:** Sonnet 4.6.
> **Bloque:** 3 de 4 del Set C (quality of life).

## 1. Problema

Tras una limpieza completa, el `verify_after_clean` (cache.rs:887)
reporta consistentemente ~400 MB todavía presentes. El usuario no
sabe qué archivos son ni por qué. Hoy la consola solamente muestra:

```
total_still_present: 400 MB
```

Sin desglose, sin causa, sin acción posible.

## 2. Investigación — causas más probables

### 2.1 Archivos legítimamente bloqueados por procesos del sistema

**Ejemplos típicos:**

| Path | Por qué nunca se borra |
|---|---|
| `%WINDIR%\Logs\CBS\CBS.log` | Servicio TrustedInstaller lo mantiene abierto continuamente |
| `%WINDIR%\Logs\WindowsUpdate\*.etl` | ETW lo escribe constantemente |
| `%LOCALAPPDATA%\Microsoft\Windows\WebCache\*.dat` | Iniciado por svchost (WebCacheSvc); el unmap requiere parar el servicio |
| `%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db` | Explorer escribe en él (sólo se libera con explorer cerrado — pero protegimos el shell en doc 01) |
| `%LOCALAPPDATA%\Microsoft\Windows\INetCache\*.db` | WinINet keep-handle |
| `%WINDIR%\SoftwareDistribution\DataStore\Logs\edb*.log` | WUAUserv, edb es un ESE database log |

Estos son `pending_rename` (programados para reboot) pero la mayoría
de usuarios no reinicia entre limpiezas → los datos se reescriben
inmediatamente → el archivo nunca llega a borrarse.

### 2.2 Archivos protegidos por ACL incluso bajo admin

`%WINDIR%\System32\config\TxR\*`, `%WINDIR%\Logs\WMI\*` — el
propietario es `TrustedInstaller` o `SYSTEM` con DACL que ni
elevación quita. Hay que tomar ownership + reset DACL para tocarlos.
ClearTool no lo hace y bien — es invasivo y rara vez merece la pena.

### 2.3 Carpetas vacías que `read_dir` no enumera bien

`walk_and_delete` (cache.rs:797-857) llama a `remove_dir` después de
vaciar pero **no fuerza recálculo del tamaño antes**. Si la
recursión encuentra un error, las carpetas vacías quedan y suman 0
bytes pero cuentan como "presente" en `verify`.

Por sí mismo no genera "400 MB de aire", pero contamina la cuenta de
archivos.

### 2.4 Filtros `older_than_days` excluyendo bytes que sí cuentan

`file_passes_filters` (cache.rs:249-299) deja archivos modificados
hoy si el filtro pide >7 días. El `verify` ve esos bytes y los
cuenta como "still present". Funcional, pero el usuario lo lee como
"no borró 400 MB" cuando en realidad **sí decidió no borrarlos por
diseño**. Hay que comunicarlo claramente.

### 2.5 Reparse points / junctions

Carpetas como `%PROGRAMDATA%\Microsoft\Windows\WER` contienen junctions
hacia `\\?\Volume{...}`. `std::fs::remove_dir_all` los sigue por
defecto en Rust 1.55+. Si el target es un volumen montado, el `walk`
puede leer bytes que **no existen físicamente en la caché** — los
cuenta dos veces.

### 2.6 Mediciones inconsistentes entre `scan_path_stats` y el real

`scan_path_stats` (cache.rs:506-556) suma `metadata.len()` para todo.
Para archivos sparse, NTFS deduplicated, o compressed → el byte count
es nominal, no físico. El usuario ve "se libera 4 GB" pero su disco
sólo recupera 3.6 GB porque NTFS tenía dedup activo.

## 3. Plan de acción

### 3.1 Telemetría detallada del verify

`src-tauri/src/domain/cache.rs::verify_after_clean` — ya recopila
`bytes_after` por location pero no clasifica el motivo. Ampliar:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResidualEntry {
    pub path: String,
    pub bytes: u64,
    pub reason: ResidualReason,
    /// Sample paths del directorio (hasta 3) para que el usuario los reconozca.
    pub sample_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResidualReason {
    /// Archivo abierto por proceso protegido (sistema/shell).
    LockedBySystem { suggested_action: String },
    /// Programado para borrarse en el próximo reboot.
    PendingReboot,
    /// ACL deniega acceso incluso con admin.
    AccessDenied,
    /// Excluido por filtros del catálogo (older_than_days, exclude pattern).
    FilteredOut,
    /// Reparse point / junction no seguido.
    ReparsePoint,
    /// Razón desconocida — investigar.
    Unknown,
}

impl Default for ResidualReason {
    fn default() -> Self { Self::Unknown }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyLocationResult {
    // ... existentes ...
    /// NUEVO. Desglose del bytes_after_clean por causa.
    pub residuals: Vec<ResidualEntry>,
}
```

Y la función:

```rust
fn classify_residual(
    path: &Path,
    is_pending: bool,
    filters: &CacheFilters,
) -> ResidualReason {
    if is_pending {
        return ResidualReason::PendingReboot;
    }
    // ¿Es reparse point?
    if let Ok(md) = std::fs::symlink_metadata(path) {
        if md.file_type().is_symlink() {
            return ResidualReason::ReparsePoint;
        }
    }
    // ¿Quién lo bloquea?
    if let Ok(lockers) = crate::platform::processes::who_locks_path(path) {
        if !lockers.is_empty() {
            let names: Vec<_> = lockers.iter().take(3).map(|l| l.name.clone()).collect();
            return ResidualReason::LockedBySystem {
                suggested_action: format!(
                    "Bloqueado por: {}. Reinicia el PC para liberar.",
                    names.join(", ")
                ),
            };
        }
    }
    // ¿ACL deniega?
    match std::fs::remove_file(path) {
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return ResidualReason::AccessDenied;
        }
        Err(_) | Ok(_) => {}
    }
    // ¿Pasa los filtros pero seguía presente? Excluido.
    if has_filters(filters) {
        return ResidualReason::FilteredOut;
    }
    ResidualReason::Unknown
}
```

### 3.2 Muestreo en lugar de enumeración exhaustiva

Para no escanear MM archivos cada vez, muestrear hasta N=30 archivos
por ubicación residual:

```rust
fn sample_residual_files(path: &Path, max: usize) -> Vec<String> {
    use walkdir::WalkDir;
    WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .take(max)
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect()
}
```

### 3.3 Acción "Intentar de nuevo" / "Marcar como ignorado"

Cuando el usuario ve la lista de residuales, dos botones por entrada:

- **Intentar de nuevo**: para una ubicación con `LockedBySystem`,
  intentar `schedule_delete_on_reboot` si no estaba ya programado.
  Para `AccessDenied`, intentar `take_ownership_and_delete` con
  confirmación (es invasivo, opt-in explícito).
- **Marcar como ignorado**: añadir el path a un userset
  `%APPDATA%\ClearTool\cache-ignore.json`. El próximo `analyze_locations`
  los salta automáticamente.

Comando IPC nuevo `ignore_residual_path(path)` que añade al userset.

### 3.4 UI — sección "Quedó pendiente" en `CleanSummaryHero`

`src/features/cache-cleaner/clean-summary-hero.tsx` — añadir una
expansión cuando `summary.total_bytes_failed > 0` o
`summary.total_bytes_scheduled_reboot > 0`:

```tsx
{summary.residuals && summary.residuals.length > 0 && (
  <details className="rounded-lg bg-bg-canvas/40 border border-edge-default/30">
    <summary className="cursor-pointer px-3 py-2 text-sm text-ink-secondary">
      Quedó pendiente: {formatBytes(totalResidual)} ({summary.residuals.length} ubicaciones)
    </summary>
    <div className="px-3 pb-3 pt-1 space-y-2">
      {summary.residuals.map((r) => (
        <ResidualRow key={r.path} entry={r} onIgnore={() => ignoreResidualPath(r.path)} />
      ))}
    </div>
  </details>
)}
```

`ResidualRow` muestra:

- Icono según `reason` (cerradura para `LockedBySystem`, reloj para
  `PendingReboot`, candado para `AccessDenied`, filtro para
  `FilteredOut`, link para `ReparsePoint`).
- Nombre legible: `WebCache (15.6 MB) — bloqueado por svchost`.
- Botón pequeño "Ignorar" si la razón es `AccessDenied` o `LockedBySystem`.
- Tooltip con `suggested_action`.

### 3.5 Pantalla del summary — visibilidad por defecto

El bloque de residuales debe aparecer **expandido por defecto** la
primera vez que se ve. Si en una limpieza posterior los residuales
son los mismos (ya marcados como ignorados), colapsar.

Para esto, comparar la lista de residuales con `cache-ignore.json`:

```ts
const allIgnored = summary.residuals.every((r) =>
  ignoredPaths.includes(r.path)
);

<details open={!allIgnored}>
```

### 3.6 Acción opcional: forzar el delete en reboot para residuales pendientes

Si `summary.totalBytesScheduledReboot > 0`, mostrar un CTA al final
del summary:

```
ℹ Hay 256 MB programados para borrarse al reiniciar.

[ Reiniciar ahora ]   [ Más tarde ]
```

Comando IPC `request_reboot()` con `shutdown.exe /r /t 60 /c
"ClearTool: completando limpieza"`. Confirmación visual de 5 s antes
de ejecutar.

## 4. Tests

```rust
#[test]
fn residual_reason_locked_by_system_when_locker_present() {
    // mock con un locker artificial
    // ... (requiere stub de who_locks_path) ...
}

#[test]
fn residual_reason_pending_reboot_when_in_pending_list() {
    // ...
}

#[test]
fn residual_reason_filtered_when_filters_present_and_path_clean() {
    let filters = CacheFilters {
        older_than_days: Some(30),
        ..Default::default()
    };
    let temp = tempfile::tempdir().unwrap();
    let p = temp.path().join("test.txt");
    std::fs::write(&p, b"x").unwrap();
    assert!(matches!(
        classify_residual(&p, false, &filters),
        ResidualReason::FilteredOut
    ));
}
```

## 5. Criterio de done

- [ ] `VerifyLocationResult` incluye `residuals: Vec<ResidualEntry>`.
- [ ] `classify_residual` clasifica al menos 5 de 6 motivos posibles.
- [ ] `CleanSummaryHero` muestra la sección "Quedó pendiente" cuando
      hay residuales con tamaño total y desglose por motivo.
- [ ] Cada `ResidualRow` ofrece **Ignorar** (para `LockedBySystem` y
      `AccessDenied`).
- [ ] El comando IPC `ignore_residual_path` persiste en
      `%APPDATA%\ClearTool\cache-ignore.json` (ring buffer 100).
- [ ] El `analyze_locations` lee `cache-ignore.json` y filtra esos
      paths del `ready` / `permission_issues`, listándolos en
      `skipped` con `reason: UserIgnored`.
- [ ] CTA "Reiniciar ahora" disponible cuando
      `totalBytesScheduledReboot > 0`.
- [ ] Tests pasan: 3 nuevos.

## 6. Riesgos

- **`who_locks_path` ya tiene caché de 30 s** (doc 08 implementado).
  Llamarlo desde `classify_residual` reutiliza esa caché — no añade
  coste perceptible.
- **`take_ownership_and_delete`** es invasivo. Mantenerlo como opción
  **opt-in explícita**, con `ConfirmDialog` previo: "Esto cambia
  los permisos del archivo de forma permanente. ¿Continuar?".
- **`cache-ignore.json` mal formado**: aplicar el mismo patrón que
  `throughput-stats.json` — `load() -> Default` si falla parsing.
- **Falsos positivos en `FilteredOut`**: el filtro `older_than_days`
  excluye archivos jóvenes intencionadamente; mostrarlos como
  "residuales" puede confundir. Mitigación: copy explícito en el
  tooltip ("Excluido por seguridad: este archivo se modificó en los
  últimos N días").
- **Sample de archivos puede revelar paths privados** del usuario en
  el log de errores. Si el log se exporta (doc 19 de auditoría), hay
  que aplicar `cache-ignore.json` y redactar paths sensibles. Por
  ahora, no exportamos logs — sin riesgo inmediato.
