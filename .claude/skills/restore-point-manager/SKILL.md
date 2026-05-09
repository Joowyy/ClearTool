---
name: restore-point-manager
description: Crear, listar, validar y restaurar puntos de restauración del sistema (System Restore) desde Rust vía WMI/Win32. Toda operación destructiva en ClearTool DEBE crear un restore point antes mediante este skill. Maneja también el log de cambios reversible append-only en %LOCALAPPDATA%\\ClearTool\\logs\\.
---

# Skill: restore-point-manager

Toda operación destructiva pasa por aquí antes de tocar nada.

## Cuándo usar

- Antes de ejecutar cualquier debloat, registry tweak, o disable de servicio.
- Antes de batch destructivo (varias operaciones encadenadas) — un único restore point para todo el batch.
- Cuando el usuario pide ver/restaurar puntos previos.

## API que expones

```rust
pub struct RestorePointId(pub u32);

#[tauri::command]
pub async fn create_restore_point(
    description: String,
    point_type: RestorePointType,    // Application, ApplicationUninstall, Modify, Cancel
) -> Result<RestorePointInfo, AppError>;

#[tauri::command]
pub async fn list_restore_points() -> Result<Vec<RestorePointInfo>, AppError>;

#[tauri::command]
pub async fn restore_to_point(id: RestorePointId) -> Result<(), AppError>;
```

## Implementación: crear punto vía WMI

System Restore vive en WMI namespace `root\default`, clase `SystemRestore`. Método `CreateRestorePoint`.

```rust
// services/restore_point.rs
use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Wmi::*;

pub fn create_restore_point(description: &str, point_type: u32) -> Result<u32, AppError> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
        let services = locator.ConnectServer(
            &BSTR::from(r"root\default"),
            &BSTR::default(), &BSTR::default(),
            &BSTR::default(),
            0, &BSTR::default(),
            None,
        )?;

        // SystemRestore.CreateRestorePoint(Description, RestorePointType, EventType)
        // EventType: BEGIN_SYSTEM_CHANGE = 100
        // RestorePointType: APPLICATION_INSTALL = 0, MODIFY_SETTINGS = 12
        let class_path: BSTR = "SystemRestore".into();
        let method: BSTR = "CreateRestorePoint".into();

        let class: IWbemClassObject = services.GetObject(&class_path, 0, None)?.into();
        let mut params: IWbemClassObject = class.GetMethod(&method, 0, None, None)?.into();

        // ... rellenar params con description, point_type, 100 ...
        // (ver windows-rs docs para Put/Get de propiedades)

        let result = services.ExecMethod(&class_path, &method, 0, None, Some(&params), None, None)?;
        // Leer ReturnValue de result
        Ok(0)  // sustituir por el id real (SequenceNumber)
    }
}
```

> El detalle de Put/Get vía windows-rs es verboso. Para iteración rápida, una alternativa pragmática es invocar PowerShell:
>
> ```powershell
> Checkpoint-Computer -Description "ClearTool: <op>" -RestorePointType MODIFY_SETTINGS
> ```
>
> que internamente llama a la misma WMI. Documenta la decisión y los trade-offs en `.claude/specs/04-modules/restore-point-system.md`.

## Throttling de Microsoft

System Restore tiene un cooldown duro: solo permite **1 punto por cada 24 horas** salvo que se modifique:

```
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore /v SystemRestorePointCreationFrequency /t REG_DWORD /d 0
```

ClearTool aplica este tweak (con consentimiento) la primera vez que se usa, para que cada operación destructiva pueda tener su propio punto.

## Verificación

System Restore solo crea puntos si:

- El servicio `VSS` está corriendo.
- El servicio `swprv` está corriendo.
- System Protection está habilitado en C:.
- Hay al menos 300 MB de espacio reservado.

Antes de cada `create_restore_point`, ejecuta:

```rust
pub fn verify_system_restore_ready() -> Result<(), AppError> {
    // 1. Servicios VSS y swprv en estado Running.
    // 2. Get-ComputerRestorePoint funciona (si falla, System Restore deshabilitado).
    // 3. Si está deshabilitado: prompt al usuario para activarlo (`Enable-ComputerRestore -Drive C:\`).
}
```

## Log de cambios reversible

Independiente del restore point, ClearTool mantiene un log JSONL append-only:

```
%LOCALAPPDATA%\ClearTool\logs\<YYYY-MM-DD>.jsonl
```

Cada línea es un evento:

```jsonc
{
  "ts": "2026-05-09T18:23:01.123Z",
  "operation": "debloat.remove-appx-allusers",
  "target": {"type": "appx", "name": "Microsoft.BingNews"},
  "restorePointId": 42,
  "before": {"installed": true, "package": "Microsoft.BingNews_..."},
  "after":  {"installed": false},
  "reversal": {"method": "store-reinstall", "details": "..."},
  "outcome": "removed"
}
```

El frontend lee este log para mostrar la timeline reversible. Cada línea con outcome `removed`/`modified` es candidata a "deshacer" si la reversa es viable.

## Tests

- Crear punto en VM, listar -> debe aparecer.
- Restaurar -> verificar que cambios previos al punto fueron deshechos.
- Throttling: dos llamadas consecutivas con < 1 día deben crear ambos puntos (gracias al tweak).
- Si VSS está parado, `verify_system_restore_ready` debe fallar con error específico.
