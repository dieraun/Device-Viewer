//! Integration tests for the hardware acquisition layer.
//!
//! The crate has no `src/lib.rs` yet, so the module tree is included directly.
//! This keeps the data layer testable while the GUI entry point is unfinished.

#[allow(dead_code)]
#[path = "../src/device_query/mod.rs"]
mod device_query;

use device_query::{
    ErrorLevel, HardwareSnapshot, QueryError, QueryErrorKind, UnitSystem, bandwidth, format_bytes,
    format_duration, format_frequency, vendor_map,
};
use tokio_util::sync::CancellationToken;

#[test]
fn package_metadata_is_stable() {
    assert_eq!(env!("CARGO_PKG_NAME"), "device-viewer");
    assert_eq!(env!("CARGO_PKG_VERSION"), "1.0.0");
}

#[test]
fn binary_and_decimal_units_differ() {
    assert_eq!(format_bytes(1024, UnitSystem::Binary), "1.00 KiB");
    assert_eq!(format_bytes(1024, UnitSystem::Decimal), "1.02 KB");
    assert_eq!(format_bytes(0, UnitSystem::Binary), "0 B");
}

#[test]
fn frequencies_and_durations_are_formatted() {
    assert_eq!(format_frequency(4_800), "4.80 GHz");
    assert_eq!(format_frequency(999), "999 MHz");
    assert_eq!(format_duration(3_661), "1h 1m 1s");
}

#[test]
fn vendor_map_is_offline_and_has_known_vendors() {
    assert_eq!(vendor_map::vendor_name(0x10DE), Some("NVIDIA"));
    assert_eq!(
        vendor_map::vendor_name(0x1002),
        Some("Advanced Micro Devices")
    );
    assert!(vendor_map::vendor_name(0xFFFF).is_none());
    assert!(vendor_map::pci_class_name(0x03).is_some());
    assert_eq!(vendor_map::describe_pci(0x10DE, 0xFFFF), "NVIDIA [FFFF]");
}

#[test]
fn snapshot_serializes_every_optional_section() {
    let snapshot = HardwareSnapshot::default();
    let json = serde_json::to_string(&snapshot).expect("snapshot must serialize");
    let restored: HardwareSnapshot =
        serde_json::from_str(&json).expect("snapshot must deserialize");
    assert_eq!(restored.schema_version, snapshot.schema_version);
    assert_eq!(restored.failures.len(), 0);
    assert!(restored.directx.is_none());
}

#[test]
fn warning_errors_do_not_abort_other_modules() {
    let warning = QueryError::permission_denied("usb", "access denied");
    let error = QueryError::timeout("storage", std::time::Duration::from_millis(5));
    assert_eq!(warning.level, ErrorLevel::Warning);
    assert_eq!(error.level, ErrorLevel::Error);
    assert_eq!(error.kind, QueryErrorKind::Timeout);
}

#[tokio::test]
async fn cancelled_module_is_reported_without_running() {
    let cancel = CancellationToken::new();
    cancel.cancel();

    let result = bandwidth::query(&cancel).await;
    let error = result.expect_err("cancelled query must fail");
    assert_eq!(error.kind, QueryErrorKind::Cancelled);
    assert_eq!(error.module, "bandwidth");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full_collection_isolates_module_failures() {
    let cancel = CancellationToken::new();
    let options = device_query::QueryOptions {
        default_timeout: std::time::Duration::from_secs(5),
        ..device_query::QueryOptions::default()
    };
    let snapshot = device_query::collect_all(&options, &cancel).await;

    assert_eq!(snapshot.schema_version, 1);
    assert!(snapshot.captured_at_unix_ms > 0);
    assert!(
        snapshot
            .failures
            .iter()
            .all(|error| !error.module.is_empty() && !error.message.is_empty())
    );
}

#[test]
fn non_finite_telemetry_is_sanitized_before_serialization() {
    let mut snapshot = HardwareSnapshot::default();
    let mut cpu = device_query::cpu::CpuInfo::default();
    cpu.global_usage_percent = f32::NAN;
    cpu.cores.push(device_query::cpu::CpuCore {
        id: 0,
        usage_percent: f32::INFINITY,
        frequency_mhz: 3_600,
    });
    snapshot.cpu = Some(cpu);

    let mut sensors = device_query::sensors::SensorsInfo::default();
    sensors
        .temperatures
        .push(device_query::sensors::TemperatureSensor {
            kind: "cpu".to_string(),
            label: "package".to_string(),
            celsius: f32::NAN,
            max_celsius: Some(f32::NEG_INFINITY),
            critical_celsius: Some(f32::INFINITY),
        });
    snapshot.sensors = Some(sensors);

    snapshot.sanitize();
    let json = serde_json::to_string(&snapshot).expect("sanitized snapshot serializes");
    let restored: HardwareSnapshot = serde_json::from_str(&json).expect("sanitized JSON parses");
    let cpu = restored.cpu.expect("CPU section");
    assert!(cpu.global_usage_percent.is_finite());
    assert!(cpu.cores[0].usage_percent.is_finite());
    let sensor = &restored.sensors.expect("sensor section").temperatures[0];
    assert!(sensor.celsius.is_finite());
    assert!(sensor.max_celsius.is_some_and(f32::is_finite));
    assert!(sensor.critical_celsius.is_some_and(f32::is_finite));
}

#[test]
fn future_snapshot_fields_and_errors_are_forward_compatible() {
    let json = r#"{
        "schema_version": 99,
        "captured_at_unix_ms": 42,
        "future_section": { "kind": "hologram" },
        "cpu": null,
        "failures": [{
            "module": "future",
            "kind": "quantum_entanglement",
            "level": "fatal",
            "message": "written by a newer build"
        }]
    }"#;
    let snapshot: HardwareSnapshot =
        serde_json::from_str(json).expect("unknown fields must be ignored");
    assert_eq!(snapshot.schema_version, 99);
    assert_eq!(snapshot.captured_at_unix_ms, 42);
    assert_eq!(snapshot.elapsed_ms, 0);
    assert!(snapshot.cpu.is_none());
    assert_eq!(snapshot.failures.len(), 1);
    assert_eq!(snapshot.failures[0].kind, QueryErrorKind::Unknown);
    assert_eq!(snapshot.failures[0].level, ErrorLevel::Unknown);

    let encoded = serde_json::to_string(&snapshot).expect("snapshot reserializes");
    assert!(encoded.contains("\"kind\":\"unknown\""));
    assert!(encoded.contains("\"level\":\"unknown\""));
}

