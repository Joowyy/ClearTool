---
name: windows-registry-ops
description: Patrones seguros e idempotentes para leer y escribir el registro de Windows (HKLM, HKCU, HKU, HKCR) desde Rust usando winreg. Uso obligatorio cuando un comando Tauri toca el registro. Incluye whitelist de ramas permitidas, plantilla de backup automático a .reg con timestamp, validación contra path traversal, y el patrón ReversibleRegistryEdit para registrar cambios reversibles.
---

# Skill: windows-registry-ops

Toda operación de escritura al registro en ClearTool debe pasar por los patrones documentados aquí.

## Cuándo usar este skill

- Vas a escribir, modificar o borrar valores del registro.
- Vas a leer valores que después usarás para tomar decisiones destructivas.
- Vas a implementar tweaks de Windows 11.

## Cuándo NO usar

- Leer información puramente cosmética (versión Windows, edición). Para eso basta `winreg` directo sin las salvaguardas.

## Whitelist de ramas permitidas para ESCRITURA

```rust
// services/registry.rs

/// Ramas que ClearTool tiene permiso de modificar.
/// Cualquier intento de escritura fuera de esta lista devuelve AppError::Permission.
pub const WRITE_ALLOWED_PREFIXES: &[(&str, &str)] = &[
    // (hive, path_prefix_lowercase)
    ("HKLM", r"software\policies\microsoft\windows\"),
    ("HKLM", r"software\policies\microsoft\dsh"),
    ("HKLM", r"software\policies\microsoft\edge"),
    ("HKLM", r"software\policies\microsoft\edgeupdate"),
    ("HKLM", r"software\microsoft\windows\currentversion\policies\"),
    ("HKLM", r"system\currentcontrolset\services\"),    // solo subkeys de servicios concretos validados aparte
    ("HKCU", r"software\microsoft\windows\currentversion\contentdeliverymanager\"),
    ("HKCU", r"software\microsoft\windows\currentversion\explorer\advanced"),
    ("HKCU", r"software\policies\microsoft\windows\windowsai"),
    ("HKCU", r"software\policies\microsoft\windows\windowscopilot"),
    ("HKCU", r"software\classes\clsid\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\"), // menú clásico
    ("HKCU", r"software\cleartool\"),                    // namespace propio
];

pub const READ_ALLOWED_PREFIXES: &[(&str, &str)] = &[
    // mucho más permisivo, pero todavía bounded
    ("HKLM", r"software\"),
    ("HKLM", r"system\currentcontrolset\"),
    ("HKCU", r"software\"),
    ("HKU",  r""),
    ("HKCR", r""),
];

/// Bloquear absolutamente.
pub const FORBIDDEN_PREFIXES: &[(&str, &str)] = &[
    ("HKLM", r"sam"),
    ("HKLM", r"security"),
    ("HKLM", r"bcd"),
];
```

## Validación previa

```rust
pub fn validate_write_path(hive: Hive, path: &str) -> Result<(), AppError> {
    let lower = path.to_ascii_lowercase();
    let h = hive.as_str();
    if FORBIDDEN_PREFIXES.iter().any(|(fh, fp)| *fh == h && lower.starts_with(fp)) {
        return Err(AppError::Permission(format!("forbidden: {}\\{}", h, path)));
    }
    if !WRITE_ALLOWED_PREFIXES.iter().any(|(ah, ap)| *ah == h && lower.starts_with(ap)) {
        return Err(AppError::Permission(format!("not in whitelist: {}\\{}", h, path)));
    }
    Ok(())
}
```

## Patrón ReversibleRegistryEdit

Toda escritura registra estado anterior y se materializa en el "log de cambios" reversible.

```rust
pub struct ReversibleRegistryEdit {
    pub hive: Hive,
    pub path: String,
    pub value_name: String,
    pub previous: Option<RegistryValue>,
    pub next: RegistryValue,
    pub applied_at: chrono::DateTime<chrono::Utc>,
}

impl ReversibleRegistryEdit {
    pub fn apply(&self) -> Result<(), AppError> {
        validate_write_path(self.hive, &self.path)?;
        let key = self.hive.create_subkey(&self.path)?;
        self.next.write_to(&key, &self.value_name)?;
        Ok(())
    }

    pub fn revert(&self) -> Result<(), AppError> {
        validate_write_path(self.hive, &self.path)?;
        let key = self.hive.open_subkey_with_flags(&self.path, KEY_SET_VALUE)?;
        match &self.previous {
            Some(prev) => prev.write_to(&key, &self.value_name)?,
            None => key.delete_value(&self.value_name)?,
        }
        Ok(())
    }
}
```

## Backup automático a .reg

Antes de cualquier modificación, exporta la rama afectada:

```rust
pub fn backup_branch(hive: Hive, path: &str) -> Result<PathBuf, AppError> {
    let backup_dir = local_appdata().join("ClearTool/backups/registry");
    std::fs::create_dir_all(&backup_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let safe = path.replace('\\', "_").replace('/', "_");
    let file = backup_dir.join(format!("{}_{}_{}.reg", hive.as_str(), safe, stamp));

    let status = std::process::Command::new("reg.exe")
        .arg("export")
        .arg(format!("{}\\{}", hive.as_str(), path))
        .arg(&file)
        .arg("/y")
        .status()?;

    if !status.success() {
        return Err(AppError::Registry(format!("reg export failed for {}", path)));
    }
    Ok(file)
}
```

## DWORD vs QWORD vs SZ

Siempre tipar explícitamente:

```rust
pub enum RegistryValue {
    Dword(u32),
    Qword(u64),
    Sz(String),
    ExpandSz(String),
    MultiSz(Vec<String>),
    Binary(Vec<u8>),
    None,
}
```

## Tests obligatorios

Cualquier nueva escritura al registro requiere test que:

1. Confirme que la validación rechaza paths fuera del whitelist.
2. Confirme idempotencia (aplicar dos veces no lanza error).
3. Confirme que `revert` restaura exactamente el estado previo.

## Lecturas recomendadas

- `winreg` crate: https://docs.rs/winreg/latest/winreg/
- Win32 Registry API: https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry
