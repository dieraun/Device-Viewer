//! Operating system identity, network interfaces and audio endpoints.

use serde::{Deserialize, Serialize};
use sysinfo::{Networks, System};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OsInfo {
    pub name: String,
    pub long_name: String,
    pub version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub architecture: String,
    pub distribution_id: String,
    pub uptime_seconds: Option<u64>,
    pub boot_time_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub addresses: Vec<String>,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioDevice {
    pub name: String,
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub networks: Vec<NetworkInterface>,
    pub audio: Vec<AudioDevice>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<SystemInfo> {
    super::run_blocking("system", cancel, || {
        let mut warnings = Vec::new();
        let os = OsInfo {
            name: System::name().unwrap_or_default(),
            long_name: System::long_os_version().unwrap_or_default(),
            version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            hostname: System::host_name().unwrap_or_default(),
            architecture: System::cpu_arch(),
            distribution_id: System::distribution_id(),
            uptime_seconds: System::uptime().ok(),
            boot_time_unix: System::boot_time().ok(),
        };

        let networks = match Networks::new_with_refreshed_list() {
            Ok(networks) => networks
                .iter()
                .map(|(name, data)| NetworkInterface {
                    name: name.to_string_lossy().into_owned(),
                    mac_address: data.mac_address().to_string(),
                    addresses: data.ip_networks().iter().map(ToString::to_string).collect(),
                    received_bytes: data.total_received(),
                    transmitted_bytes: data.total_transmitted(),
                })
                .collect(),
            Err(error) => {
                warnings.push(format!("network interfaces unavailable: {error}"));
                Vec::new()
            }
        };

        let audio = enumerate_audio(&mut warnings);

        Ok(SystemInfo {
            os,
            networks,
            audio,
            warnings,
        })
    })
    .await
}

#[cfg(target_os = "windows")]
fn enumerate_audio(warnings: &mut Vec<String>) -> Vec<AudioDevice> {
    win_audio::enumerate(warnings)
}

#[cfg(target_os = "linux")]
fn enumerate_audio(warnings: &mut Vec<String>) -> Vec<AudioDevice> {
    linux_audio::enumerate(warnings)
}

#[cfg(target_os = "macos")]
fn enumerate_audio(warnings: &mut Vec<String>) -> Vec<AudioDevice> {
    macos_audio::enumerate(warnings)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn enumerate_audio(_warnings: &mut Vec<String>) -> Vec<AudioDevice> {
    Vec::new()
}

#[cfg(target_os = "windows")]
mod win_audio {
    use super::AudioDevice;
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        DIGCF_PRESENT, GUID_DEVCLASS_MEDIA, HDEVINFO, SP_DEVINFO_DATA, SPDRP_DEVICEDESC,
        SPDRP_FRIENDLYNAME, SPDRP_MFG, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
        SetupDiGetClassDevsW, SetupDiGetDeviceRegistryPropertyW,
    };
    use windows::core::PCWSTR;

    struct DeviceInfoSet(HDEVINFO);

    impl Drop for DeviceInfoSet {
        fn drop(&mut self) {
            unsafe {
                let _ = SetupDiDestroyDeviceInfoList(self.0);
            }
        }
    }

    pub(super) fn enumerate(warnings: &mut Vec<String>) -> Vec<AudioDevice> {
        let set = match unsafe {
            SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_MEDIA),
                PCWSTR::null(),
                None,
                DIGCF_PRESENT,
            )
        } {
            Ok(set) => DeviceInfoSet(set),
            Err(error) => {
                warnings.push(format!("audio device enumeration failed: {error}"));
                return Vec::new();
            }
        };

        let mut devices = Vec::new();
        for index in 0..256u32 {
            let mut data = SP_DEVINFO_DATA::default();
            data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
            if unsafe { SetupDiEnumDeviceInfo(set.0, index, &mut data) }.is_err() {
                break;
            }

            let name = property(set.0, &data, SPDRP_FRIENDLYNAME)
                .or_else(|| property(set.0, &data, SPDRP_DEVICEDESC));
            let Some(name) = name else { continue };
            if devices
                .iter()
                .any(|device: &AudioDevice| device.name == name)
            {
                continue;
            }
            devices.push(AudioDevice {
                name,
                vendor: property(set.0, &data, SPDRP_MFG),
            });
        }
        devices
    }

    fn property(
        set: HDEVINFO,
        data: &SP_DEVINFO_DATA,
        property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<String> {
        let mut buffer = [0u16; 512];
        let mut required = 0u32;
        unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                set,
                data,
                property,
                None,
                Some(std::slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    buffer.len() * 2,
                )),
                Some(&mut required),
            )
        }
        .ok()?;

        let len = (required as usize / 2).min(buffer.len()).saturating_sub(1);
        let text = String::from_utf16_lossy(&buffer[..len]);
        (!text.trim().is_empty()).then_some(text)
    }
}

#[cfg(target_os = "linux")]
mod linux_audio {
    use super::AudioDevice;
    use std::path::Path;

    pub(super) fn enumerate(warnings: &mut Vec<String>) -> Vec<AudioDevice> {
        let mut devices = Vec::new();
        let base = Path::new("/sys/class/sound");
        let entries = match std::fs::read_dir(base) {
            Ok(entries) => entries,
            Err(error) => {
                warnings.push(format!("cannot read /sys/class/sound: {error}"));
                return devices;
            }
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("card") || name.contains('-') {
                continue;
            }
            let label = std::fs::read_to_string(entry.path().join("id"))
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or(name);
            devices.push(AudioDevice {
                name: label,
                vendor: None,
            });
        }
        devices
    }
}

#[cfg(target_os = "macos")]
mod macos_audio {
    use super::AudioDevice;

    pub(super) fn enumerate(warnings: &mut Vec<String>) -> Vec<AudioDevice> {
        let output = match std::process::Command::new("system_profiler")
            .args(["SPAudioDataType", "-json"])
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                warnings.push(format!("system_profiler SPAudioDataType failed: {error}"));
                return Vec::new();
            }
        };

        let value: serde_json::Value = match serde_json::from_slice(&output.stdout) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("invalid audio JSON: {error}"));
                return Vec::new();
            }
        };

        let mut devices = Vec::new();
        if let Some(items) = value
            .get("SPAudioDataType")
            .and_then(|value| value.as_array())
        {
            for item in items {
                if let Some(name) = item.get("_name").and_then(|value| value.as_str()) {
                    devices.push(AudioDevice {
                        name: name.to_string(),
                        vendor: item
                            .get("coreaudio_device_manufacturer")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                    });
                }
            }
        }
        devices
    }
}
