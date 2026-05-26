// platform/gpu.rs — detección de GPUs y métricas en vivo.
//
// En Windows:
//   1. `Get-CimInstance Win32_VideoController` → nombre, AdapterRAM total.
//   2. `Get-Counter '\GPU Engine(*)\Utilization Percentage'` → uso% por engine.
//   3. `Get-CimInstance -Namespace root/wmi MSAcpi_ThermalZoneTemperature` → temperatura.
//
// En Linux:
//   Intenta leer /sys/class/drm/ y /sys/class/hwmon/ para info básica de GPU.
//   Devuelve vacío si no encuentra datos (best-effort).

use crate::models::telemetry::GpuInfo;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use crate::platform::powershell;
    use serde::Deserialize;

    const PS_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue';
$controllers = Get-CimInstance Win32_VideoController -ErrorAction SilentlyContinue | ForEach-Object {
    [PSCustomObject]@{
        index = $_.DeviceID;
        name = $_.Name;
        ramBytes = if ($_.AdapterRAM -ne $null) { [int64]$_.AdapterRAM } else { 0 }
    }
};

$counters = @();
try {
    $samples = (Get-Counter '\GPU Engine(*)\Utilization Percentage' -ErrorAction SilentlyContinue).CounterSamples;
    if ($samples) {
        $byLuid = @{};
        foreach ($s in $samples) {
            if ($s.Path -match 'luid_0x([0-9a-f]+)_0x([0-9a-f]+)') {
                $key = "$($Matches[1])_$($Matches[2])";
                if (-not $byLuid.ContainsKey($key)) { $byLuid[$key] = 0.0 }
                $byLuid[$key] += [double]$s.CookedValue;
            }
        }
        foreach ($k in $byLuid.Keys) {
            $counters += [PSCustomObject]@{ luid = $k; percent = [double]$byLuid[$k] };
        }
    }
} catch {}

[PSCustomObject]@{
    controllers = $controllers;
    counters = $counters;
} | ConvertTo-Json -Depth 5 -Compress
"#;

    #[derive(Debug, Deserialize)]
    struct GpuPayload {
        #[serde(default)]
        controllers: Vec<Controller>,
        #[serde(default)]
        counters: Vec<Counter>,
    }

    #[derive(Debug, Deserialize)]
    struct Controller {
        #[serde(default)]
        name: Option<String>,
        #[serde(default, rename = "ramBytes")]
        ram_bytes: Option<i64>,
    }

    #[derive(Debug, Deserialize)]
    struct Counter {
        #[serde(default)]
        percent: Option<f64>,
    }

    pub fn snapshot() -> Vec<GpuInfo> {
        let output = powershell::run_script(PS_SCRIPT);
        let Ok(out) = output else { return Vec::new() };
        if !out.status.success() {
            return Vec::new();
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let payload: GpuPayload = match serde_json::from_str(trimmed) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        let total_percent: f64 = payload.counters.iter().filter_map(|c| c.percent).sum();
        let n_gpus = payload.controllers.len().max(1);

        payload
            .controllers
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let name = c.name.unwrap_or_else(|| "Unknown".to_string());
                let vendor = guess_vendor(&name);
                let usage = if total_percent > 0.0 {
                    Some((total_percent / n_gpus as f64) as f32)
                } else {
                    None
                };
                let memory_total = c.ram_bytes.map(|n| n as u64);
                GpuInfo {
                    index: i as u32,
                    name,
                    vendor,
                    usage_percent: usage,
                    memory_used_bytes: None,
                    memory_total_bytes: memory_total,
                    temp_celsius: None,
                }
            })
            .collect()
    }

    fn guess_vendor(name: &str) -> Option<String> {
        let n = name.to_lowercase();
        if n.contains("nvidia") || n.contains("geforce") || n.contains("rtx") || n.contains("gtx") {
            Some("NVIDIA".to_string())
        } else if n.contains("amd") || n.contains("radeon") {
            Some("AMD".to_string())
        } else if n.contains("intel") || n.contains("iris") || n.contains("uhd") {
            Some("Intel".to_string())
        } else {
            None
        }
    }
}

#[cfg(windows)]
pub use windows_impl::snapshot;

#[cfg(not(windows))]
pub fn snapshot() -> Vec<GpuInfo> {
    // Linux: intentar leer info de GPU desde sysfs
    let mut gpus = Vec::new();

    // Intentar detectar GPUs via /sys/class/drm/card*/device/
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        let mut index = 0u32;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("card") && !name_str.contains('-') {
                let device_path = entry.path().join("device");

                // Leer nombre del driver
                let vendor = if let Ok(link) = std::fs::read_link(device_path.join("driver")) {
                    let driver_name = link.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if driver_name.contains("nvidia") {
                        Some("NVIDIA".to_string())
                    } else if driver_name.contains("amdgpu") || driver_name.contains("radeon") {
                        Some("AMD".to_string())
                    } else if driver_name.contains("i915") || driver_name.contains("xe") {
                        Some("Intel".to_string())
                    } else {
                        Some(driver_name)
                    }
                } else {
                    None
                };

                // Intentar leer VRAM total desde device/mem_info_vram_total
                let memory_total = std::fs::read_to_string(device_path.join("mem_info_vram_total"))
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok());

                let name = vendor.clone().unwrap_or_else(|| "Unknown".to_string());

                gpus.push(GpuInfo {
                    index,
                    name,
                    vendor,
                    usage_percent: None,
                    memory_used_bytes: None,
                    memory_total_bytes: memory_total,
                    temp_celsius: None,
                });
                index += 1;
            }
        }
    }

    gpus
}
