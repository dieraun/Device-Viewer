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
    /// 适配器能创建的最高 Direct3D 11 功能级别，例如 "11.1"。
    #[serde(default)]
    pub feature_level: Option<String>,
    /// 能否创建 Direct3D 12 设备；None 表示探测本身失败。
    #[serde(default)]
    pub d3d12_supported: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectXInfo {
    pub available: bool,
    /// DXGI 运行时版本，例如 "1.6"。
    #[serde(default)]
    pub dxgi_version: Option<String>,
    /// 注册表 HKLM\SOFTWARE\Microsoft\DirectX 的 Version 值。
    #[serde(default)]
    pub runtime_version: Option<String>,
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
    // 非 Windows 平台没有 DirectX 属于预期，不是故障。
    Ok(DirectXInfo::default())
}

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use windows::Win32::Foundation::{ERROR_SUCCESS, HMODULE};
    use windows::Win32::Graphics::Direct3D::{
        D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_9_1, D3D_FEATURE_LEVEL_9_3,
        D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_11_1,
    };
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11CreateDevice,
    };
    use windows::Win32::Graphics::Direct3D12::D3D12CreateDevice;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND, IDXGIAdapter,
        IDXGIAdapter1, IDXGIFactory1, IDXGIFactory2, IDXGIFactory3, IDXGIFactory4, IDXGIFactory5,
        IDXGIFactory6, IDXGIFactory7,
    };
    use windows::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, REG_VALUE_TYPE, RRF_RT_REG_SZ, RegGetValueW,
    };
    use windows::core::{IUnknown, Interface, PCWSTR};

    /// 依次尝试的功能级别，从高到低；D3D11 会选其中支持的最高一级。
    const LEVELS: [D3D_FEATURE_LEVEL; 6] = [
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
        D3D_FEATURE_LEVEL_9_3,
        D3D_FEATURE_LEVEL_9_1,
    ];

    pub(super) fn enumerate() -> QueryResult<DirectXInfo> {
        // SAFETY: CreateDXGIFactory1 无参数，返回 COM 接口指针并按 HRESULT
        // 报告失败；windows crate 负责把失败折成 Result。
        let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.map_err(|error| {
            QueryError::backend("directx", "cannot create DXGI factory")
                .with_detail(error.to_string())
        })?;

        let dxgi_version = Some(dxgi_version(&factory));
        let runtime_version = runtime_version();

        let mut adapters = Vec::new();
        let mut warnings = Vec::new();

        for index in 0..64u32 {
            // SAFETY: factory 是本函数新建的有效 COM 对象，调用期间一直存活。
            match unsafe { factory.EnumAdapters1(index) } {
                // SAFETY: adapter 由同一工厂的 EnumAdapters1 返回，调用期间工厂与 adapter 都存活。
                Ok(adapter) => match unsafe { adapter.GetDesc1() } {
                    Ok(desc) => {
                        let (feature_level, d3d12_supported) = probe(&adapter);
                        adapters.push(DirectXAdapter {
                            name: wide_to_string(&desc.Description),
                            vendor_id: desc.VendorId,
                            device_id: desc.DeviceId,
                            dedicated_video_memory: desc.DedicatedVideoMemory as u64,
                            dedicated_system_memory: desc.DedicatedSystemMemory as u64,
                            shared_system_memory: desc.SharedSystemMemory as u64,
                            software: (desc.Flags as i32) & DXGI_ADAPTER_FLAG_SOFTWARE.0 != 0,
                            remote: desc.Flags & 1 != 0,
                            feature_level,
                            d3d12_supported,
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
            dxgi_version,
            runtime_version,
            adapters,
            warnings,
        })
    }

    /// 探测适配器的 D3D11 功能级别与 D3D12 支持；探测失败不当作警告，
    /// 只是这一项留空，由 UI 显示为未知。
    fn probe(adapter: &IDXGIAdapter1) -> (Option<String>, Option<bool>) {
        (probe_d3d11(adapter), probe_d3d12(adapter))
    }

    fn probe_d3d11(adapter: &IDXGIAdapter1) -> Option<String> {
        let adapter = adapter.cast::<IDXGIAdapter>().ok()?;
        let mut level = D3D_FEATURE_LEVEL::default();
        // SAFETY: adapter 是有效的 IDXGIAdapter，LEVELS 是静态数组且长度作为
        // 元素个数传入；只写入本函数的栈变量 level，不交出所有权。
        let result = unsafe {
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(&LEVELS),
                D3D11_SDK_VERSION,
                None,
                Some(&mut level),
                None,
            )
        };
        result.ok().map(|()| feature_level_label(level))
    }

    fn probe_d3d12(adapter: &IDXGIAdapter1) -> Option<bool> {
        let unknown: IUnknown = adapter.cast().ok()?;
        // 必须真的拿到设备指针：D3D12CreateDevice 在功能级别不足时可能返回
        // S_FALSE，只看 HRESULT 会把它误判成支持。
        let mut device: Option<IUnknown> = None;
        // SAFETY: unknown 是有效的适配器 IUnknown；device 是本函数的栈变量，
        // 调用期间一直有效，成功时由 D3D12CreateDevice 写入新设备。
        let result = unsafe {
            D3D12CreateDevice(
                &unknown,
                D3D_FEATURE_LEVEL_11_0,
                &mut device as *mut Option<IUnknown>,
            )
        };
        Some(result.is_ok() && device.is_some())
    }

    /// 依次尝试 DXGI 1.7 → 1.1 的工厂接口，命中即说明运行时提供了该版本。
    fn dxgi_version(factory: &IDXGIFactory1) -> String {
        let label = if factory.cast::<IDXGIFactory7>().is_ok() {
            "1.7"
        } else if factory.cast::<IDXGIFactory6>().is_ok() {
            "1.6"
        } else if factory.cast::<IDXGIFactory5>().is_ok() {
            "1.5"
        } else if factory.cast::<IDXGIFactory4>().is_ok() {
            "1.4"
        } else if factory.cast::<IDXGIFactory3>().is_ok() {
            "1.3"
        } else if factory.cast::<IDXGIFactory2>().is_ok() {
            "1.2"
        } else {
            "1.1"
        };
        label.to_string()
    }

    fn feature_level_label(level: D3D_FEATURE_LEVEL) -> String {
        match level.0 {
            0xb100 => "11.1".to_string(),
            0xb000 => "11.0".to_string(),
            0xa100 => "10.1".to_string(),
            0xa000 => "10.0".to_string(),
            0x9300 => "9.3".to_string(),
            0x9200 => "9.2".to_string(),
            0x9100 => "9.1".to_string(),
            other => format!("0x{other:04x}"),
        }
    }

    /// 读取 HKLM\SOFTWARE\Microsoft\DirectX 的 Version 值，失败返回 None。
    fn runtime_version() -> Option<String> {
        let subkey: Vec<u16> = "SOFTWARE\\Microsoft\\DirectX\0".encode_utf16().collect();
        let value: Vec<u16> = "Version\0".encode_utf16().collect();
        let mut buffer = [0u16; 128];
        let mut size = (buffer.len() * 2) as u32;
        let mut kind = REG_VALUE_TYPE::default();
        // SAFETY: 两个字符串都以 NUL 结尾且在本函数内存活；buffer 与 size
        // 描述同一块可写内存，RegGetValueW 只在这块范围内写入。
        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_SZ,
                Some(&mut kind),
                Some(buffer.as_mut_ptr() as *mut core::ffi::c_void),
                Some(&mut size),
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let chars = (size as usize / 2).min(buffer.len());
        let len = buffer[..chars]
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(chars);
        let text = String::from_utf16_lossy(&buffer[..len]);
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn wide_to_string(buffer: &[u16]) -> String {
        let len = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..len])
    }
}
