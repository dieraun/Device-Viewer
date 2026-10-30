//! Neural processing units and other processing accelerators.
//!
//! Discrete NPUs are usually PCI devices with class `0x12`. SoC-integrated NPUs
//! are not enumerable through PCI; in that case the section stays empty and
//! reports an honest warning instead of inventing data.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};
use super::run_blocking;
use super::vendor_map;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpuDevice {
    pub address: String,
    pub name: String,
    pub vendor: Option<String>,
    pub vendor_id: u16,
    pub device_id: u16,
    pub driver: Option<String>,
    pub class_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpuInfo {
    pub devices: Vec<NpuDevice>,
    pub runtime_libraries: Vec<String>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<NpuInfo> {
    run_blocking("npu", cancel, || {
        let info = pci_info::PciInfo::enumerate_pci().map_err(|error| {
            QueryError::backend("npu", "PCI enumeration failed").with_detail(error.to_string())
        })?;

        let mut devices = Vec::new();
        let mut warnings = Vec::new();

        for entry in &info {
            let Ok(device) = entry else { continue };
            let class_code = device.device_class_code().unwrap_or(0);
            if class_code != 0x12 && class_code != 0x0B {
                continue;
            }

            let vendor_id = device.vendor_id();
            let device_id = device.device_id();
            let address = device
                .location()
                .map(|location| location.to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            let name = vendor_map::pci_device_name(vendor_id, device_id)
                .map(str::to_string)
                .unwrap_or_else(|| vendor_map::describe_pci(vendor_id, device_id));

            devices.push(NpuDevice {
                address,
                name,
                vendor: vendor_map::vendor_name(vendor_id).map(str::to_string),
                vendor_id,
                device_id,
                driver: device.os_driver().ok().and_then(|driver| driver.clone()),
                class_name: vendor_map::pci_class_name(class_code)
                    .unwrap_or("Processing Accelerator")
                    .to_string(),
            });
        }

        if devices.is_empty() {
            warnings.push(
                "no PCI processing accelerator found; the NPU may be integrated into the SoC"
                    .to_string(),
            );
        }

        Ok(NpuInfo {
            devices,
            runtime_libraries: detect_runtime_libraries(),
            warnings,
        })
    })
    .await
}

fn detect_runtime_libraries() -> Vec<String> {
    let mut found = Vec::new();

    #[cfg(target_os = "linux")]
    {
        for path in [
            "/dev/accel",
            "/dev/dri",
            "/usr/lib/x86_64-linux-gnu/libopenvino.so",
            "/usr/lib/x86_64-linux-gnu/libonnxruntime.so",
            "/usr/lib/aarch64-linux-gnu/libopenvino.so",
        ] {
            if std::path::Path::new(path).exists() {
                found.push(path.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for path in [
            "/System/Library/Frameworks/CoreML.framework",
            "/System/Library/Frameworks/Metal.framework",
        ] {
            if std::path::Path::new(path).exists() {
                found.push(path.to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if std::env::var_os("PATH").is_some_and(|path| {
            std::env::split_paths(&path).any(|dir| dir.join("openvino.dll").exists())
        }) {
            found.push("openvino.dll".to_string());
        }
    }

    found
}
