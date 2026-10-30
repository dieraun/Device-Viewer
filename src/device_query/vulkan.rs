//! Vulkan adapter, memory heap and live GPU telemetry.
//!
//! `ash` is used only behind the `vulkan` feature. The instance is wrapped in a
//! guard so every early return destroys it.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VulkanHeap {
    pub index: usize,
    pub size_bytes: u64,
    pub device_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VulkanDevice {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: String,
    pub api_version: String,
    pub driver_version: String,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: Option<u64>,
    pub memory_budget_supported: bool,
    pub heaps: Vec<VulkanHeap>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VulkanInfo {
    pub available: bool,
    pub loader_version: Option<String>,
    pub api_version: Option<String>,
    pub devices: Vec<VulkanDevice>,
    pub warnings: Vec<String>,
}

/// Runtime counters shared by the Vulkan and OpenGL panels.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuRuntimeStats {
    pub name: String,
    pub utilization_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

/// Live utilization / VRAM usage as reported by the operating system.
///
/// Neither Vulkan nor OpenGL expose portable per-process utilization counters;
/// this uses the same OS telemetry the drivers publish to sysinfo.
pub async fn query_runtime_stats(cancel: &CancellationToken) -> QueryResult<Vec<GpuRuntimeStats>> {
    super::run_blocking("vulkan", cancel, || {
        let gpus = sysinfo::Gpus::new_with_refreshed_list()
            .map_err(|error| super::QueryError::backend("vulkan", error.to_string()))?;
        let stats = gpus
            .iter()
            .map(|gpu| GpuRuntimeStats {
                name: gpu.model().or(gpu.vendor()).unwrap_or("GPU").to_string(),
                utilization_percent: gpu.usage(),
                memory_used_bytes: gpu.used_memory(),
                memory_total_bytes: gpu.total_memory(),
            })
            .collect();
        Ok(stats)
    })
    .await
}

#[cfg(feature = "vulkan")]
pub async fn query(cancel: &CancellationToken) -> QueryResult<VulkanInfo> {
    super::run_blocking("vulkan", cancel, query_sync).await
}

#[cfg(feature = "vulkan")]
fn query_sync() -> QueryResult<VulkanInfo> {
    use ash::vk;

    let entry = unsafe { ash::Entry::load() }.map_err(|error| {
        super::QueryError::backend("vulkan", "Vulkan loader is not available")
            .with_detail(error.to_string())
    })?;

    let loader_version = unsafe { entry.try_enumerate_instance_version() }
        .ok()
        .flatten();
    let instance_api = loader_version.unwrap_or(vk::API_VERSION_1_0);

    let app_info = vk::ApplicationInfo::default()
        .application_name(c"Device Viewer")
        .application_version(1)
        .engine_name(c"Device Viewer")
        .engine_version(1)
        .api_version(instance_api);
    let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);
    let instance = unsafe { entry.create_instance(&create_info, None) }.map_err(|error| {
        super::QueryError::backend("vulkan", "could not create a Vulkan instance")
            .with_detail(error.to_string())
    })?;

    struct InstanceGuard(ash::Instance);
    impl Drop for InstanceGuard {
        fn drop(&mut self) {
            unsafe { self.0.destroy_instance(None) };
        }
    }
    let guard = InstanceGuard(instance);
    let instance = &guard.0;

    let physical_devices = unsafe { instance.enumerate_physical_devices() }.map_err(|error| {
        super::QueryError::backend("vulkan", "could not enumerate Vulkan adapters")
            .with_detail(error.to_string())
    })?;

    let mut devices = Vec::with_capacity(physical_devices.len());
    let mut warnings = Vec::new();

    for physical_device in physical_devices {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let memory = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }
            .map(|list| {
                list.iter()
                    .filter_map(|extension| {
                        extension
                            .extension_name_as_c_str()
                            .ok()
                            .map(|name| name.to_string_lossy().into_owned())
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|error| {
                warnings.push(format!("could not list device extensions: {error}"));
                Vec::new()
            });

        let budget_supported = extensions.iter().any(|name| name == "VK_EXT_memory_budget");

        let mut heaps = Vec::new();
        for (index, heap) in memory.memory_heaps_as_slice().iter().enumerate() {
            heaps.push(VulkanHeap {
                index,
                size_bytes: heap.size,
                device_local: heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL),
            });
        }

        let total_memory_bytes = memory
            .memory_heaps_as_slice()
            .iter()
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum();

        let mut used_memory_bytes = None;
        if budget_supported && instance_api >= vk::API_VERSION_1_1 {
            let mut budget = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
            {
                let mut properties2 =
                    vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget);
                unsafe {
                    instance
                        .get_physical_device_memory_properties2(physical_device, &mut properties2);
                }
            }
            let mut used = 0u64;
            for (index, heap) in memory.memory_heaps_as_slice().iter().enumerate() {
                if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL)
                    && index < budget.heap_usage.len()
                {
                    used = used.saturating_add(budget.heap_usage[index]);
                }
            }
            used_memory_bytes = Some(used);
        }

        let device_type = match properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::CPU => "CPU",
            _ => "Other",
        };

        let api_version = properties.api_version;
        let driver_version = properties.driver_version;

        devices.push(VulkanDevice {
            name: properties
                .device_name_as_c_str()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|_| format!("Vulkan device {:04X}", properties.device_id)),
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
            device_type: device_type.to_string(),
            api_version: format!(
                "{}.{}.{}",
                (api_version >> 22) & 0x7f,
                (api_version >> 12) & 0x3ff,
                api_version & 0xfff
            ),
            driver_version: format!(
                "{}.{}.{}",
                (driver_version >> 22) & 0x7f,
                (driver_version >> 12) & 0x3ff,
                driver_version & 0xfff
            ),
            total_memory_bytes,
            used_memory_bytes,
            memory_budget_supported: budget_supported,
            heaps,
            extensions,
        });
    }

    if devices.is_empty() {
        warnings.push("Vulkan loader found no physical devices".to_string());
    }

    let version_text = loader_version.map(format_vulkan_version);
    Ok(VulkanInfo {
        available: true,
        loader_version: version_text.clone(),
        api_version: version_text,
        devices,
        warnings,
    })
}

#[cfg(feature = "vulkan")]
fn format_vulkan_version(version: u32) -> String {
    format!(
        "{}.{}.{}",
        (version >> 22) & 0x7f,
        (version >> 12) & 0x3ff,
        version & 0xfff
    )
}

#[cfg(not(feature = "vulkan"))]
pub async fn query(_cancel: &CancellationToken) -> QueryResult<VulkanInfo> {
    Ok(VulkanInfo {
        available: false,
        loader_version: None,
        api_version: None,
        devices: Vec::new(),
        warnings: vec!["compiled without the `vulkan` feature".to_string()],
    })
}