#[test]
fn minimal_snapshot_uses_schema_defaults() {
    let snapshot: HardwareSnapshot =
        serde_json::from_str(r#"{"cpu":null}"#).expect("minimal snapshot parses");
    assert_eq!(snapshot.schema_version, 1);
    assert_eq!(snapshot.captured_at_unix_ms, 0);
    assert_eq!(snapshot.failures.len(), 0);
    assert!(snapshot.memory.is_none());
}

#[tokio::test]
async fn per_module_timeout_and_cancellation_are_enforced() {
    let cancel = CancellationToken::new();
    let options = device_query::QueryOptions {
        default_timeout: std::time::Duration::from_millis(20),
        ..device_query::QueryOptions::default()
    };
    assert_eq!(
        options.timeout_for("cpu"),
        std::time::Duration::from_millis(20)
    );

    let error = device_query::run_module("slow", &cancel, options.timeout_for("slow"), async {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        Ok::<_, QueryError>(())
    })
    .await
    .expect_err("slow module must time out");
    assert_eq!(error.kind, QueryErrorKind::Timeout);
    assert_eq!(error.module, "slow");
    assert!(!error.is_warning());

    let token = CancellationToken::new();
    let canceller = token.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        canceller.cancel();
    });
    let error = device_query::run_module(
        "blocked",
        &token,
        std::time::Duration::from_secs(5),
        async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok::<_, QueryError>(())
        },
    )
    .await
    .expect_err("cancelled module must stop");
    assert_eq!(error.kind, QueryErrorKind::Cancelled);
    assert_eq!(error.module, "blocked");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn core_modules_return_structured_results() {
    let cancel = CancellationToken::new();
    let (cpu, memory, pcie, storage, motherboard, display, system, sensors, usb) = tokio::join!(
        device_query::cpu::query(&cancel),
        device_query::memory::query(&cancel),
        device_query::pcie::query(&cancel),
        device_query::storage::query(&cancel),
        device_query::motherboard::query(&cancel),
        device_query::display::query(&cancel),
        device_query::system::query(&cancel),
        device_query::sensors::query(&cancel),
        device_query::usb::query(&cancel),
    );

    let cpu = cpu.expect("CPU query must succeed");
    assert!(cpu.logical_cores >= 1);
    assert_eq!(cpu.cores.len(), cpu.logical_cores);

    let memory = memory.expect("memory query must succeed");
    assert!(memory.total_bytes > 0);
    assert!(memory.used_bytes <= memory.total_bytes);

    let pcie = pcie.expect("PCIe query must return a structured result");
    assert!(pcie.devices.iter().all(|device| !device.address.is_empty()));

    let storage = storage.expect("storage query must return a structured result");
    assert!(storage.devices.iter().all(|device| {
        !device.name.is_empty()
            && device
                .health
                .as_ref()
                .and_then(|health| health.percentage_used)
                .is_none_or(|used| used <= 100)
    }));

    let motherboard = motherboard.expect("motherboard query must succeed");
    assert!(motherboard.vendor.is_some() || motherboard.name.is_some());

    let display = display.expect("display query must return a structured result");
    assert!(
        display
            .monitors
            .iter()
            .all(|monitor| !monitor.id.is_empty())
    );

    let system = system.expect("system query must succeed");
    assert!(!system.os.architecture.is_empty());

    #[cfg(feature = "sensors")]
    {
        let sensors = sensors.expect("sensor query must return a structured result");
        assert!(
            sensors
                .temperatures
                .iter()
                .all(|sensor| sensor.celsius.is_finite())
        );
    }
    #[cfg(not(feature = "sensors"))]
    {
        let error = sensors.expect_err("sensors must be unsupported without the feature");
        assert_eq!(error.kind, QueryErrorKind::Unsupported);
        assert!(error.is_warning());
    }

    let usb = usb.expect("USB query must return a structured result");
    assert!(
        usb.devices
            .iter()
            .all(|device| device.vendor_id.is_some() && device.product_id.is_some())
    );
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn directx_is_unsupported_off_windows() {
    let cancel = CancellationToken::new();
    let error = device_query::directx::query(&cancel)
        .await
        .expect_err("DirectX must be unsupported off Windows");
    assert_eq!(error.kind, QueryErrorKind::Unsupported);
    assert!(error.is_warning());
}
