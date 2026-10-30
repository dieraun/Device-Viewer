//! Hardware acquisition layer.
//!
//! The layer is intentionally independent from `eframe`/`egui`: it only
//! produces plain, serializable data. The GUI and the CLI share it.
//!
//! Every module exposes `async fn query(&CancellationToken) -> QueryResult<T>`.
//! [`collect_all`] fans the modules out concurrently, applies a per-module
//! timeout, and converts a failure of one module into an entry of
//! [`HardwareSnapshot::failures`] without cancelling the others.

pub mod bandwidth;
pub mod cpu;
pub mod directx;
pub mod display;
pub mod error;
pub mod memory;
pub mod metal;
pub mod motherboard;
pub mod npu;
pub mod opencl;
pub mod opengl;
pub mod openvx;
pub mod pcie;
pub mod sensors;
pub mod storage;
pub mod system;
pub mod usb;
pub mod vendor_map;
pub mod vulkan;

use std::collections::BTreeMap;
use std::future::Future;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

#[allow(unused_imports)]
pub use error::{ErrorLevel, QueryError, QueryErrorKind, QueryResult};
#[allow(unused_imports)]
pub use vulkan::GpuRuntimeStats;

/// Default deadline for a single module.
pub const DEFAULT_MODULE_TIMEOUT: Duration = Duration::from_secs(8);

/// Tunable knobs for one collection pass.
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub default_timeout: Duration,
    /// Module name → timeout. Unknown names fall back to `default_timeout`.
    pub overrides: BTreeMap<String, Duration>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            default_timeout: DEFAULT_MODULE_TIMEOUT,
            overrides: BTreeMap::new(),
        }
    }
}

impl QueryOptions {
    pub fn timeout_for(&self, module: &str) -> Duration {
        self.overrides
            .get(module)
            .copied()
            .unwrap_or(self.default_timeout)
    }
}

/// Runs one module future with cancellation and a deadline.
///
/// The function is used by [`collect_all`] but is public so dedicated views can
/// re-query a single module with the same semantics.
pub async fn run_module<F, T>(
    module: &'static str,
    cancel: &CancellationToken,
    timeout: Duration,
    future: F,
) -> QueryResult<T>
where
    F: Future<Output = QueryResult<T>>,
{
    if cancel.is_cancelled() {
        return Err(QueryError::cancelled(module));
    }

    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(QueryError::cancelled(module)),
        result = tokio::time::timeout(timeout, future) => {
            match result {
                Ok(value) => value,
                Err(_) => Err(QueryError::timeout(module, timeout)),
            }
        }
    }
}

/// Runs blocking platform code on the blocking pool while observing cancellation.
///
/// Blocking operating-system calls cannot be interrupted safely. Dropping the
/// join handle lets the caller return immediately; the worker finishes in the
/// background and its result is discarded.
pub async fn run_blocking<T, F>(
    module: &'static str,
    cancel: &CancellationToken,
    work: F,
) -> QueryResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> QueryResult<T> + Send + 'static,
{
    if cancel.is_cancelled() {
        return Err(QueryError::cancelled(module));
    }

    let handle = tokio::task::spawn_blocking(work);
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(QueryError::cancelled(module)),
        joined = handle => {
            match joined {
                Ok(result) => result,
                Err(error) => Err(QueryError::error(
                    module,
                    QueryErrorKind::Internal,
                    "blocking query task failed",
                )
                .with_detail(error.to_string())),
            }
        }
    }
}

