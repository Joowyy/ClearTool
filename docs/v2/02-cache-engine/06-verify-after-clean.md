# Paso 06 — Verify after clean

**Área**: 02-cache-engine
**Tiempo estimado**: 2 horas
**Dependencias**: Paso 05

## Qué hacemos

Función `verify_after_clean(plan_id)` que re-escanea los paths del plan y devuelve cuánto realmente quedó vs el plan original. Es la pieza que cierra el círculo: el usuario ve el resultado REAL, no el reportado.

## Por qué

Hoy el usuario hace "Limpiar" y le sale un toast verde "3 GB liberados", pero al re-escanear sale el mismo tamaño. Eso es lo que rompe la confianza. La verificación post-clean cierra esa brecha.

## Archivos que tocamos

- `src-tauri/src/domain/cache.rs` (función `verify_after_clean`)
- `src-tauri/src/models/cache.rs` (modelo `VerifyReport`)

## Cómo

### 1. Modelo

```rust
// src-tauri/src/models/cache.rs (añadir)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReport {
    pub plan_id: String,
    pub verified_at: String,
    pub per_location: Vec<VerifyLocationResult>,
    pub total_actually_freed: u64,
    pub total_still_present: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyLocationResult {
    pub id: String,
    pub display_name: String,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub bytes_actually_freed: u64,    // before - after
    pub bytes_pending_reboot: u64,    // suma de pending renames que aún apunta aquí
    pub success_percent: f32,         // 100 = todo limpio; 50 = mitad sigue ahí
}
```

### 2. Cache del plan

Necesitamos guardar el plan en memoria para que `verify_after_clean` sepa qué paths verificar. Opciones:
- (A) Pasar el plan completo como parámetro al verify command.
- (B) Cachear el último plan en memoria (Arc<Mutex<Option<CleanPlan>>>).

Elige (A) — más simple, sin estado global.

### 3. Implementación

```rust
// src-tauri/src/domain/cache.rs

pub fn verify_after_clean(plan: &CleanPlan, report: &CleanReportV2) -> AppResult<VerifyReport> {
    let mut per_location = Vec::new();
    let mut total_actually_freed = 0u64;
    let mut total_still_present = 0u64;

    // Pending renames actuales (después del clean) — útil para detectar lo que está en cola
    let pendings = crate::platform::pending_rename::list_pending_renames()
        .unwrap_or_default();

    for ready in &plan.ready {
        let path = std::path::Path::new(&ready.resolved_path);
        let bytes_after = if path.exists() {
            scan_path_stats(path).map(|(b, _, _)| b).unwrap_or(0)
        } else { 0 };

        let bytes_before = ready.bytes;
        let bytes_actually_freed = bytes_before.saturating_sub(bytes_after);

        // ¿Cuánto está en pending para este path?
        let bytes_pending = pendings.iter()
            .filter(|p| p.is_delete && p.source.starts_with(&ready.resolved_path))
            .count() as u64
            * 0; // No tenemos size de pendings sin re-stat; aproximamos a 0 o re-stat los paths

        let success_percent = if bytes_before == 0 {
            100.0
        } else {
            (bytes_actually_freed as f32 / bytes_before as f32) * 100.0
        };

        total_actually_freed += bytes_actually_freed;
        total_still_present += bytes_after;

        per_location.push(VerifyLocationResult {
            id: ready.id.clone(),
            display_name: ready.display_name.clone(),
            bytes_before,
            bytes_after,
            bytes_actually_freed,
            bytes_pending_reboot: bytes_pending,
            success_percent,
        });
    }

    Ok(VerifyReport {
        plan_id: plan.plan_id.clone(),
        verified_at: chrono::Utc::now().to_rfc3339(),
        per_location,
        total_actually_freed,
        total_still_present,
    })
}
```

### 4. Llamada automática desde el flow

En el frontend, tras `execute_plan` completar, llamar `verify_after_clean` automáticamente y mostrar el resultado en un modal "Resumen":

```tsx
// Pseudo-código frontend
const report = await executePlan(plan, opts);
const verify = await verifyAfterClean(plan, report);

toast.success(
  `Liberado: ${formatBytes(verify.totalActuallyFreed)}`,
  {
    description: `${verify.totalStillPresent === 0
      ? "Limpieza completa."
      : `${formatBytes(verify.totalStillPresent)} no se pudieron eliminar.`}`
  }
);
```

### 5. Caveat sobre paths que se rescanean

Re-escanear un path grande tarda. Si el path tenía 5 GB y vamos a re-walk:
- En SSD nuevo: <5 segundos.
- En HDD antiguo: 30+ segundos.

Considera ejecutar en background y mostrar el verify report cuando llegue. El toast inmediato puede ser "Limpieza completa, verificando resultado..." y luego actualizar.

### 6. Cálculo de bytes_pending

Hacer el conteo real de bytes en pendings es complicado (no tenemos size sin stat). Opciones:

- (A) Stat cada path pending (lento, pero preciso).
- (B) Aproximar: contar archivos pending del path × tamaño promedio del location.
- (C) Solo contar la cantidad de archivos pending (sin bytes).

Para v1, ir con **(C)**: añadir `files_pending_reboot: u32` en vez de `bytes_pending_reboot`. Más honesto.

```rust
pub files_pending_reboot: u32,
```

```rust
let files_pending = pendings.iter()
    .filter(|p| p.is_delete && path_under(&p.source, &ready.resolved_path))
    .count() as u32;

fn path_under(child: &str, parent: &str) -> bool {
    let c = child.to_lowercase().replace('/', "\\");
    let p = parent.to_lowercase().replace('/', "\\");
    c.starts_with(&p)
}
```

## Criterio de done

- [ ] `verify_after_clean` re-walka los paths del plan y devuelve VerifyReport.
- [ ] `total_actually_freed` refleja bytes reales liberados (no los reportados por execute).
- [ ] `success_percent` por location entre 0 y 100.
- [ ] Tras `execute_plan`, la UI muestra el VerifyReport automáticamente.
- [ ] Si `bytes_after > 0`, la UI sugiere "reiniciar para completar" si hay pendings.

## Próximo paso

`07-catalogo-strategies.md`.
