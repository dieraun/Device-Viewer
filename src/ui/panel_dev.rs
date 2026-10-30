//! Developer utilities: Vulkan snippets, extension notes and the internal log.

use egui::RichText;

use crate::i18n::I18n;
use crate::log;
use crate::state::{AppState, CodeLanguage};
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.dev", "page.dev.sub");

    let selected = state
        .snapshot
        .vulkan
        .as_ref()
        .and_then(|info| info.devices.get(state.ui.selected_vulkan_device))
        .cloned();

    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("dev.snippets")).strong());
            ui.selectable_value(&mut state.ui.dev_language, CodeLanguage::C, "C / Vulkan");
            ui.selectable_value(&mut state.ui.dev_language, CodeLanguage::Rust, "Rust / ash");
        });
        let (title, code) = match state.ui.dev_language {
            CodeLanguage::C => (
                "Instance + device enumeration (C)",
                c_snippet(selected.as_ref()),
            ),
            CodeLanguage::Rust => (
                "Instance + device enumeration (ash)",
                rust_snippet(selected.as_ref()),
            ),
        };
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).weak());
            widgets::copy_button(ui, i18n, code.clone());
        });
        egui::ScrollArea::horizontal()
            .max_height(280.0)
            .show(ui, |ui| {
                let mut code_view = code.as_str();
                ui.add(
                    egui::TextEdit::multiline(&mut code_view)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            });
    });

    ui.add_space(8.0);
    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("dev.extensions")).strong());
        let extensions = selected
            .as_ref()
            .map(|device| device.extensions.clone())
            .unwrap_or_default();
        if extensions.is_empty() {
            ui.label(RichText::new(i18n.t("app.none")).weak());
        } else {
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("dev-extension-grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for extension in &extensions {
                                ui.monospace(extension);
                                ui.label(extension_description(extension));
                                ui.end_row();
                            }
                        });
                });
        }
    });

    ui.add_space(8.0);
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("dev.log")).strong());
            if let Some(path) = log::path() {
                ui.label(RichText::new(path.display().to_string()).weak().size(11.0));
            }
            if ui.button(i18n.t("common.clear")).clicked() {
                log::clear();
            }
        });
        let entries = log::entries();
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in entries.iter().rev().take(200) {
                    ui.label(
                        RichText::new(format!(
                            "{} [{}] {}",
                            entry.timestamp_ms,
                            entry.level.label(),
                            entry.message
                        ))
                        .monospace()
                        .size(11.0),
                    );
                }
                if entries.is_empty() {
                    ui.label(RichText::new(i18n.t("app.none")).weak());
                }
            });
    });
}

fn c_snippet(device: Option<&crate::device_query::vulkan::VulkanDevice>) -> String {
    let name = device
        .map(|value| value.name.as_str())
        .unwrap_or("your GPU");
    format!(
        r#"#include <vulkan/vulkan.h>
#include <stdio.h>

int main(void) {{
    VkApplicationInfo app = {{0}};
    app.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    app.pApplicationName = "Device Viewer";
    app.apiVersion = VK_API_VERSION_1_3;

    VkInstanceCreateInfo info = {{0}};
    info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    info.pApplicationInfo = &app;

    VkInstance instance = VK_NULL_HANDLE;
    if (vkCreateInstance(&info, NULL, &instance) != VK_SUCCESS) return 1;

    uint32_t count = 0;
    vkEnumeratePhysicalDevices(instance, &count, NULL);
    VkPhysicalDevice *devices = calloc(count, sizeof(*devices));
    vkEnumeratePhysicalDevices(instance, &count, devices);

    for (uint32_t i = 0; i < count; i++) {{
        VkPhysicalDeviceProperties props;
        vkGetPhysicalDeviceProperties(devices[i], &props);
        printf("%u: %s\n", i, props.deviceName);
    }}

    free(devices);
    vkDestroyInstance(instance, NULL);
    return 0;
}}
/* Target device: {name} */"#
    )
}

fn rust_snippet(device: Option<&crate::device_query::vulkan::VulkanDevice>) -> String {
    let name = device
        .map(|value| value.name.as_str())
        .unwrap_or("your GPU");
    format!(
        r#"use ash::vk;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    unsafe {{
        let entry = ash::Entry::load()?;
        let app_info = vk::ApplicationInfo::default()
            .application_name(c"Device Viewer")
            .api_version(vk::API_VERSION_1_3);
        let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);
        let instance = entry.create_instance(&create_info, None)?;

        for device in instance.enumerate_physical_devices()? {{
            let props = instance.get_physical_device_properties(device);
            let name = props.device_name_as_c_str().to_string_lossy();
            let memory = instance.get_physical_device_memory_properties(device);
            let local: u64 = memory
                .memory_heaps_as_slice()
                .iter()
                .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                .map(|heap| heap.size)
                .sum();
            println!("{{name}} — {{}} MiB VRAM", local / 1024 / 1024);
        }}

        instance.destroy_instance(None);
    }}
    Ok(())
}}
// Target device: {name}"#
    )
}

fn extension_description(name: &str) -> &'static str {
    if name == "VK_KHR_swapchain" {
        "Presentation swapchain; required by the surface/swapchain path."
    } else if name == "VK_EXT_memory_budget" {
        "Per-heap budget and usage counters, used by the live VRAM chart."
    } else if name == "VK_KHR_acceleration_structure" {
        "Ray-tracing acceleration structures."
    } else if name == "VK_KHR_ray_tracing_pipeline" {
        "Ray-tracing shader stages and pipelines."
    } else if name == "VK_KHR_dynamic_rendering" {
        "Render without VkRenderPass/VkFramebuffer objects."
    } else if name == "VK_KHR_synchronization2" {
        "Revised stage/access masks and submission barriers."
    } else if name == "VK_EXT_descriptor_indexing" {
        "Bindless-style descriptor indexing beyond the core limits."
    } else if name == "VK_KHR_buffer_device_address" {
        "GPU virtual addresses for buffers, used by bindless designs."
    } else if name == "VK_KHR_maintenance1"
        || name == "VK_KHR_maintenance2"
        || name == "VK_KHR_maintenance3"
        || name == "VK_KHR_maintenance4"
    {
        "Maintenance revision that clarifies or extends core behavior."
    } else if name.starts_with("VK_KHR_") {
        "Khronos ratified extension."
    } else if name.starts_with("VK_EXT_") {
        "Vendor-neutral EXT extension."
    } else if name.starts_with("VK_NV_") {
        "NVIDIA vendor extension."
    } else if name.starts_with("VK_AMD_") {
        "AMD vendor extension."
    } else if name.starts_with("VK_INTEL_") {
        "Intel vendor extension."
    } else {
        "Vendor or platform extension; consult the Vulkan registry."
    }
}