/// A complete, serializable snapshot of the machine.
///
/// Every hardware section is optional. New fields must stay `Option` so that
/// snapshots written by older builds keep deserializing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSnapshot {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub captured_at_unix_ms: u64,
    #[serde(default)]
    pub elapsed_ms: u64,
    #[serde(default)]
    pub cpu: Option<cpu::CpuInfo>,
    #[serde(default)]
    pub memory: Option<memory::MemoryInfo>,
    #[serde(default)]
    pub pcie: Option<pcie::PcieInfo>,
    #[serde(default)]
    pub vulkan: Option<vulkan::VulkanInfo>,
    #[serde(default)]
    pub opengl: Option<opengl::OpenGlInfo>,
    #[serde(default)]
    pub opencl: Option<opencl::OpenClInfo>,
    #[serde(default)]
    pub npu: Option<npu::NpuInfo>,
    #[serde(default)]
    pub usb: Option<usb::UsbInfo>,
    #[serde(default)]
    pub storage: Option<storage::StorageInfo>,
    #[serde(default)]
    pub motherboard: Option<motherboard::MotherboardInfo>,
    #[serde(default)]
    pub display: Option<display::DisplayInfo>,
    #[serde(default)]
    pub directx: Option<directx::DirectXInfo>,
    #[serde(default)]
    pub metal: Option<metal::MetalInfo>,
    #[serde(default)]
    pub openvx: Option<openvx::OpenVxInfo>,
    #[serde(default)]
    pub system: Option<system::SystemInfo>,
    #[serde(default)]
    pub sensors: Option<sensors::SensorsInfo>,
    #[serde(default)]
    pub bandwidth: Option<bandwidth::BandwidthInfo>,
    /// Failed modules. A warning never prevents the snapshot from being used.
    #[serde(default)]
    pub failures: Vec<QueryError>,
}

const fn default_schema_version() -> u32 {
    1
}

impl Default for HardwareSnapshot {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            captured_at_unix_ms: 0,
            elapsed_ms: 0,
            cpu: None,
            memory: None,
            pcie: None,
            vulkan: None,
            opengl: None,
            opencl: None,
            npu: None,
            usb: None,
            storage: None,
            motherboard: None,
            display: None,
            directx: None,
            metal: None,
            openvx: None,
            system: None,
            sensors: None,
            bandwidth: None,
            failures: Vec::new(),
        }
    }
}

impl HardwareSnapshot {
    pub fn warning_count(&self) -> usize {
        self.failures.iter().filter(|e| e.is_warning()).count()
    }

    pub fn error_count(&self) -> usize {
        self.failures.iter().filter(|e| e.is_error()).count()
    }

    /// Replaces every non-finite floating point value with a finite fallback.
    ///
    /// JSON has no representation for `NaN`/infinity: serializing one produces
    /// `null`, which then fails to deserialize back into `f32`/`f64`. Drivers
    /// and text parsers occasionally emit such values, so snapshots are
    /// normalized before they are written or exported.
    pub fn sanitize(&mut self) {
        fn finite(value: f32) -> f32 {
            if value.is_finite() { value } else { 0.0 }
        }
        fn finite_opt(value: Option<f32>) -> Option<f32> {
            value.map(finite)
        }
        fn finite_f64(value: Option<f64>) -> Option<f64> {
            value.filter(|value| value.is_finite())
        }

        if let Some(cpu) = &mut self.cpu {
            cpu.global_usage_percent = finite(cpu.global_usage_percent);
            for core in &mut cpu.cores {
                core.usage_percent = finite(core.usage_percent);
            }
        }
        if let Some(usb) = &mut self.usb {
            for device in &mut usb.devices {
                device.speed_mbps = finite_f64(device.speed_mbps);
            }
        }
        if let Some(storage) = &mut self.storage {
            for device in &mut storage.devices {
                if let Some(health) = &mut device.health {
                    health.temperature_c = finite_f64(health.temperature_c);
                }
            }
        }
        if let Some(sensors) = &mut self.sensors {
            for sensor in &mut sensors.temperatures {
                sensor.celsius = finite(sensor.celsius);
                sensor.max_celsius = finite_opt(sensor.max_celsius);
                sensor.critical_celsius = finite_opt(sensor.critical_celsius);
            }
            for sensor in &mut sensors.voltages {
                sensor.volts = finite_opt(sensor.volts);
            }
        }
        if let Some(bandwidth) = &mut self.bandwidth {
            bandwidth.read_megabytes_per_second = finite_f64(bandwidth.read_megabytes_per_second);
            bandwidth.write_megabytes_per_second = finite_f64(bandwidth.write_megabytes_per_second);
            bandwidth.copy_megabytes_per_second = finite_f64(bandwidth.copy_megabytes_per_second);
            for sample in &mut bandwidth.samples {
                if !sample.elapsed_ms.is_finite() {
                    sample.elapsed_ms = 0.0;
                }
                if !sample.megabytes_per_second.is_finite() {
                    sample.megabytes_per_second = 0.0;
                }
            }
        }
    }
}

