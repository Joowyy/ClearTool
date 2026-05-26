# Paso 01 — Backend: 6 comandos de red

**Área**: 08-network-utilities
**Tiempo estimado**: 2 horas
**Dependencias**: ninguna

## Qué hacemos

Implementar 6 funciones en `platform::network` que llaman los comandos de red estándar de Windows.

## Archivos

- `src-tauri/src/platform/network.rs` (nuevo)
- `src-tauri/src/domain/network.rs` (nuevo, con audit log)

## Cómo

```rust
// src-tauri/src/platform/network.rs
use crate::core::{AppError, AppResult};
use std::process::Command;

pub fn flush_dns() -> AppResult<String> { run("ipconfig", &["/flushdns"]) }
pub fn release_ip() -> AppResult<String> { run("ipconfig", &["/release"]) }
pub fn renew_ip() -> AppResult<String> { run("ipconfig", &["/renew"]) }
pub fn reset_winsock() -> AppResult<String> { run("netsh", &["winsock", "reset"]) }
pub fn reset_tcpip() -> AppResult<String> { run("netsh", &["int", "ip", "reset"]) }
pub fn reset_proxy() -> AppResult<String> { run("netsh", &["winhttp", "reset", "proxy"]) }

const HOSTS_DEFAULT: &str = r#"# Copyright (c) 1993-2009 Microsoft Corp.
#
# This is a sample HOSTS file used by Microsoft TCP/IP for Windows.

# localhost name resolution is handled within DNS itself.
#   127.0.0.1       localhost
#   ::1             localhost
"#;

pub fn restore_hosts_file() -> AppResult<()> {
    let path = r"C:\Windows\System32\drivers\etc\hosts";

    // Backup primero
    let backup = format!("{}.cleartool-backup-{}", path, chrono::Utc::now().timestamp());
    std::fs::copy(path, &backup)
        .map_err(|e| AppError::Io(format!("backup hosts: {}", e)))?;

    std::fs::write(path, HOSTS_DEFAULT)
        .map_err(|e| AppError::Io(format!("write hosts: {}", e)))?;

    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> AppResult<String> {
    let out = Command::new(cmd).args(args).output()
        .map_err(|e| AppError::Io(format!("{}: {}", cmd, e)))?;
    if !out.status.success() {
        return Err(AppError::Io(format!(
            "{} {} failed: {}",
            cmd,
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
```

### Domain con audit

```rust
// src-tauri/src/domain/network.rs
use crate::core::AppResult;
use crate::models::restore::ReverseRecipe;
use crate::platform::network as nw;

pub fn flush_dns() -> AppResult<String> {
    let r = nw::flush_dns()?;
    audit_entry("flush-dns", ReverseRecipe::Noop {
        reason: "DNS cache se reconstruye automáticamente con uso.".into(),
    });
    Ok(r)
}

pub fn renew_ip() -> AppResult<String> {
    let r = nw::release_ip();
    let _ = r; // ignore release error
    let r2 = nw::renew_ip()?;
    audit_entry("renew-ip", ReverseRecipe::Noop {
        reason: "Cambiar IP no es reversible directamente.".into(),
    });
    Ok(r2)
}

pub fn reset_winsock() -> AppResult<String> {
    // Crear restore point — esto puede romper Internet si hay config custom
    let _ = crate::domain::restore::ensure_or_create("ClearTool — antes de reset Winsock");
    let r = nw::reset_winsock()?;
    audit_entry("reset-winsock", ReverseRecipe::Noop {
        reason: "Requiere reinicio. La configuración previa se pierde.".into(),
    });
    Ok(r)
}

pub fn reset_tcpip() -> AppResult<String> {
    let _ = crate::domain::restore::ensure_or_create("ClearTool — antes de reset TCP/IP");
    let r = nw::reset_tcpip()?;
    audit_entry("reset-tcpip", ReverseRecipe::Noop {
        reason: "Requiere reinicio. La configuración previa se pierde.".into(),
    });
    Ok(r)
}

pub fn reset_proxy() -> AppResult<String> {
    let r = nw::reset_proxy()?;
    audit_entry("reset-proxy", ReverseRecipe::Noop {
        reason: "Reconfigurar proxy manualmente si era necesario.".into(),
    });
    Ok(r)
}

pub fn restore_hosts_file() -> AppResult<()> {
    let _ = crate::domain::restore::ensure_or_create("ClearTool — antes de restaurar hosts");
    nw::restore_hosts_file()?;
    audit_entry("restore-hosts", ReverseRecipe::Noop {
        reason: "Backup creado al lado del hosts original (.cleartool-backup-<ts>).".into(),
    });
    Ok(())
}

fn audit_entry(operation: &str, recipe: ReverseRecipe) {
    let _ = crate::domain::audit::write_entry(&crate::domain::audit::make_entry(
        "network", operation, false, None, vec![],
        recipe, "success", None,
    ));
}
```

## Criterio de done

- [ ] 6 funciones implementadas y testadas individualmente.
- [ ] `flush_dns` ejecuta sin requerir admin (es user-level).
- [ ] Las que requieren admin (`reset_winsock`, `reset_tcpip`, restore hosts) fallan con AppError::Permission si no está elevated.
- [ ] `restore_hosts_file` crea backup antes.
- [ ] Cada operación genera audit entry.
- [ ] Restore point automático antes de Winsock/TCPIP reset.
