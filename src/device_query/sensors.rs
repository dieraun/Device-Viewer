//! Thermal, fan and voltage sensors.
//!
//! `sysinfo` exposes the platform component list (CPU package, NVMe, GPU on
//! some systems). Linux additionally exposes raw `hwmon` channels, which is the
//! only portable-ish source for fan RPM and voltage rails.

use serde::{Deserialize, Serialize};
#[cfg(feature = "sensors")]
use sysinfo::Components;
use tokio_util::sync::CancellationToken;

#[cfg(not(feature = "sensors"))]
use super::error::QueryError;
use super::error::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemperatureSensor {
    /// `cpu`, `gpu`, `storage`, `motherboard` or `other`.
    pub kind: String,
    pub label: String,
    pub celsius: f32,
    pub max_celsius: Option<f32>,
    pub critical_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FanSensor {
    pub label: String,
    pub rpm: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoltageSensor {
    pub label: String,
    pub volts: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorsInfo {
    pub temperatures: Vec<TemperatureSensor>,
    pub fans: Vec<FanSensor>,
    pub voltages: Vec<VoltageSensor>,
    pub warnings: Vec<String>,
}

#[cfg(not(feature = "sensors"))]
pub async fn query(_cancel: &CancellationToken) -> QueryResult<SensorsInfo> {
    Err(QueryError::unsupported(
        "sensors",
        "this build was compiled without the `sensors` feature",
    ))
}

#[cfg(feature = "sensors")]
pub async fn query(cancel: &CancellationToken) -> QueryResult<SensorsInfo> {
    super::run_blocking("sensors", cancel, || {
        let mut info = SensorsInfo::default();

        match Components::new_with_refreshed_list() {
            Ok(components) => {
                for component in &components {
                    if let Some(celsius) = component.temperature() {
                        let label = component.label().to_string();
                        info.temperatures.push(TemperatureSensor {
                            kind: classify(&label).to_string(),
                            label,
                            celsius,
                            max_celsius: component.max(),
                            critical_celsius: component.critical(),
                        });
                    }
                }
                if info.temperatures.is_empty() {
                    info.warnings
                        .push("no temperature component was reported by sysinfo".to_string());
                }
            }
            Err(error) => info
                .warnings
                .push(format!("component sensors unavailable: {error}")),
        }

        query_nvidia_temperatures(&mut info);

        #[cfg(target_os = "linux")]
        linux::collect(&mut info);

        #[cfg(target_os = "windows")]
        info.warnings.push(
            "fan and voltage channels require vendor tooling on Windows; temperatures come from ACPI where available"
                .to_string(),
        );

        deduplicate(&mut info);
        Ok(info)
    })
    .await
}

#[cfg(feature = "sensors")]
fn classify(label: &str) -> &'static str {
    let lower = label.to_ascii_lowercase();
    if lower.contains("gpu") || lower.contains("nvidia") || lower.contains("amdgpu") {
        "gpu"
    } else if lower.contains("nvme")
        || lower.contains("ssd")
        || lower.contains("drive")
        || lower.contains("disk")
    {
        "storage"
    } else if lower.contains("cpu")
        || lower.contains("core")
        || lower.contains("package")
        || lower.contains("tdie")
    {
        "cpu"
    } else if lower.contains("board")
        || lower.contains("motherboard")
        || lower.contains("acpi")
        || lower.contains("pch")
    {
        "motherboard"
    } else {
        "other"
    }
}

#[cfg(all(feature = "sensors", target_os = "windows"))]
fn query_nvidia_temperatures(info: &mut SensorsInfo) {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output();
    let Ok(output) = output else { return };
    if !output.status.success() {
        return;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let Some((name, temperature)) = line.rsplit_once(',') else {
            continue;
        };
        let Ok(value) = temperature.trim().parse::<f32>() else {
            continue;
        };
        info.temperatures.push(TemperatureSensor {
            kind: "gpu".to_string(),
            label: name.trim().to_string(),
            celsius: value,
            max_celsius: None,
            critical_celsius: None,
        });
    }
}

#[cfg(all(feature = "sensors", target_os = "linux"))]
fn query_nvidia_temperatures(info: &mut SensorsInfo) {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output();
    let Ok(output) = output else { return };
    if !output.status.success() {
        return;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let Some((name, temperature)) = line.rsplit_once(',') else {
            continue;
        };
        let Ok(value) = temperature.trim().parse::<f32>() else {
            continue;
        };
        info.temperatures.push(TemperatureSensor {
            kind: "gpu".to_string(),
            label: name.trim().to_string(),
            celsius: value,
            max_celsius: None,
            critical_celsius: None,
        });
    }
}

#[cfg(all(
    feature = "sensors",
    not(any(target_os = "windows", target_os = "linux"))
))]
fn query_nvidia_temperatures(_info: &mut SensorsInfo) {}

