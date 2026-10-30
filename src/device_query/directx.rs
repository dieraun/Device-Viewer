//! DirectX / DXGI adapter information (Windows only).

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectXAdapter {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_memory: u64,
    pub dedicated_system_memory: u64,
    pub shared_system_memory: u64,
    pub software: bool,
    pub remote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectXInfo {
    pub available: bool,
    pub adapters: Vec<DirectXAdapter>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<DirectXInfo> {
    super::run_blocking("directx", cancel, platform_query).await
}

#[cfg(target_os = "windows")]
fn platform_query() -> QueryResult<DirectXInfo> {
    win::enumerate()
}

#[cfg(not(target_os = "windows"))]
fn platform_query() -> QueryResult<DirectXInfo> {
    Err(QueryError::unsupported(
        "directx",
        "DirectX / DXGI is only available on Windows",
    ))
}

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND, IDXGIFactory1,
    };

    pub(super) fn enumerate() -> QueryResult<DirectXInfo> {
        let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.map_err(|error| {
            QueryError::backend("directx", "cannot create DXGI factory")
                .with_detail(error.to_string())
        })?;

        let mut adapters = Vec::new();
        let mut warnings = Vec::new();

        for index in 0..64u32 {
            match unsafe { factory.EnumAdapters1(index) } {
                Ok(adapter) => match unsafe { adapter.GetDesc1() } {
                    Ok(desc) => {
                        adapters.push(DirectXAdapter {
                            name: wide_to_string(&desc.Description),
                            vendor_id: desc.VendorId,
                            device_id: desc.DeviceId,
                            dedicated_video_memory: desc.DedicatedVideoMemory as u64,
                            dedicated_system_memory: desc.DedicatedSystemMemory as u64,
                            shared_system_memory: desc.SharedSystemMemory as u64,
                            software: (desc.Flags as i32) & DXGI_ADAPTER_FLAG_SOFTWARE.0 != 0,
                            remote: desc.Flags & 1 != 0,
                        });
                    }
                    Err(error) => {
                        warnings.push(format!("cannot read adapter {index} description: {error}"))
                    }
                },
                Err(error) => {
                    if error.code() == DXGI_ERROR_NOT_FOUND {
                        break;
                    }
                    warnings.push(format!("adapter {index} enumeration failed: {error}"));
                    break;
                }
            }
        }

        if adapters.is_empty() {
            warnings.push("DXGI reported no display adapters".to_string());
        }

        Ok(DirectXInfo {
            available: true,
            adapters,
            warnings,
        })
    }

    fn wide_to_string(buffer: &[u16]) -> String {
        let len = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..len])
    }
}
