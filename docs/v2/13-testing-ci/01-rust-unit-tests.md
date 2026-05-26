# Paso 01 — Rust unit tests para platform + domain

**Área**: 13-testing-ci
**Tiempo estimado**: 4-6 horas (distribuidas)
**Dependencias**: módulos a testear ya implementados

## Qué hacemos

Añadir tests unitarios para los módulos que se pueden testear sin Windows real:
- Validadores (regex, paths, denylist).
- Parsers (catalog JSON, audit log entries).
- Lógica de clasificación (process category, cache strategy).
- Reverse recipe builders.

Y tests de integración (requieren Windows runner) para:
- list_processes_extended (verifica que devuelve >0 procesos).
- Restart Manager API.
- Registry read (HKLM/HKCU subkeys conocidos).

## Archivos

- Tests inline en cada módulo (`#[cfg(test)] mod tests { ... }`).
- Tests de integración en `src-tauri/tests/`.

## Cómo

### 1. Tests inline en módulos

```rust
// src-tauri/src/platform/processes.rs (al final)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_system_processes() {
        assert_eq!(classify("csrss.exe", None), ProcessCategory::System);
        assert_eq!(classify("svchost.exe", None), ProcessCategory::Service);
        assert_eq!(classify("notepad.exe", None), ProcessCategory::UserApp);
    }

    #[test]
    fn classify_browsers() {
        assert_eq!(classify("chrome.exe", None), ProcessCategory::Browser);
        assert_eq!(classify("msedge.exe", None), ProcessCategory::Browser);
        assert_eq!(classify("firefox.exe", None), ProcessCategory::Browser);
    }

    #[test]
    fn is_system_protected_obvious_cases() {
        assert!(is_protected_by_name("csrss.exe"));
        assert!(is_protected_by_name("CSRSS.EXE"));  // case-insensitive
        assert!(is_protected_by_name("lsass.exe"));
        assert!(!is_protected_by_name("chrome.exe"));
    }

    #[test]
    fn pid_0_and_4_always_protected() {
        assert!(is_system_protected_pid(0));
        assert!(is_system_protected_pid(4));
    }
}
```

### 2. Tests para denylist + path normalize

```rust
// src-tauri/src/domain/catalog.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denylist_blocks_windows_terminal() {
        let deny = load_denylist().unwrap();
        assert!(is_path_denied(
            r"C:\Users\foo\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState",
            &deny,
        ));
    }

    #[test]
    fn denylist_allows_normal_temp() {
        let deny = load_denylist().unwrap();
        assert!(!is_path_denied(r"C:\Users\foo\AppData\Local\Temp", &deny));
    }
}
```

### 3. Tests para regex de validación

```rust
// src-tauri/src/platform/debloat.rs (al final)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pfn_regex_accepts_valid() {
        assert!(PFN_RE.is_match("Microsoft.Copilot_8wekyb3d8bbwe"));
        assert!(PFN_RE.is_match("SpotifyAB.SpotifyMusic_zpdnekdrzrea0"));
    }

    #[test]
    fn pfn_regex_rejects_injection() {
        assert!(!PFN_RE.is_match("Microsoft.Copilot; Remove-Item C:\\"));
        assert!(!PFN_RE.is_match("Microsoft.Copilot$(whoami)"));
        assert!(!PFN_RE.is_match("Microsoft.Copilot`cmd.exe`"));
    }
}
```

### 4. Tests para normalizeError equivalente

Si el frontend ya tiene errors.test.ts, no es necesario en Rust. Pero para el `with_context`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_context_prepends_message() {
        let err = AppError::Io("file not found".into());
        let wrapped = err.with_context("eliminando archivo X");
        match wrapped {
            AppError::Io(msg) => assert_eq!(msg, "eliminando archivo X: file not found"),
            _ => panic!("kind cambió"),
        }
    }
}
```

### 5. Tests para audit log

```rust
// src-tauri/src/domain/audit.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_and_read_entry() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("audit.jsonl");

        let entry = make_entry(
            "test", "noop", false, None, vec!["x".into()],
            ReverseRecipe::Noop { reason: "test".into() },
            "success", None,
        );

        write_entry_to(&path, &entry).unwrap();

        let read = read_log_from(&path).unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].module, "test");
    }
}
```

(Refactorizar `write_entry` para aceptar path inyectable para que sea testeable).

### 6. Tests de integración (requieren Windows real)

Carpeta `src-tauri/tests/`:

```rust
// src-tauri/tests/integration_processes.rs
// Estos tests requieren Windows. Usan #[cfg(windows)] guard.

#[cfg(windows)]
#[test]
fn list_processes_returns_some() {
    let procs = cleartool::platform::processes::list_processes_extended().unwrap();
    assert!(!procs.is_empty(), "debería haber procesos en el sistema");
}

#[cfg(windows)]
#[test]
fn list_processes_contains_system_process() {
    let procs = cleartool::platform::processes::list_processes_extended().unwrap();
    let has_system = procs.iter().any(|p|
        p.category == cleartool::models::process::ProcessCategory::System
    );
    assert!(has_system, "debe haber al menos un proceso System");
}
```

### 7. Test runner config

`Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
serial_test = "3"   # para tests que tocan registry/global state
```

`.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

### 8. Ejecutar

```bash
cd src-tauri
cargo test                  # todos
cargo test --lib            # solo unit tests
cargo test --tests          # solo integration
cargo test classify_        # solo los que matchan
```

## Criterio de done

- [ ] 50+ tests inline en módulos.
- [ ] 5-10 tests integration en `tests/`.
- [ ] `cargo test` corre <30 segundos.
- [ ] Coverage >40% en `core/`, `domain/`, `platform/validators`.
- [ ] CI corre `cargo test` en cada PR (paso 04).
- [ ] No tests flaky (los que dependen de timing → marcarlos `#[ignore]` o usar `serial_test`).
