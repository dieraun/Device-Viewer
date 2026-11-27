//! USB device tree.
//!
//! Windows uses SetupAPI, Linux reads sysfs and macOS parses
//! `system_profiler SPUSBDataType -json`.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;
use super::vendor_map;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsbDevice {
    pub bus: Option<u8>,
    pub address: Option<u8>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub serial: Option<String>,
    pub class: Option<String>,
    pub speed_mbps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsbInfo {
    pub devices: Vec<UsbDevice>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<UsbInfo> {
    let mut info = super::run_blocking("usb", cancel, platform_query).await?;
    for device in &mut info.devices {
        if device.vendor.is_none() {
            if let Some(vendor) = device.vendor_id.and_then(vendor_map::usb_vendor_name) {
                device.vendor = Some(vendor.to_string());
            }
        }
    }
    Ok(info)
}

#[cfg(target_os = "windows")]
fn platform_query() -> QueryResult<UsbInfo> {
    win::enumerate()
}

#[cfg(target_os = "linux")]
fn platform_query() -> QueryResult<UsbInfo> {
    linux::enumerate()
}

#[cfg(target_os = "macos")]
fn platform_query() -> QueryResult<UsbInfo> {
    macos::enumerate()
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn platform_query() -> QueryResult<UsbInfo> {
    Ok(UsbInfo {
        devices: Vec::new(),
        warnings: vec!["USB enumeration is not implemented for this platform".to_string()],
    })
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub(super) fn enumerate() -> QueryResult<UsbInfo> {
        let base = std::path::Path::new("/sys/bus/usb/devices");
        let mut devices = Vec::new();
        let mut warnings = Vec::new();

        let entries = match std::fs::read_dir(base) {
            Ok(entries) => entries,
            Err(error) => {
                return Err(super::super::QueryError::io(
                    "usb",
                    "cannot read /sys/bus/usb/devices",
                    &error,
                ));
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(vendor_id) = read_hex(&path.join("idVendor")) else {
                continue;
            };
            let Some(product_id) = read_hex(&path.join("idProduct")) else {
                continue;
            };

            devices.push(UsbDevice {
                bus: read_u8(&path.join("busnum")),
                address: read_u8(&path.join("devnum")),
                vendor_id: Some(vendor_id),
                product_id: Some(product_id),
                vendor: read_string(&path.join("manufacturer")),
                product: read_string(&path.join("product")),
                serial: read_string(&path.join("serial")),
                class: read_hex(&path.join("bDeviceClass")).map(class_name),
                speed_mbps: read_string(&path.join("speed"))
                    .and_then(|value| value.parse::<f64>().ok()),
            });
        }

        devices.sort_by(|left, right| (left.bus, left.address).cmp(&(right.bus, right.address)));
        if devices.is_empty() {
            warnings.push("no USB devices were reported by sysfs".to_string());
        }

        Ok(UsbInfo { devices, warnings })
    }

    fn read_string(path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn read_hex(path: &std::path::Path) -> Option<u16> {
        let value = read_string(path)?;
        u16::from_str_radix(value.trim_start_matches("0x"), 16).ok()
    }

    fn read_u8(path: &std::path::Path) -> Option<u8> {
        read_string(path)?.parse::<u8>().ok()
    }

    fn class_name(code: u16) -> String {
        match code {
            0x01 => "Audio",
            0x02 => "Communications",
            0x03 => "Human Interface",
            0x05 => "Physical",
            0x06 => "Still Imaging",
            0x07 => "Printer",
            0x08 => "Mass Storage",
            0x09 => "Hub",
            0x0A => "CDC Data",
            0x0B => "Smart Card",
            0x0D => "Content Security",
            0x0E => "Video",
            0x0F => "Personal Healthcare",
            0x10 => "Audio/Video",
            0xDC => "Diagnostic",
            0xE0 => "Wireless",
            0xEF => "Miscellaneous",
            0xFE => "Application Specific",
            0xFF => "Vendor Specific",
            _ => "Unspecified",
        }
        .to_string()
    }
}

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        DIGCF_PRESENT, GUID_DEVCLASS_USB, SP_DEVINFO_DATA, SPDRP_ADDRESS, SPDRP_BUSNUMBER,
        SPDRP_CLASS, SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME, SPDRP_HARDWAREID, SPDRP_MFG,
        SPDRP_SERVICE, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW,
    };
    use windows::core::PCWSTR;

    struct DeviceInfoSet(windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO);

    impl Drop for DeviceInfoSet {
        fn drop(&mut self) {
            // SAFETY: 句柄由 SetupDiGetClassDevsW 返回且只被这个 guard 持有，
            // Drop 只会释放一次。
            unsafe {
                let _ = SetupDiDestroyDeviceInfoList(self.0);
            }
        }
    }

    pub(super) fn enumerate() -> QueryResult<UsbInfo> {
        // SAFETY: 只按设备类 GUID 查询，参数都是本函数构造的有效值；返回的
        // 句柄立刻交给 DeviceInfoSet 托管。
        let set = unsafe {
            SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_USB),
                PCWSTR::null(),
                None,
                DIGCF_PRESENT,
            )
        }
        .map(DeviceInfoSet)
        .map_err(|error| {
            super::super::QueryError::backend("usb", "SetupAPI USB enumeration failed")
                .with_detail(error.to_string())
        })?;

        let mut devices = Vec::new();
        let mut warnings = Vec::new();

        for index in 0.. {
            let mut data = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            // SAFETY: set 有效，data 是已填好 cbSize 的栈上结构体，由 API 写入。
            if unsafe { SetupDiEnumDeviceInfo(set.0, index, &mut data) }.is_err() {
                break;
            }

            let hardware_ids = get_string(set.0, &data, SPDRP_HARDWAREID).unwrap_or_default();
            let (vendor_id, product_id) = parse_vid_pid(&hardware_ids);
            let description = get_string(set.0, &data, SPDRP_DEVICEDESC);
            let friendly = get_string(set.0, &data, SPDRP_FRIENDLYNAME);
            let name = friendly
                .clone()
                .or_else(|| description.clone())
                .unwrap_or_else(|| "USB device".to_string());

            // 不能按本地化名称过滤：非英文 Windows 上 "Root Hub" 并不出现。
            // 设备实例 ID 与驱动服务名是与语言无关的稳定标识。
            let instance_id = get_instance_id(set.0, &data).unwrap_or_default();
            let service = get_string(set.0, &data, SPDRP_SERVICE).unwrap_or_default();
            if is_hub_or_controller(&instance_id, &service) {
                continue;
            }

            if vendor_id.is_none() && product_id.is_none() {
                warnings.push(format!("USB device without VID/PID: {name}"));
                continue;
            }

            devices.push(UsbDevice {
                bus: get_u32(set.0, &data, SPDRP_BUSNUMBER).map(|value| value as u8),
                address: get_u32(set.0, &data, SPDRP_ADDRESS).map(|value| value as u8),
                vendor_id,
                product_id,
                vendor: get_string(set.0, &data, SPDRP_MFG),
                product: description,
                serial: None,
                class: get_string(set.0, &data, SPDRP_CLASS).or_else(|| Some("USB".to_string())),
                speed_mbps: None,
            });
        }

        devices.sort_by(|left, right| (left.bus, left.address).cmp(&(right.bus, right.address)));
        if devices.is_empty() {
            warnings.push("no USB devices were reported by SetupAPI".to_string());
        }

        Ok(UsbInfo { devices, warnings })
    }

    fn get_string(
        set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        data: &SP_DEVINFO_DATA,
        property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<String> {
        // 两趟调用：固定缓冲区遇上长属性（例如多字符串的 SPDRP_HARDWAREID）
        // 会返回 ERROR_INSUFFICIENT_BUFFER，旧写法 .ok()? 直接丢设备。
        // 先探所需字节数，再按需分配重试。
        let mut probe = [0u8; 8];
        let mut required = 0u32;
        // SAFETY: set/data 来自同一台设备，属性缓冲区是不足 8 字节的探针，
        // API 只在容量内写入并通过 required 报告真正需要的字节数。
        match unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                set,
                data,
                property,
                None,
                Some(&mut probe),
                Some(&mut required),
            )
        } {
            Ok(()) => return decode_utf16(&probe),
            // 只有“缓冲区不够”才带出 required；其他错误（找不到属性、
            // 数据类型不符）表示设备没有这个属性。
            Err(_) if required > 0 => {}
            Err(_) => return None,
        }

        let mut buffer = vec![0u8; required as usize];
        let mut used = 0u32;
        // SAFETY: 缓冲区刚好是探针报告的长度，API 最多写这么多字节。
        unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                set,
                data,
                property,
                None,
                Some(buffer.as_mut_slice()),
                Some(&mut used),
            )
            .ok()?;
        }
        decode_utf16(&buffer)
    }

    fn decode_utf16(bytes: &[u8]) -> Option<String> {
        let words: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .take_while(|word| *word != 0)
            .collect();
        if words.is_empty() {
            None
        } else {
            Some(String::from_utf16_lossy(&words))
        }
    }

    fn get_instance_id(
        set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        data: &SP_DEVINFO_DATA,
    ) -> Option<String> {
        let mut probe = [0u16; 128];
        let mut required = 0u32;
        // SAFETY: 同上，先把 128 个 wchar 的探针交给 API，超长时只读取
        // required 报告的长度。
        match unsafe {
            SetupDiGetDeviceInstanceIdW(set, data, Some(&mut probe), Some(&mut required))
        } {
            Ok(()) => return trim_utf16(&probe),
            Err(_) if required > 0 => {}
            Err(_) => return None,
        }
        let mut buffer = vec![0u16; required as usize];
        let mut used = 0u32;
        // SAFETY: 缓冲区按 required 分配，API 不会写入超出该长度的内容。
        unsafe {
            SetupDiGetDeviceInstanceIdW(set, data, Some(buffer.as_mut_slice()), Some(&mut used))
                .ok()?;
        }
        trim_utf16(&buffer)
    }

    fn trim_utf16(words: &[u16]) -> Option<String> {
        let end = words
            .iter()
            .position(|word| *word == 0)
            .unwrap_or(words.len());
        if end == 0 {
            None
        } else {
            Some(String::from_utf16_lossy(&words[..end]))
        }
    }

    /// Root hubs live under `USB\\ROOT_HUB*` and host controllers are driven
    /// by the `USB*HCI` services; both are language-independent markers.
    fn is_hub_or_controller(instance_id: &str, service: &str) -> bool {
        let root_hub = instance_id.to_ascii_uppercase().contains("ROOT_HUB");
        let controller = ["USBXHCI", "USBEHCI", "USBOHCI", "USBUHCI"]
            .iter()
            .any(|known| service.eq_ignore_ascii_case(known));
        root_hub || controller
    }

    fn get_u32(
        set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        data: &SP_DEVINFO_DATA,
        property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<u32> {
        let mut buffer = [0u8; 4];
        let mut required = 0u32;
        // SAFETY: 4 字节缓冲区足够放下 SPDRP_* 的 DWORD 值，API 只在容量内写。
        unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                set,
                data,
                property,
                None,
                Some(&mut buffer),
                Some(&mut required),
            )
            .ok()?;
        }
        Some(u32::from_le_bytes(buffer))
    }

    fn parse_vid_pid(hardware_ids: &str) -> (Option<u16>, Option<u16>) {
        let mut vendor = None;
        let mut product = None;
        for part in hardware_ids.split(['\\', '&']) {
            if let Some(value) = part.strip_prefix("VID_") {
                vendor = u16::from_str_radix(value, 16).ok();
            } else if let Some(value) = part.strip_prefix("PID_") {
                product = u16::from_str_radix(value, 16).ok();
            }
        }
        (vendor, product)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub(super) fn enumerate() -> QueryResult<UsbInfo> {
        let output = std::process::Command::new("system_profiler")
            .args(["SPUSBDataType", "-json"])
            .output()
            .map_err(|error| {
                super::super::QueryError::backend("usb", "system_profiler failed to start")
                    .with_detail(error.to_string())
            })?;

        if !output.status.success() {
            return Err(super::super::QueryError::backend(
                "usb",
                "system_profiler returned an error",
            )
            .with_detail(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| {
            super::super::QueryError::parse("usb", "invalid system_profiler JSON")
                .with_detail(error.to_string())
        })?;

        let mut devices = Vec::new();
        if let Some(items) = value.get("SPUSBDataType") {
            walk(items, &mut devices);
        }

        Ok(UsbInfo {
            devices,
            warnings: Vec::new(),
        })
    }

    fn walk(value: &serde_json::Value, devices: &mut Vec<UsbDevice>) {
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    walk(item, devices);
                }
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("vendor_id") {
                    devices.push(UsbDevice {
                        bus: None,
                        address: None,
                        vendor_id: parse_hex(map.get("vendor_id")),
                        product_id: parse_hex(map.get("product_id")),
                        vendor: map
                            .get("manufacturer")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        product: map
                            .get("_name")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        serial: map
                            .get("serial_num")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        class: map
                            .get("bDeviceClass")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        speed_mbps: map
                            .get("device_speed")
                            .and_then(|value| value.as_str())
                            .and_then(parse_speed),
                    });
                }
                if let Some(children) = map.get("USBDevice") {
                    walk(children, devices);
                }
            }
            _ => {}
        }
    }

    fn parse_hex(value: Option<&serde_json::Value>) -> Option<u16> {
        let text = value?.as_str()?;
        let text = text.split_whitespace().next()?.trim_start_matches("0x");
        u16::from_str_radix(text, 16).ok()
    }

    fn parse_speed(value: &str) -> Option<f64> {
        let number = value.split_whitespace().next()?.parse::<f64>().ok()?;
        if value.contains("Gb") {
            Some(number * 1000.0)
        } else {
            Some(number)
        }
    }
}
