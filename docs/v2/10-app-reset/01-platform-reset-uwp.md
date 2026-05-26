# Paso 01 — Platform: `reset_uwp_app`

**Área**: 10-app-reset
**Tiempo estimado**: 2 horas
**Dependencias**: ninguna

## Qué hacemos

Función que ejecuta `Reset-AppxPackage` para un package family name, con validación regex del input (anti-injection).

## Archivos

- `src-tauri/src/platform/debloat.rs` (añadir función)
- `src-tauri/src/ipc/debloat.rs` (añadir comando)

## Cómo

### Backend

```rust
// src-tauri/src/platform/debloat.rs (añadir)

use regex::Regex;
use once_cell::sync::Lazy;

// Validación: PFN debe ser letras/números/guiones/puntos/underscores.
static PFN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z0-9._-]+_[A-Za-z0-9]+$").unwrap()
});

pub async fn reset_uwp_app(package_family_name: &str) -> AppResult<()> {
    if !PFN_RE.is_match(package_family_name) {
        return Err(AppError::Validation(format!(
            "PackageFamilyName inválido: '{}'", package_family_name
        )));
    }

    let script = format!(
        r#"Get-AppxPackage -Name "{}" | Reset-AppxPackage"#,
        package_family_name.split('_').next().unwrap_or("")
    );

    let out = crate::platform::powershell::run_script_owned(&script)?;
    if !out.status.success() {
        return Err(AppError::Powershell(format!(
            "Reset-AppxPackage falló: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}
```

### IPC

```rust
// src-tauri/src/ipc/debloat.rs (añadir)

#[tauri::command]
pub async fn reset_uwp_app(package_family_name: String) -> AppResult<()> {
    let result = crate::platform::debloat::reset_uwp_app(&package_family_name).await;

    // Audit log
    let _ = crate::domain::audit::write_entry(&crate::domain::audit::make_entry(
        "debloat",
        "reset-appx",
        false,
        None,
        vec![package_family_name.clone()],
        crate::models::restore::ReverseRecipe::Noop {
            reason: "Reset-AppxPackage borra data local. Para revertir, re-login en la app.".into(),
        },
        if result.is_ok() { "success" } else { "failed" },
        result.as_ref().err().map(|e| format!("{}", e)),
    ));

    result
}
```

Registrar en `lib.rs`.

### TS wrapper

```ts
// src/api/client.ts
export const resetUwpApp = (packageFamilyName: string) =>
  invoke<void>("reset_uwp_app", { packageFamilyName });
```

## Criterio de done

- [ ] `reset_uwp_app("SpotifyAB.SpotifyMusic_zpdnekdrzrea0")` ejecuta y devuelve Ok.
- [ ] Inputs inválidos (con `;`, ``$()``, etc.) devuelven `AppError::Validation`.
- [ ] Audit log entry generada.
- [ ] Test manual: reset Spotify → al abrir Spotify, login requerido (proof de que el reset borró data).
