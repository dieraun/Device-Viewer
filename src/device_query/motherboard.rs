//! Motherboard, firmware and system product identity.

use serde::{Deserialize, Serialize};
use sysinfo::{Motherboard, Product};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;
use super::run_blocking;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BiosInfo {
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MotherboardInfo {
    pub vendor: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub serial: Option<String>,
    pub asset_tag: Option<String>,
    pub bios: Option<BiosInfo>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<MotherboardInfo> {
    run_blocking("motherboard", cancel, || {
        let mut warnings = Vec::new();
        let mut info = MotherboardInfo::default();

        match Motherboard::new() {
            Ok(board) => {
                info.vendor = non_empty(board.vendor_name());
                info.name = non_empty(board.name());
                info.version = non_empty(board.version());
                info.serial = non_empty(board.serial_number());
                info.asset_tag = non_empty(board.asset_tag());
            }
            Err(error) => warnings.push(format!("SMBIOS baseboard data unavailable: {error}")),
        }

        if info.name.is_none() {
            info.name = non_empty(Product::name().ok());
        }
        if info.vendor.is_none() {
            info.vendor = non_empty(Product::vendor_name().ok());
        }
        if info.serial.is_none() {
            info.serial = non_empty(Product::serial_number().ok());
        }
        if info.version.is_none() {
            info.version = non_empty(Product::version().ok());
        }

        info.bios = read_bios();
        info.warnings = warnings;
        Ok(info)
    })
    .await
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

#[cfg(target_os = "linux")]
fn read_bios() -> Option<BiosInfo> {
    let vendor = read_trim("/sys/class/dmi/id/bios_vendor");
    let version = read_trim("/sys/class/dmi/id/bios_version");
    let release_date = read_trim("/sys/class/dmi/id/bios_date");
    if vendor.is_none() && version.is_none() && release_date.is_none() {
        None
    } else {
        Some(BiosInfo {
            vendor,
            version,
            release_date,
        })
    }
}

#[cfg(not(target_os = "linux"))]
fn read_bios() -> Option<BiosInfo> {
    None
}

#[cfg(target_os = "linux")]
fn read_trim(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