/// Fans out every module concurrently and assembles a snapshot.
///
/// One module timing out or returning an error never prevents the others from
/// completing: `tokio::join!` collects every result before they are applied.
pub async fn collect_all(options: &QueryOptions, cancel: &CancellationToken) -> HardwareSnapshot {
    let started = Instant::now();

    let cpu_job = run_module(
        "cpu",
        cancel,
        options.timeout_for("cpu"),
        cpu::query(cancel),
    );
    let memory_job = run_module(
        "memory",
        cancel,
        options.timeout_for("memory"),
        memory::query(cancel),
    );
    let pcie_job = run_module(
        "pcie",
        cancel,
        options.timeout_for("pcie"),
        pcie::query(cancel),
    );
    let vulkan_job = run_module(
        "vulkan",
        cancel,
        options.timeout_for("vulkan"),
        vulkan::query(cancel),
    );
    let opengl_job = run_module(
        "opengl",
        cancel,
        options.timeout_for("opengl"),
        opengl::query(cancel),
    );
    let opencl_job = run_module(
        "opencl",
        cancel,
        options.timeout_for("opencl"),
        opencl::query(cancel),
    );
    let npu_job = run_module(
        "npu",
        cancel,
        options.timeout_for("npu"),
        npu::query(cancel),
    );
    let usb_job = run_module(
        "usb",
        cancel,
        options.timeout_for("usb"),
        usb::query(cancel),
    );
    let storage_job = run_module(
        "storage",
        cancel,
        options.timeout_for("storage"),
        storage::query(cancel),
    );
    let motherboard_job = run_module(
        "motherboard",
        cancel,
        options.timeout_for("motherboard"),
        motherboard::query(cancel),
    );
    let display_job = run_module(
        "display",
        cancel,
        options.timeout_for("display"),
        display::query(cancel),
    );
    let directx_job = run_module(
        "directx",
        cancel,
        options.timeout_for("directx"),
        directx::query(cancel),
    );
    let metal_job = run_module(
        "metal",
        cancel,
        options.timeout_for("metal"),
        metal::query(cancel),
    );
    let openvx_job = run_module(
        "openvx",
        cancel,
        options.timeout_for("openvx"),
        openvx::query(cancel),
    );
    let system_job = run_module(
        "system",
        cancel,
        options.timeout_for("system"),
        system::query(cancel),
    );
    let sensors_job = run_module(
        "sensors",
        cancel,
        options.timeout_for("sensors"),
        sensors::query(cancel),
    );
    let bandwidth_job = run_module(
        "bandwidth",
        cancel,
        options.timeout_for("bandwidth"),
        bandwidth::query(cancel),
    );

    let (
        cpu_result,
        memory_result,
        pcie_result,
        vulkan_result,
        opengl_result,
        opencl_result,
        npu_result,
        usb_result,
        storage_result,
        motherboard_result,
        display_result,
        directx_result,
        metal_result,
        openvx_result,
        system_result,
        sensors_result,
        bandwidth_result,
    ) = tokio::join!(
        cpu_job,
        memory_job,
        pcie_job,
        vulkan_job,
        opengl_job,
        opencl_job,
        npu_job,
        usb_job,
        storage_job,
        motherboard_job,
        display_job,
        directx_job,
        metal_job,
        openvx_job,
        system_job,
        sensors_job,
        bandwidth_job,
    );

    let mut snapshot = HardwareSnapshot::default();
    macro_rules! apply {
        ($result:expr, $field:ident) => {
            match $result {
                Ok(value) => snapshot.$field = Some(value),
                Err(error) => snapshot.failures.push(error),
            }
        };
    }

    apply!(cpu_result, cpu);
    apply!(memory_result, memory);
    apply!(pcie_result, pcie);
    apply!(vulkan_result, vulkan);
    apply!(opengl_result, opengl);
    apply!(opencl_result, opencl);
    apply!(npu_result, npu);
    apply!(usb_result, usb);
    apply!(storage_result, storage);
    apply!(motherboard_result, motherboard);
    apply!(display_result, display);
    apply!(directx_result, directx);
    apply!(metal_result, metal);
    apply!(openvx_result, openvx);
    apply!(system_result, system);
    apply!(sensors_result, sensors);
    apply!(bandwidth_result, bandwidth);

    snapshot.elapsed_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    snapshot.captured_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default();
    snapshot.sanitize();
    snapshot
}

