// platform/services.rs — adaptador al Service Control Manager.

use crate::core::{AppError, AppResult};

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Services::*;

    const START_BOOT: u32 = 0;
    const START_SYSTEM: u32 = 1;
    const START_AUTO: u32 = 2;
    const START_MANUAL: u32 = 3;
    const START_DISABLED: u32 = 4;

    #[derive(Debug, Clone)]
    pub struct ServiceRuntime {
        pub name: String,
        pub display_name: String,
        pub state: String,
        pub start_type: String,
        pub description: Option<String>,
    }

    pub fn list_all() -> AppResult<Vec<ServiceRuntime>> {
        unsafe {
            let scm = OpenSCManagerW(
                PCWSTR::null(),
                PCWSTR::null(),
                SC_MANAGER_ENUMERATE_SERVICE,
            )
            .map_err(|e| AppError::Services(format!("OpenSCManagerW: {:?}", e)))?;

            let mut bytes_needed = 0u32;
            let mut services_returned = 0u32;
            let mut resume_handle = 0u32;
            let mut buf: Vec<u8> = vec![0; 1024 * 1024];

            loop {
                let r = EnumServicesStatusExW(
                    scm,
                    SC_ENUM_PROCESS_INFO,
                    SERVICE_WIN32,
                    SERVICE_STATE_ALL,
                    Some(&mut buf),
                    &mut bytes_needed,
                    &mut services_returned,
                    Some(&mut resume_handle),
                    PCWSTR::null(),
                );

                if r.is_ok() {
                    break;
                }
                let need = (bytes_needed as usize).max(buf.len() * 2);
                if need > 16 * 1024 * 1024 {
                    let _ = CloseServiceHandle(scm);
                    return Err(AppError::Services("buffer enum > 16MB".into()));
                }
                buf.resize(need, 0);
            }

            let services_ptr = buf.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW;
            let services_slice =
                std::slice::from_raw_parts(services_ptr, services_returned as usize);

            let mut out = Vec::with_capacity(services_returned as usize);
            for s in services_slice {
                let name = pcwstr_to_string(s.lpServiceName);
                let display = pcwstr_to_string(s.lpDisplayName);
                let state = state_to_str(s.ServiceStatusProcess.dwCurrentState);
                let start_type =
                    query_start_type(scm, &name).unwrap_or_else(|_| "Unknown".to_string());
                let description = query_description(scm, &name).ok();

                out.push(ServiceRuntime {
                    name,
                    display_name: display,
                    state,
                    start_type,
                    description,
                });
            }

            let _ = CloseServiceHandle(scm);
            Ok(out)
        }
    }

    unsafe fn pcwstr_to_string(p: PWSTR) -> String {
        if p.is_null() {
            return String::new();
        }
        let mut len = 0;
        while *p.0.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(p.0, len);
        String::from_utf16_lossy(slice)
    }

    fn state_to_str(s: SERVICE_STATUS_CURRENT_STATE) -> String {
        match s.0 {
            1 => "Stopped",
            2 => "StartPending",
            3 => "StopPending",
            4 => "Running",
            5 => "ContinuePending",
            6 => "PausePending",
            7 => "Paused",
            _ => "Unknown",
        }
        .to_string()
    }

    unsafe fn query_start_type(scm: SC_HANDLE, name: &str) -> AppResult<String> {
        let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let h = OpenServiceW(
            scm,
            PCWSTR(wname.as_ptr()),
            SERVICE_QUERY_CONFIG,
        )
        .map_err(|e| {
            AppError::Services(format!("OpenServiceW({}): {:?}", name, e))
        })?;

        let mut bytes_needed = 0u32;
        let mut buf: Vec<u8> = vec![0; 4096];
        let r = QueryServiceConfigW(
            h,
            Some(buf.as_mut_ptr() as *mut _),
            buf.len() as u32,
            &mut bytes_needed,
        );
        if r.is_err() {
            buf.resize(bytes_needed as usize, 0);
            let _ = QueryServiceConfigW(
                h,
                Some(buf.as_mut_ptr() as *mut _),
                buf.len() as u32,
                &mut bytes_needed,
            );
        }

        let cfg = &*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);
        let st = start_type_to_str(cfg.dwStartType.0);

        let _ = CloseServiceHandle(h);
        Ok(st)
    }

    fn start_type_to_str(start_type: u32) -> String {
        match start_type {
            0 => "Boot",
            1 => "System",
            2 => "Automatic",
            3 => "Manual",
            4 => "Disabled",
            _ => "Unknown",
        }
        .to_string()
    }

    unsafe fn query_description(scm: SC_HANDLE, name: &str) -> AppResult<String> {
        let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let h = OpenServiceW(
            scm,
            PCWSTR(wname.as_ptr()),
            SERVICE_QUERY_CONFIG,
        )
        .map_err(|e| AppError::Services(format!("OpenServiceW desc: {:?}", e)))?;

        let mut bytes_needed = 0u32;
        let mut buf: Vec<u8> = vec![0; 4096];
        let r = QueryServiceConfig2W(
            h,
            SERVICE_CONFIG_DESCRIPTION,
            Some(&mut buf),
            &mut bytes_needed,
        );
        let _ = r;

        let desc = &*(buf.as_ptr() as *const SERVICE_DESCRIPTIONW);
        let s = if desc.lpDescription.is_null() {
            String::new()
        } else {
            let mut len = 0;
            while *desc.lpDescription.0.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(desc.lpDescription.0, len);
            String::from_utf16_lossy(slice)
        };

        let _ = CloseServiceHandle(h);
        Ok(s)
    }

    pub fn set_start_type(name: &str, start_type: &str) -> AppResult<()> {
        let st = match start_type {
            "Boot" => START_BOOT,
            "System" => START_SYSTEM,
            "Automatic" | "AutomaticDelayed" => START_AUTO,
            "Manual" => START_MANUAL,
            "Disabled" => START_DISABLED,
            _ => {
                return Err(AppError::Services(format!(
                    "start_type inválido: {}",
                    start_type
                )))
            }
        };

        unsafe {
            let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
                .map_err(|e| AppError::Services(format!("OpenSCManager: {:?}", e)))?;
            let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
            let h = OpenServiceW(
                scm,
                PCWSTR(wname.as_ptr()),
                SERVICE_CHANGE_CONFIG,
            )
            .map_err(|e| {
                AppError::Services(format!("OpenService change: {:?}", e))
            })?;

            let r = ChangeServiceConfigW(
                h,
                ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
                SERVICE_START_TYPE(st),
                SERVICE_ERROR(SERVICE_NO_CHANGE),
                PCWSTR::null(),
                PCWSTR::null(),
                None,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
            );

            let _ = CloseServiceHandle(h);
            let _ = CloseServiceHandle(scm);

            r.map_err(|e| {
                if e.code() == E_ACCESSDENIED {
                    AppError::Permission(format!(
                        "ChangeServiceConfig {}: protected",
                        name
                    ))
                } else {
                    AppError::Services(format!("ChangeServiceConfig: {:?}", e))
                }
            })
        }
    }

    pub fn start(name: &str) -> AppResult<()> {
        unsafe {
            let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
                .map_err(|e| AppError::Services(format!("OpenSCManager: {:?}", e)))?;
            let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
            let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_START)
                .map_err(|e| {
                    AppError::Services(format!("OpenService start: {:?}", e))
                })?;
            let r = StartServiceW(h, None);
            let _ = CloseServiceHandle(h);
            let _ = CloseServiceHandle(scm);
            r.map_err(|e| AppError::Services(format!("StartServiceW: {:?}", e)))
        }
    }

    pub fn stop(name: &str) -> AppResult<()> {
        unsafe {
            let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
                .map_err(|e| AppError::Services(format!("OpenSCManager: {:?}", e)))?;
            let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
            let h = OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_STOP)
                .map_err(|e| AppError::Services(format!("OpenService stop: {:?}", e)))?;
            let mut status = SERVICE_STATUS::default();
            let r = ControlService(h, SERVICE_CONTROL_STOP, &mut status);
            let _ = CloseServiceHandle(h);
            let _ = CloseServiceHandle(scm);
            r.map_err(|e| AppError::Services(format!("ControlService stop: {:?}", e)))
        }
    }

    pub fn dependencies_of(name: &str) -> AppResult<Vec<String>> {
        unsafe {
            let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
                .map_err(|e| AppError::Services(format!("OpenSCManager deps: {:?}", e)))?;
            let wname: Vec<u16> = name.encode_utf16().chain([0]).collect();
            let h = OpenServiceW(
                scm,
                PCWSTR(wname.as_ptr()),
                SERVICE_ENUMERATE_DEPENDENTS,
            )
            .map_err(|e| {
                AppError::Services(format!("OpenService dependents: {:?}", e))
            })?;

            let mut bytes_needed = 0u32;
            let mut services_returned = 0u32;
            let mut buf: Vec<u8> = vec![0; 4096];
            loop {
                let r = EnumDependentServicesW(
                    h,
                    SERVICE_STATE_ALL,
                    Some(buf.as_mut_ptr() as *mut ENUM_SERVICE_STATUSW),
                    buf.len() as u32,
                    &mut bytes_needed,
                    &mut services_returned,
                );
                if r.is_ok() {
                    break;
                }
                let need = (bytes_needed as usize).max(buf.len() * 2);
                if need > 1024 * 1024 {
                    break;
                }
                buf.resize(need, 0);
            }

            let ptr = buf.as_ptr() as *const ENUM_SERVICE_STATUSW;
            let slice = std::slice::from_raw_parts(ptr, services_returned as usize);
            let names = slice
                .iter()
                .map(|s| pcwstr_to_string(s.lpServiceName))
                .collect();

            let _ = CloseServiceHandle(h);
            let _ = CloseServiceHandle(scm);
            Ok(names)
        }
    }
}

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
pub fn list_all() -> AppResult<Vec<ServiceRuntime>> {
    Err(AppError::External(
        "Gestión de servicios Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub struct ServiceRuntime {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_type: String,
    pub description: Option<String>,
}

#[cfg(not(windows))]
pub fn set_start_type(_name: &str, _start_type: &str) -> AppResult<()> {
    Err(AppError::External(
        "Gestión de servicios Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn start(_name: &str) -> AppResult<()> {
    Err(AppError::External(
        "Gestión de servicios Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn stop(_name: &str) -> AppResult<()> {
    Err(AppError::External(
        "Gestión de servicios Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn dependencies_of(_name: &str) -> AppResult<Vec<String>> {
    Err(AppError::External(
        "Gestión de servicios Windows no disponible en esta plataforma".to_string(),
    ))
}
