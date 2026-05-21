// platform/elevation.rs — detección y auto-relanzamiento de elevación UAC.
//
// `is_elevated()` usa OpenProcessToken + GetTokenInformation con TOKEN_ELEVATION.
// Es 1000x más rápido y fiable que `net session` (que depende del idioma del SO
// y del estado del servicio "Server").
//
// `relaunch_as_admin()` se invoca al arrancar si el manifest pide elevación
// pero el proceso no la tiene: lanza una copia del exe con verbo "runas"
// (UAC) y sale.

#[cfg(target_os = "windows")]
pub fn is_elevated() -> bool {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        )
        .is_ok();

        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_elevated() -> bool {
    // En no-Windows no aplica el modelo UAC. Asumimos elevado para tests.
    true
}

/// Intenta relanzar como administrador. Retorna `true` si se relanzó
/// (el proceso hijo toma el control). Retorna `false` si el usuario
/// canceló UAC o si falló el relaunch.
#[cfg(target_os = "windows")]
pub fn try_relaunch_as_admin() -> bool {
    if is_elevated() {
        return false;
    }

    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let exe_wide: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let args: Vec<u16> = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let h = ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR(args.as_ptr()),
            None,
            SW_NORMAL,
        );
        h.0 as isize > 32
    }
}

/// Legacy alias para compatibilidad.
#[deprecated(since = "0.2.0", note = "use try_relaunch_as_admin() instead")]
#[cfg(target_os = "windows")]
pub fn relaunch_as_admin_if_needed() -> bool {
    try_relaunch_as_admin()
}

#[cfg(not(target_os = "windows"))]
pub fn try_relaunch_as_admin() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
#[deprecated]
pub fn relaunch_as_admin_if_needed() -> bool {
    false
}

// Helpers de encoding para el path/args en relanzamiento.
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