/// Byte unit convention used by the formatters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnitSystem {
    /// 1024-based units (KiB, MiB, GiB, ...).
    #[default]
    Binary,
    /// 1000-based units (KB, MB, GB, ...).
    Decimal,
}

impl UnitSystem {
    pub const fn base(self) -> f64 {
        match self {
            Self::Binary => 1024.0,
            Self::Decimal => 1000.0,
        }
    }

    pub const fn labels(self) -> [&'static str; 7] {
        match self {
            Self::Binary => ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"],
            Self::Decimal => ["B", "KB", "MB", "GB", "TB", "PB", "EB"],
        }
    }
}

/// Formats a byte count with the selected unit convention.
pub fn format_bytes(bytes: u64, units: UnitSystem) -> String {
    let labels = units.labels();
    if bytes < 1 {
        return format!("0 {}", labels[0]);
    }
    let mut value = bytes as f64;
    let mut index = 0;
    while value >= units.base() && index + 1 < labels.len() {
        value /= units.base();
        index += 1;
    }
    if index == 0 {
        format!("{} {}", bytes, labels[index])
    } else {
        format!("{value:.2} {}", labels[index])
    }
}

/// Formats a frequency given in MHz.
pub fn format_frequency(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.2} GHz", mhz as f64 / 1000.0)
    } else {
        format!("{mhz} MHz")
    }
}

/// Formats a duration given in seconds as `d h m s`.
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {secs}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_bytes_use_1024() {
        assert_eq!(format_bytes(1536, UnitSystem::Binary), "1.50 KiB");
    }

    #[test]
    fn decimal_bytes_use_1000() {
        assert_eq!(format_bytes(1536, UnitSystem::Decimal), "1.54 KB");
    }

    #[test]
    fn frequency_switches_to_ghz() {
        assert_eq!(format_frequency(3200), "3.20 GHz");
        assert_eq!(format_frequency(800), "800 MHz");
    }

    #[test]
    fn duration_is_human_readable() {
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(90_061), "1d 1h 1m");
    }

    #[test]
    fn snapshot_round_trips_without_optional_sections() {
        let snapshot = HardwareSnapshot::default();
        let json = serde_json::to_string(&snapshot).expect("serialize");
        let decoded: HardwareSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.schema_version, 1);
        assert!(decoded.cpu.is_none());
    }

    #[tokio::test]
    async fn timeout_is_reported_as_error() {
        let cancel = CancellationToken::new();
        let result = run_module("test", &cancel, Duration::from_millis(5), async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok::<(), QueryError>(())
        })
        .await;
        let error = result.expect_err("timeout expected");
        assert_eq!(error.kind, QueryErrorKind::Timeout);
    }
}
