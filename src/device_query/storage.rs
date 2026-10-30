//! Storage devices and SMART health.
//!
//! `smartctl` supplies model, firmware, capacity and SMART counters for both
//! NVMe and SATA drives. sysinfo supplies a fallback list when smartmontools is
//! not installed, so the panel never stays empty.

use serde::{Deserialize, Serialize};
use sysinfo::Disks;
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;
use super::run_blocking;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageHealth {
    pub passed: Option<bool>,
    pub temperature_c: Option<f64>,
    pub power_on_hours: Option<u64>,
    pub power_cycle_count: Option<u64>,
    pub total_written_bytes: Option<u64>,
    pub bad_blocks: Option<u64>,
    pub percentage_used: Option<u8>,
    pub media_errors: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageDevice {
    pub name: String,
    pub model: Option<String>,
    pub firmware: Option<String>,
    pub serial: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub protocol: Option<String>,
    pub device_type: Option<String>,
    pub mount_point: Option<String>,
    pub file_system: Option<String>,
    pub removable: Option<bool>,
    pub smart_available: bool,
    pub health: Option<StorageHealth>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageInfo {
    pub devices: Vec<StorageDevice>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<StorageInfo> {
    run_blocking("storage", cancel, || {
        let mut warnings = Vec::new();
        let mut devices = Vec::new();

        match smartctl_rs::scan::scan_devices() {
            Ok(scanned) => {
                for scan in scanned {
                    let mut device = StorageDevice {
                        name: scan.name.clone(),
                        protocol: non_empty(scan.protocol),
                        device_type: non_empty(scan.dev_type),
                        smart_available: true,
                        ..StorageDevice::default()
                    };

                    match smartctl_rs::device::get_device_info(&scan.name) {
                        Ok(smart) => apply_smart(&mut device, &smart),
                        Err(error) => {
                            device.smart_available = false;
                            device.warnings.push(format!("SMART query failed: {error}"));
                        }
                    }
                    devices.push(device);
                }
            }
            Err(error) => {
                warnings.push(format!(
                    "smartmontools is unavailable ({error}); using the OS disk list"
                ));
            }
        }

        if let Ok(disks) = Disks::new_with_refreshed_list() {
            for disk in &disks {
                let name = disk.name().to_string_lossy().into_owned();
                if devices.iter().any(|device| device.name == name) {
                    continue;
                }
                devices.push(StorageDevice {
                    name,
                    capacity_bytes: Some(disk.total_space()),
                    device_type: Some(disk.kind().to_string()),
                    mount_point: Some(disk.mount_point().to_string_lossy().into_owned()),
                    file_system: Some(disk.file_system().to_string_lossy().into_owned()),
                    removable: Some(disk.is_removable()),
                    smart_available: false,
                    ..StorageDevice::default()
                });
            }
        }

        if devices.is_empty() {
            warnings.push("no storage devices were reported".to_string());
        }

        Ok(StorageInfo { devices, warnings })
    })
    .await
}

fn apply_smart(device: &mut StorageDevice, smart: &smartctl_rs::device::SmartOutput) {
    device.model = smart.model_name.clone();
    device.firmware = smart.firmware_version.clone();
    device.serial = smart.serial_number.clone();
    device.capacity_bytes = smart.user_capacity.as_ref().map(|capacity| capacity.bytes);

    let mut health = StorageHealth {
        passed: smart.smart_status.as_ref().map(|status| status.passed),
        temperature_c: smart
            .temperature
            .as_ref()
            .map(|temperature| temperature.current as f64),
        power_on_hours: smart.power_on_time.as_ref().map(|time| time.hours),
        power_cycle_count: smart.power_cycle_count,
        ..StorageHealth::default()
    };

    if let Some(log) = &smart.nvme_smart_health_information_log {
        health.total_written_bytes = log
            .get("data_units_written")
            .and_then(|units| u64::try_from(*units).ok())
            .and_then(|units| units.checked_mul(512_000));
        health.media_errors = log
            .get("media_errors")
            .and_then(|value| u64::try_from(*value).ok());
        health.percentage_used = log
            .get("percentage_used")
            .and_then(|value| u8::try_from(*value).ok());
        if health.power_on_hours.is_none() {
            health.power_on_hours = log
                .get("power_on_hours")
                .and_then(|value| u64::try_from(*value).ok());
        }
    }

    if let Some(attributes) = &smart.ata_smart_attributes {
        if health.bad_blocks.is_none() {
            health.bad_blocks = attributes
                .table
                .iter()
                .find(|attribute| attribute.id == 5)
                .map(|attribute| attribute.raw.value);
        }
        if health.total_written_bytes.is_none() {
            health.total_written_bytes = attributes
                .table
                .iter()
                .find(|attribute| attribute.id == 241)
                .and_then(|attribute| attribute.raw.value.checked_mul(512));
        }
        if health.temperature_c.is_none() {
            health.temperature_c = attributes
                .table
                .iter()
                .find(|attribute| attribute.id == 194)
                .map(|attribute| attribute.raw.value as f64);
        }
    }

    device.health = Some(health);
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}