#[cfg(all(feature = "sensors", target_os = "linux"))]
mod linux {
    use super::{FanSensor, SensorsInfo, TemperatureSensor, VoltageSensor, classify};
    use std::path::{Path, PathBuf};

    pub(super) fn collect(info: &mut SensorsInfo) {
        let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") else {
            return;
        };

        for entry in entries.flatten() {
            let root = entry.path();
            let chip = read_trim(&root.join("name")).unwrap_or_else(|| "hwmon".to_string());
            collect_temperatures(&root, &chip, info);
            collect_fans(&root, &chip, info);
            collect_voltages(&root, &chip, info);
        }
    }

    fn collect_temperatures(root: &Path, chip: &str, info: &mut SensorsInfo) {
        for path in numbered(root, "temp", "_input") {
            let Some(raw) = read_f32(&path) else { continue };
            let stem = path
                .file_stem()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            let index = stem.trim_start_matches("temp").trim_end_matches("_input");
            let label = read_trim(&root.join(format!("temp{index}_label")))
                .unwrap_or_else(|| format!("{chip} {stem}"));
            info.temperatures.push(TemperatureSensor {
                kind: classify(&format!("{chip} {label}")).to_string(),
                label,
                celsius: raw / 1000.0,
                max_celsius: read_f32(&root.join(format!("temp{index}_max")))
                    .map(|value| value / 1000.0),
                critical_celsius: read_f32(&root.join(format!("temp{index}_crit")))
                    .map(|value| value / 1000.0),
            });
        }
    }

    fn collect_fans(root: &Path, chip: &str, info: &mut SensorsInfo) {
        for path in numbered(root, "fan", "_input") {
            let Some(raw) = read_f32(&path) else { continue };
            let stem = path
                .file_stem()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            let index = stem.trim_start_matches("fan").trim_end_matches("_input");
            let label = read_trim(&root.join(format!("fan{index}_label")))
                .unwrap_or_else(|| format!("{chip} {stem}"));
            info.fans.push(FanSensor {
                label,
                rpm: Some(raw.max(0.0) as u32),
            });
        }
    }

    fn collect_voltages(root: &Path, chip: &str, info: &mut SensorsInfo) {
        for path in numbered(root, "in", "_input") {
            let Some(raw) = read_f32(&path) else { continue };
            let stem = path
                .file_stem()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            let index = stem.trim_start_matches("in").trim_end_matches("_input");
            let label = read_trim(&root.join(format!("in{index}_label")))
                .unwrap_or_else(|| format!("{chip} {stem}"));
            info.voltages.push(VoltageSensor {
                label,
                volts: Some(raw / 1000.0),
            });
        }
    }

    fn numbered(root: &Path, prefix: &str, suffix: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let Ok(entries) = std::fs::read_dir(root) else {
            return paths;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(prefix) && name.ends_with(suffix) {
                paths.push(entry.path());
            }
        }
        paths.sort();
        paths
    }

    fn read_trim(path: &Path) -> Option<String> {
        std::fs::read_to_string(path)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn read_f32(path: &Path) -> Option<f32> {
        read_trim(path)?.parse().ok()
    }
}

#[cfg(feature = "sensors")]
fn deduplicate(info: &mut SensorsInfo) {
    info.temperatures
        .sort_by(|left, right| (&left.kind, &left.label).cmp(&(&right.kind, &right.label)));
    info.temperatures
        .dedup_by(|left, right| left.kind == right.kind && left.label == right.label);
    info.fans
        .sort_by(|left, right| left.label.cmp(&right.label));
    info.fans.dedup_by(|left, right| left.label == right.label);
    info.voltages
        .sort_by(|left, right| left.label.cmp(&right.label));
    info.voltages
        .dedup_by(|left, right| left.label == right.label);
}
