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
    /// False when the platform cannot tell whether this monitor is the primary
    /// one; the UI then shows "unknown" instead of inventing an answer.
    #[serde(default = "default_true")]
    pub primary_known: bool,
    /// False when the desktop layout is not exposed; `position_x/y` are then
    /// meaningless zeros and the UI hides them.
    #[serde(default = "default_true")]
    pub position_known: bool,
}

const fn default_true() -> bool {
    true
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

        // SAFETY: data 指向上面刚声明的 Vec，枚举期间同步回调且向量一直有效。
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
            // SAFETY: device 的 cb 已按结构体大小初始化，API 只写这个结构体。
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

    /// 枚举回调：只负责捕获 panic 并把结果折成 BOOL。
    ///
    /// 回调跑在 FFI 边界上，panic 跨边界即未定义行为；因此任何 panic 都在
    /// 这里被吞掉并返回 BOOL(0)，让系统停止枚举，已收集的显示器仍然保留。
    ///
    /// # Safety
    ///
    /// 只能由 `EnumDisplayMonitors` 在枚举期间同步调用；`data` 是调用方
    /// 持有的 `&mut Vec<MonitorInfo>` 地址，仅在本次枚举期间有效。
    // SAFETY: 见上方 # Safety：只在枚举期间被同步调用一次，data 指向调用方
    // 持有的向量；函数体内部再用 catch_unwind 挡住 panic 跨 FFI 边界。
    unsafe extern "system" fn monitor_callback(
        monitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let collected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            collect_monitor(monitor, data);
        }));
        match collected {
            Ok(()) => BOOL(1),
            Err(_) => BOOL(0),
        }
    }

    /// 把单个显示器写进回调传入的向量；由 `monitor_callback` 调用。
    fn collect_monitor(monitor: HMONITOR, data: LPARAM) {
        // SAFETY: 见 `monitor_callback` 的 # Safety 一节；回调期间这条
        // `&mut Vec` 借用是唯一的可变借用，且向量生命周期覆盖整个枚举。
        let monitors = unsafe { &mut *(data.0 as *mut Vec<MonitorInfo>) };
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        // SAFETY: info.monitorInfo.cbSize 已初始化，monitor 是回调参数。
        if unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) }.as_bool() {
            let id = wide_to_string(&info.szDevice);
            let mut mode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            // SAFETY: mode.dmSize 已初始化，设备名来自刚查询到的 szDevice。
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
                // 0/1 Hz 是“默认模式”的占位值，不能当成真实刷新率显示。
                refresh_hz: has_mode
                    .then_some(mode.dmDisplayFrequency)
                    .filter(|hz| *hz > 1),
                position_x: if has_mode {
                    // SAFETY: 只有挂起成功时 has_mode 才为真，此时 dmPosition
                    // 属于已填充的 DEVMODEW；该联合体在此模式下只读坐标。
                    unsafe { mode.Anonymous1.Anonymous2.dmPosition.x }
                } else {
                    rect.left
                },
                position_y: if has_mode {
                    // SAFETY: 同上，与 position_x 读取同一已初始化的联合体。
                    unsafe { mode.Anonymous1.Anonymous2.dmPosition.y }
                } else {
                    rect.top
                },
                // MONITORINFOF_PRIMARY is 1 in wingdi.h.
                primary: info.monitorInfo.dwFlags & 1 != 0,
                primary_known: true,
                position_known: true,
            });
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

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;

    pub(super) fn enumerate() -> QueryResult<DisplayInfo> {
        let mut monitors = Vec::new();
        let mut warnings = Vec::new();
        let mut adapters = Vec::new();

        // xrandr 是用户态唯一能拿到“主屏”标记与多屏相对位置的来源。
        // 拿不到时宁可标注未知，也不把第一个枚举到的接口当成主屏。
        let layout = xrandr_layout();
        if layout.is_none() {
            warnings.push(
                "xrandr is unavailable; primary monitor and desktop layout are unknown".to_string(),
            );
        }

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
                let placed = layout
                    .as_ref()
                    .and_then(|table| table.get(connector_of(&file_name)).copied());

                monitors.push(MonitorInfo {
                    id: file_name.clone(),
                    name,
                    width,
                    height,
                    refresh_hz: None,
                    position_x: placed.map(|entry| entry.x).unwrap_or(0),
                    position_y: placed.map(|entry| entry.y).unwrap_or(0),
                    primary: placed.is_some_and(|entry| entry.primary),
                    primary_known: placed.is_some(),
                    position_known: placed.is_some(),
                });
            }
        } else {
            warnings.push("cannot read /sys/class/drm".to_string());
        }

        // 一块显卡对应一个 cardN 目录；只读 card0 会漏掉双显卡机器。
        if let Ok(entries) = std::fs::read_dir(base) {
            let mut seen: Vec<String> = Vec::new();
            for entry in entries.flatten() {
                let card = entry.file_name().to_string_lossy().into_owned();
                if !is_card_name(&card) || seen.iter().any(|seen| seen == &card) {
                    continue;
                }
                seen.push(card);
                let device = entry.path().join("device");
                let vendor_id = read_hex(&device.join("vendor"));
                let device_id = read_hex(&device.join("device"));
                if let (Some(vendor), Some(device)) = (vendor_id, device_id) {
                    adapters.push(DisplayAdapter {
                        name: super::super::vendor_map::describe_pci(vendor, device),
                        vendor: super::super::vendor_map::vendor_name(vendor).map(str::to_string),
                        vram_bytes: None,
                    });
                }
            }
        }

        Ok(DisplayInfo {
            monitors,
            adapters,
            warnings,
        })
    }

    #[derive(Clone, Copy)]
    struct LayoutEntry {
        primary: bool,
        x: i32,
        y: i32,
    }

    fn xrandr_layout() -> Option<BTreeMap<String, LayoutEntry>> {
        let output = std::process::Command::new("xrandr")
            .arg("--query")
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(parse_xrandr(&String::from_utf8_lossy(&output.stdout)))
    }

    fn parse_xrandr(text: &str) -> BTreeMap<String, LayoutEntry> {
        let mut table = BTreeMap::new();
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let Some(connector) = parts.next() else {
                continue;
            };
            if parts.next() != Some("connected") {
                continue;
            }
            let mut primary = false;
            let mut placement = None;
            for part in parts {
                if part == "primary" {
                    primary = true;
                } else if part.contains('x') && part.contains('+') {
                    placement = parse_geometry(part);
                }
            }
            if let Some((x, y)) = placement {
                table.insert(connector.to_string(), LayoutEntry { primary, x, y });
            }
        }
        table
    }

    fn parse_geometry(value: &str) -> Option<(i32, i32)> {
        let (_, rest) = value.split_once('+')?;
        let (x, y) = rest.split_once('+')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    }

    /// Turns `card0-DP-1` into the `DP-1` name xrandr prints.
    fn connector_of(file_name: &str) -> &str {
        file_name
            .split_once('-')
            .map(|(_, name)| name)
            .unwrap_or(file_name)
    }

    fn is_card_name(name: &str) -> bool {
        name.strip_prefix("card").is_some_and(|number| {
            !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
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
        const HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
        if edid.len() < 128 || edid[0..8] != HEADER {
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
                // system_profiler 只在部分版本给出主屏标记，也不给桌面坐标；
                // 拿不到就标为未知，而不是把枚举到的第一个显示器当成主屏。
                let main = item
                    .get("spdisplays_main")
                    .or_else(|| item.get("_spdisplays_main"))
                    .and_then(|value| value.as_str())
                    .map(|value| value.eq_ignore_ascii_case("spdisplays_yes"));
                monitors.push(MonitorInfo {
                    id: adapter_name.clone(),
                    name: adapter_name,
                    width,
                    height,
                    refresh_hz: None,
                    position_x: 0,
                    position_y: 0,
                    primary: main.unwrap_or(false),
                    primary_known: main.is_some(),
                    position_known: false,
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
