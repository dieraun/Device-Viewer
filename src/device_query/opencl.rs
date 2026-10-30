//! OpenCL platform and device enumeration.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClDevice {
    pub name: String,
    pub vendor: String,
    pub vendor_id: u32,
    pub device_type: String,
    pub version: String,
    pub driver_version: String,
    pub opencl_c_version: String,
    pub max_compute_units: u32,
    pub max_clock_mhz: u32,
    pub global_memory_bytes: u64,
    pub local_memory_bytes: u64,
    pub max_work_group_size: usize,
    pub max_work_item_sizes: Vec<usize>,
    pub available: bool,
    pub compiler_available: bool,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClPlatform {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub profile: String,
    pub extensions: Vec<String>,
    pub devices: Vec<OpenClDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClInfo {
    pub available: bool,
    pub platforms: Vec<OpenClPlatform>,
    pub warnings: Vec<String>,
}

#[cfg(feature = "opencl")]
pub async fn query(cancel: &CancellationToken) -> QueryResult<OpenClInfo> {
    super::run_blocking("opencl", cancel, query_sync).await
}

#[cfg(feature = "opencl")]
fn query_sync() -> QueryResult<OpenClInfo> {
    use opencl3::device::{CL_DEVICE_TYPE_ALL, Device};
    use opencl3::platform::get_platforms;

    let platforms = get_platforms().map_err(|error| {
        super::QueryError::backend("opencl", "OpenCL platform enumeration failed")
            .with_detail(error.to_string())
    })?;

    let mut infos = Vec::with_capacity(platforms.len());
    let mut warnings = Vec::new();

    for platform in platforms {
        let device_ids = match platform.get_devices(CL_DEVICE_TYPE_ALL) {
            Ok(ids) => ids,
            Err(error) => {
                warnings.push(format!("could not list platform devices: {error}"));
                Vec::new()
            }
        };

        let mut devices = Vec::with_capacity(device_ids.len());
        for device_id in device_ids {
            let device = Device::new(device_id);
            let raw_type = device.dev_type().unwrap_or(0) as u64;
            devices.push(OpenClDevice {
                name: device
                    .name()
                    .unwrap_or_else(|_| "OpenCL device".to_string()),
                vendor: device.vendor().unwrap_or_default(),
                vendor_id: device.vendor_id().unwrap_or(0),
                device_type: device_type_text(raw_type),
                version: device.version().unwrap_or_default(),
                driver_version: device.driver_version().unwrap_or_default(),
                opencl_c_version: device.opencl_c_version().unwrap_or_default(),
                max_compute_units: device.max_compute_units().unwrap_or(0),
                max_clock_mhz: device.max_clock_frequency().unwrap_or(0),
                global_memory_bytes: device.global_mem_size().unwrap_or(0),
                local_memory_bytes: device.local_mem_size().unwrap_or(0),
                max_work_group_size: device.max_work_group_size().unwrap_or(0),
                max_work_item_sizes: device.max_work_item_sizes().unwrap_or_default(),
                available: device.available().unwrap_or(false),
                compiler_available: device.compiler_available().unwrap_or(false),
                extensions: device
                    .extensions()
                    .map(|value| value.split_whitespace().map(str::to_string).collect())
                    .unwrap_or_default(),
            });
        }

        infos.push(OpenClPlatform {
            name: platform.name().unwrap_or_default(),
            vendor: platform.vendor().unwrap_or_default(),
            version: platform.version().unwrap_or_default(),
            profile: platform.profile().unwrap_or_default(),
            extensions: platform
                .extensions()
                .map(|value| value.split_whitespace().map(str::to_string).collect())
                .unwrap_or_default(),
            devices,
        });
    }

    Ok(OpenClInfo {
        available: true,
        platforms: infos,
        warnings,
    })
}

#[cfg(feature = "opencl")]
fn device_type_text(bits: u64) -> String {
    // OpenCL device type bits from CL_DEVICE_TYPE_*.
    const DEFAULT: u64 = 1;
    const CPU: u64 = 2;
    const GPU: u64 = 4;
    const ACCELERATOR: u64 = 8;
    const CUSTOM: u64 = 16;

    let mut labels = Vec::new();
    if bits & DEFAULT != 0 {
        labels.push("Default");
    }
    if bits & CPU != 0 {
        labels.push("CPU");
    }
    if bits & GPU != 0 {
        labels.push("GPU");
    }
    if bits & ACCELERATOR != 0 {
        labels.push("Accelerator");
    }
    if bits & CUSTOM != 0 {
        labels.push("Custom");
    }
    if labels.is_empty() {
        "Unknown".to_string()
    } else {
        labels.join(" | ")
    }
}

#[cfg(not(feature = "opencl"))]
pub async fn query(_cancel: &CancellationToken) -> QueryResult<OpenClInfo> {
    Ok(OpenClInfo {
        available: false,
        platforms: Vec::new(),
        warnings: vec!["compiled without the `opencl` feature".to_string()],
    })
}
