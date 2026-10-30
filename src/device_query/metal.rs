//! Apple Metal device information (macOS only).

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetalDevice {
    pub name: String,
    pub registry_id: u64,
    pub unified_memory: bool,
    pub recommended_max_working_set_bytes: u64,
    pub max_threads_per_threadgroup: [u64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetalInfo {
    pub available: bool,
    pub devices: Vec<MetalDevice>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<MetalInfo> {
    super::run_blocking("metal", cancel, platform_query).await
}

#[cfg(all(target_os = "macos", feature = "metal"))]
fn platform_query() -> QueryResult<MetalInfo> {
    let devices: Vec<MetalDevice> = metal::Device::all()
        .iter()
        .map(|device| {
            let size = device.max_threads_per_threadgroup();
            MetalDevice {
                name: device.name().to_string(),
                registry_id: device.registry_id(),
                unified_memory: device.has_unified_memory(),
                recommended_max_working_set_bytes: device.recommended_max_working_set_size(),
                max_threads_per_threadgroup: [
                    size.width as u64,
                    size.height as u64,
                    size.depth as u64,
                ],
            }
        })
        .collect();

    let mut warnings = Vec::new();
    if devices.is_empty() {
        warnings.push("Metal reported no devices".to_string());
    }

    Ok(MetalInfo {
        available: !devices.is_empty(),
        devices,
        warnings,
    })
}

#[cfg(all(target_os = "macos", not(feature = "metal")))]
fn platform_query() -> QueryResult<MetalInfo> {
    Err(QueryError::unsupported(
        "metal",
        "binary was built without the `metal` feature",
    ))
}

#[cfg(not(target_os = "macos"))]
fn platform_query() -> QueryResult<MetalInfo> {
    Err(QueryError::unsupported(
        "metal",
        "Metal is only available on macOS",
    ))
}
