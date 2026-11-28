//! Realtime sampling for the values that change every second.
//!
//! A full collection pass walks SMART, DXGI and Vulkan and takes seconds, so it
//! cannot run at the display refresh rate. This monitor keeps one sysinfo
//! handle alive and re-reads only the cheap counters (CPU usage, memory,
//! uptime), which is what the panels draw as a live gauge.
//!
//! The monitor is intentionally independent from the async collector: it runs
//! on its own thread so a slow module can never stall the gauges.

use std::time::Duration;

use sysinfo::System;

use super::cpu::CpuCore;

/// How often the live thread samples. Matches the GPU poll and stays well above
/// sysinfo::MINIMUM_CPU_UPDATE_INTERVAL, so CPU usage is a real delta and not a
/// snapshot of one instant.
pub const LIVE_INTERVAL: Duration = Duration::from_secs(1);

/// One live reading.
///
/// Only the volatile counters are carried; static identity (vendor, model,
/// module list) belongs to the full collection result.
#[derive(Debug, Clone, Default)]
pub struct LiveSample {
    pub cpu_usage_percent: f32,
    pub cores: Vec<CpuCore>,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_free_bytes: u64,
    pub memory_available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub uptime_seconds: u64,
}

/// Keeps a warm System and produces a LiveSample per call.
pub struct LiveMonitor {
    system: System,
}

impl LiveMonitor {
    /// Opens the monitor, or returns None when the OS refuses enumeration.
    pub fn new() -> Option<Self> {
        let mut system = System::new().ok()?;
        system.refresh_cpu_all();
        system.refresh_memory();
        Some(Self { system })
    }

    /// Reads every volatile counter once.
    pub fn sample(&mut self) -> LiveSample {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        let cores = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(index, cpu)| CpuCore {
                id: index,
                usage_percent: cpu.usage(),
                frequency_mhz: cpu.frequency(),
            })
            .collect();
        LiveSample {
            cpu_usage_percent: self.system.global_cpu_usage(),
            cores,
            memory_total_bytes: self.system.total_memory(),
            memory_used_bytes: self.system.used_memory(),
            memory_free_bytes: self.system.free_memory(),
            memory_available_bytes: self.system.available_memory(),
            swap_total_bytes: self.system.total_swap(),
            swap_used_bytes: self.system.used_swap(),
            uptime_seconds: System::uptime().unwrap_or_default(),
        }
    }
}
