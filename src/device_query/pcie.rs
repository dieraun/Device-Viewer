//! PCI/PCIe topology and link capabilities.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};
use super::run_blocking;
use super::vendor_map;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PcieLink {
    pub current_speed: Option<String>,
    pub current_width: Option<u16>,
    pub max_speed: Option<String>,
    pub max_width: Option<u16>,
    pub generation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PcieDevice {
    pub address: String,
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: Option<String>,
    pub device_name: Option<String>,
    pub class_code: u8,
    pub subclass_code: u8,
    pub class_name: String,
    pub driver: Option<String>,
    pub revision: Option<u8>,
    pub subsystem_vendor_id: Option<u16>,
    pub subsystem_device_id: Option<u16>,
    pub link: Option<PcieLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PcieInfo {
    pub devices: Vec<PcieDevice>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<PcieInfo> {
    run_blocking("pcie", cancel, || {
        let info = pci_info::PciInfo::enumerate_pci().map_err(|error| {
            QueryError::backend("pcie", "PCI enumeration failed").with_detail(error.to_string())
        })?;

        let mut devices = Vec::new();
        let mut warnings = Vec::new();

        for entry in &info {
            let device = match entry {
                Ok(device) => device,
                Err(error) => {
                    warnings.push(format!("could not read a PCI device: {error}"));
                    continue;
                }
            };

            let address = device
                .location()
                .map(|location| location.to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let class_code = device.device_class_code().unwrap_or(0);
            let subclass_code = device.device_subclass_code().unwrap_or(0);

            devices.push(PcieDevice {
                address: address.clone(),
                vendor_id: device.vendor_id(),
                device_id: device.device_id(),
                vendor_name: vendor_map::vendor_name(device.vendor_id()).map(str::to_string),
                device_name: vendor_map::pci_device_name(device.vendor_id(), device.device_id())
                    .map(str::to_string),
                class_code,
                subclass_code,
                class_name: vendor_map::pci_class_name(class_code)
                    .unwrap_or("Unknown")
                    .to_string(),
                driver: device.os_driver().ok().and_then(|driver| driver.clone()),
                revision: device.revision().ok(),
                subsystem_vendor_id: device.subsystem_vendor_id().ok().flatten(),
                subsystem_device_id: device.subsystem_device_id().ok().flatten(),
                link: read_link(&address),
            });
        }

        devices.sort_by(|left, right| left.address.cmp(&right.address));
        if devices.is_empty() {
            warnings.push("no PCI devices were reported".to_string());
        }

        Ok(PcieInfo { devices, warnings })
    })
    .await
}

#[cfg(target_os = "linux")]
fn read_link(address: &str) -> Option<PcieLink> {
    let base = format!("/sys/bus/pci/devices/{address}");
    let current_speed = read_trim(&format!("{base}/current_link_speed"));
    let current_width = read_trim(&format!("{base}/current_link_width"))
        .and_then(|value| value.parse::<u16>().ok());
    let max_speed = read_trim(&format!("{base}/max_link_speed"));
    let max_width =
        read_trim(&format!("{base}/max_link_width")).and_then(|value| value.parse::<u16>().ok());

    if current_speed.is_none() && max_speed.is_none() {
        return None;
    }

    let generation = max_speed
        .as_deref()
        .or(current_speed.as_deref())
        .and_then(pcie_generation);

    Some(PcieLink {
        current_speed,
        current_width,
        max_speed,
        max_width,
        generation,
    })
}

#[cfg(not(target_os = "linux"))]
fn read_link(_address: &str) -> Option<PcieLink> {
    None
}

#[cfg(target_os = "linux")]
fn read_trim(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(any(target_os = "linux", test))]
fn pcie_generation(speed: &str) -> Option<String> {
    let value = speed
        .split_whitespace()
        .next()
        .and_then(|token| token.parse::<f32>().ok())?;
    let generation = if value >= 60.0 {
        "6.0"
    } else if value >= 30.0 {
        "5.0"
    } else if value >= 15.0 {
        "4.0"
    } else if value >= 7.5 {
        "3.0"
    } else if value >= 4.5 {
        "2.0"
    } else {
        "1.0"
    };
    Some(format!("PCIe {generation}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_derived_from_speed() {
        assert_eq!(
            pcie_generation("16.0 GT/s PCIe").as_deref(),
            Some("PCIe 4.0")
        );
        assert_eq!(
            pcie_generation("8.0 GT/s PCIe").as_deref(),
            Some("PCIe 3.0")
        );
    }
}
