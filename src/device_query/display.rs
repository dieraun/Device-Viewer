//! Monitors and display adapters.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: Option<u32>,
    pub position_x: i32,
    pub position_y: i32,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayAdapter {
    pub name: String,
    pub vendor: Option<String>,
    pub vram_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayInfo {
    pub monitors: Vec<MonitorInfo>,
    pub adapters: Vec<DisplayAdapter>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<DisplayInfo> {
    super::run_blocking("display", cancel, platform_query).await
}

#[cfg(target_os = "windows")]
fn platform_query() -> QueryResult<DisplayInfo> {
    win::enumerate()
}

#[cfg(target_os = "linux")]
fn platform_query() -> QueryResult<DisplayInfo> {
    linux::enumerate()
}

#[cfg(target_os = "macos")]
fn platform_query() -> QueryResult<DisplayInfo> {
    macos::enumerate()
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn platform_query() -> QueryResult<DisplayInfo> {
    Ok(DisplayInfo {
        warnings: vec!["display enumeration is not implemented for this platform".to_string()],
        ..DisplayInfo::default()
    })
}

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        DEVMODEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS,
        EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW, HDC,
        HMONITOR, MONITORINFOEXW,
    };
    use windows::core::{BOOL, PCWSTR};

    pub(super) fn enumerate() -> QueryResult<DisplayInfo> {
        let mut monitors: Vec<MonitorInfo> = Vec::new();
        let data = LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize);

        let enumerated = unsafe { EnumDisplayMonitors(None, None, Some(monitor_callback), data) };

        let mut warnings = Vec::new();
        if !enumerated.as_bool() {
            warnings.push("EnumDisplayMonitors returned no monitors".to_string());
        }

        let mut adapters = Vec::new();
        for index in 0..16 {
            let mut device = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };
            let ok = unsafe { EnumDisplayDevicesW(PCWSTR::null(), index, &mut device, 0) };
            if !ok.as_bool() {
                break;
            }
            if device.StateFlags.0 & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP.0 == 0 {
                continue;
            }
            let name = wide_to_string(&device.DeviceString);
            if name.is_empty()
                || adapters
                    .iter()
                    .any(|adapter: &DisplayAdapter| adapter.name == name)
            {
                continue;
            }
            adapters.push(DisplayAdapter {
                name,
                vendor: None,
                vram_bytes: None,
            });
        }

        Ok(DisplayInfo {
            monitors,
            adapters,
            warnings,
        })
    }

    unsafe extern "system" fn monitor_callback(
        monitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let monitors = unsafe { &mut *(data.0 as *mut Vec<MonitorInfo>) };
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        if unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) }.as_bool() {
            let id = wide_to_string(&info.szDevice);
            let mut mode = DEVMODEW::default();
            mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            let has_mode = unsafe {
                EnumDisplaySettingsW(
                    PCWSTR(info.szDevice.as_ptr()),
                    ENUM_CURRENT_SETTINGS,
                    &mut mode,
                )
            }
            .as_bool();

            let rect = info.monitorInfo.rcMonitor;
            monitors.push(MonitorInfo {
                name: id.clone(),
                id,
                width: if has_mode {
                    mode.dmPelsWidth
                } else {
                    rect.right.saturating_sub(rect.left) as u32
                },
                height: if has_mode {
                    mode.dmPelsHeight
                } else {
                    rect.bottom.saturating_sub(rect.top) as u32
                },
                refresh_hz: has_mode.then_some(mode.dmDisplayFrequency),
                position_x: if has_mode {
                    unsafe { mode.Anonymous1.Anonymous2.dmPosition.x }
                } else {
                    rect.left
                },
                position_y: if has_mode {
                    unsafe { mode.Anonymous1.Anonymous2.dmPosition.y }
                } else {
                    rect.top
                },
                // MONITORINFOF_PRIMARY is 1 in wingdi.h.
                primary: info.monitorInfo.dwFlags & 1 != 0,
            });
        }

        BOOL(1)
    }

    fn wide_to_string(buffer: &[u16]) -> String {
        let len = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..len])
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::path::Path;

    pub(super) fn enumerate() -> QueryResult<DisplayInfo> {
        let mut monitors = Vec::new();
        let mut warnings = Vec::new();
        let mut adapters = Vec::new();

        let base = Path::new("/sys/class/drm");
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().into_owned();
                if !file_name.starts_with("card") || !file_name.contains('-') {
                    continue;
                }
                if read_trim(&path.join("status")).as_deref() != Some("connected") {
                    continue;
                }

                let edid = std::fs::read(path.join("edid")).ok();
                let name = edid
                    .as_deref()
                    .and_then(parse_edid_name)
                    .unwrap_or_else(|| file_name.clone());
                let (width, height) = read_trim(&path.join("modes"))
                    .and_then(|modes| parse_mode(&modes))
                    .unwrap_or((0, 0));

                monitors.push(MonitorInfo {
                    id: file_name.clone(),
                    name,
                    width,
                    height,
                    refresh_hz: None,
                    position_x: 0,
                    position_y: 0,
                    primary: monitors.is_empty(),
                });
            }
        } else {
            warnings.push("cannot read /sys/class/drm".to_string());
        }

        let card_path = Path::new("/sys/class/drm/card0/device");
        let vendor_id = read_hex(&card_path.join("vendor"));
        let device_id = read_hex(&card_path.join("device"));
        if let (Some(vendor), Some(device)) = (vendor_id, device_id) {
            adapters.push(DisplayAdapter {
                name: super::super::vendor_map::describe_pci(vendor, device),
                vendor: super::super::vendor_map::vendor_name(vendor).map(str::to_string),
                vram_bytes: None,
            });
        }

        Ok(DisplayInfo {
            monitors,
            adapters,
            warnings,
        })
    }

    fn read_trim(path: &Path) -> Option<String> {
        std::fs::read_to_string(path)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn read_hex(path: &Path) -> Option<u16> {
        let value = read_trim(path)?;
        u16::from_str_radix(value.trim_start_matches("0x"), 16).ok()
    }

    fn parse_mode(modes: &str) -> Option<(u32, u32)> {
        let first = modes.lines().next()?;
        let (width, height) = first.split_once('x')?;
        Some((width.parse().ok()?, height.parse().ok()?))
    }

    fn parse_edid_name(edid: &[u8]) -> Option<String> {
        if edid.len() < 128 || &edid[0..8] != [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
            return None;
        }
        for index in 0..4 {
            let offset = 54 + index * 18;
            if offset + 18 > edid.len() {
                break;
            }
            if edid[offset + 3] == 0xFC {
                let bytes: Vec<u8> = edid[offset + 5..offset + 18]
                    .iter()
                    .copied()
                    .take_while(|byte| *byte != 0x0A && *byte != 0)
                    .collect();
                let name = String::from_utf8_lossy(&bytes).trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub(super) fn enumerate() -> QueryResult<DisplayInfo> {
        let output = std::process::Command::new("system_profiler")
            .args(["SPDisplaysDataType", "-json"])
            .output()
            .map_err(|error| {
                super::super::QueryError::backend("display", "system_profiler failed to start")
                    .with_detail(error.to_string())
            })?;

        let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| {
            super::super::QueryError::parse("display", "invalid system_profiler JSON")
                .with_detail(error.to_string())
        })?;

        let mut monitors = Vec::new();
        let mut adapters = Vec::new();
        if let Some(items) = value
            .get("SPDisplaysDataType")
            .and_then(|value| value.as_array())
        {
            for item in items {
                let adapter_name = item
                    .get("_name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Display adapter")
                    .to_string();
                let resolution = item
                    .get("_spdisplays_resolution")
                    .and_then(|value| value.as_str())
                    .and_then(parse_resolution);
                adapters.push(DisplayAdapter {
                    name: adapter_name.clone(),
                    vendor: item
                        .get("spdisplays_vendor")
                        .and_then(|value| value.as_str())
                        .map(str::to_string),
                    vram_bytes: None,
                });
                let (width, height) = resolution.unwrap_or((0, 0));
                monitors.push(MonitorInfo {
                    id: adapter_name.clone(),
                    name: adapter_name,
                    width,
                    height,
                    refresh_hz: None,
                    position_x: 0,
                    position_y: 0,
                    primary: monitors.is_empty(),
                });
            }
        }

        Ok(DisplayInfo {
            monitors,
            adapters,
            warnings: Vec::new(),
        })
    }

    fn parse_resolution(value: &str) -> Option<(u32, u32)> {
        let cleaned = value.replace("Retina", " ").replace("UI Looks like", " ");
        let token = cleaned.split_whitespace().find(|part| part.contains('x'))?;
        let (width, height) = token.split_once('x')?;
        Some((width.parse().ok()?, height.parse().ok()?))
    }
}
