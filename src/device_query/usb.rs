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
        SetupDiGetDeviceRegistryPropertyW,
    };
    use windows::core::PCWSTR;

    struct DeviceInfoSet(windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO);

    impl Drop for DeviceInfoSet {
        fn drop(&mut self) {
            unsafe {
                let _ = SetupDiDestroyDeviceInfoList(self.0);
            }
        }
    }

    pub(super) fn enumerate() -> QueryResult<UsbInfo> {
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

            if name.contains("Root Hub") || name.contains("Host Controller") {
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
            let _ = get_string(set.0, &data, SPDRP_SERVICE);
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
        let mut buffer = [0u8; 2048];
        let mut required = 0u32;
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

        let words: Vec<u16> = buffer
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

    fn get_u32(
        set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        data: &SP_DEVINFO_DATA,
        property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<u32> {
        let mut buffer = [0u8; 4];
        let mut required = 0u32;
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
