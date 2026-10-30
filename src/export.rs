//! Snapshot rendering for TXT, Markdown and JSON exports.
//!
//! Rendering is pure: it takes a [`HardwareSnapshot`] and returns a string. The
//! GUI uses the same code for its preview and for clipboard tables, and the CLI
//! uses it for `--out`.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::device_query::{
    HardwareSnapshot, UnitSystem, format_bytes, format_duration, format_frequency,
};
use crate::i18n::I18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Txt,
    Markdown,
    Json,
}

impl ExportFormat {
    pub const ALL: [Self; 3] = [Self::Txt, Self::Markdown, Self::Json];

    pub const fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Markdown => "md",
            Self::Json => "json",
        }
    }

    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Txt => "export.txt",
            Self::Markdown => "export.md",
            Self::Json => "export.json",
        }
    }
}

pub fn render(
    snapshot: &HardwareSnapshot,
    format: ExportFormat,
    i18n: I18n,
    units: UnitSystem,
) -> String {
    match format {
        ExportFormat::Json => {
            serde_json::to_string_pretty(snapshot).unwrap_or_else(|_| "{}".to_string())
        }
        ExportFormat::Markdown => render_markdown(snapshot, i18n, units),
        ExportFormat::Txt => markdown_to_text(&render_markdown(snapshot, i18n, units)),
    }
}

pub fn markdown_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut output = String::new();
    output.push('|');
    for header in headers {
        let _ = write!(output, " {} |", escape_cell(header));
    }
    output.push('\n');
    output.push('|');
    for _ in headers {
        output.push_str(" --- |");
    }
    output.push('\n');
    for row in rows {
        output.push('|');
        for cell in row {
            let _ = write!(output, " {} |", escape_cell(cell));
        }
        output.push('\n');
    }
    output
}

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn render_markdown(snapshot: &HardwareSnapshot, i18n: I18n, units: UnitSystem) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "# {}\n", i18n.t("app.title"));
    let _ = writeln!(
        output,
        "{}: {}\n",
        i18n.t("app.last_updated"),
        crate::state::format_timestamp(snapshot.captured_at_unix_ms)
    );
    let _ = writeln!(
        output,
        "- {}: {} ms",
        i18n.t("app.elapsed"),
        snapshot.elapsed_ms
    );
    let _ = writeln!(
        output,
        "- {}: {}",
        i18n.t("app.warnings"),
        snapshot.warning_count()
    );
    let _ = writeln!(
        output,
        "- {}: {}\n",
        i18n.t("app.errors"),
        snapshot.error_count()
    );

    if let Some(cpu) = &snapshot.cpu {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.cpu"));
        let mut rows = vec![
            vec![i18n.t("common.vendor").to_string(), cpu.vendor.clone()],
            vec![i18n.t("common.model").to_string(), cpu.brand.clone()],
            vec![
                i18n.t("common.architecture").to_string(),
                cpu.architecture.clone(),
            ],
            vec![
                i18n.t("common.cores").to_string(),
                format!(
                    "{} physical / {} logical",
                    cpu.physical_cores, cpu.logical_cores
                ),
            ],
            vec![
                i18n.t("common.usage").to_string(),
                format!("{:.1}%", cpu.global_usage_percent),
            ],
        ];
        if let Some(load) = cpu.load_average {
            rows.push(vec![
                i18n.t("common.load_average").to_string(),
                format!("{:.2} / {:.2} / {:.2}", load[0], load[1], load[2]),
            ]);
        }
        output.push_str(&markdown_table(
            &[i18n.t("common.name"), i18n.t("common.value")],
            &rows,
        ));
        output.push('\n');
        if !cpu.cores.is_empty() {
            let rows: Vec<Vec<String>> = cpu
                .cores
                .iter()
                .map(|core| {
                    vec![
                        core.id.to_string(),
                        format!("{:.1}%", core.usage_percent),
                        format_frequency(core.frequency_mhz),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &["#", i18n.t("common.usage"), i18n.t("common.frequency")],
                &rows,
            ));
            output.push('\n');
        }
    }

    if let Some(memory) = &snapshot.memory {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.overview"));
        let rows = vec![
            vec![
                i18n.t("common.total").to_string(),
                format_bytes(memory.total_bytes, units),
            ],
            vec![
                i18n.t("common.used").to_string(),
                format_bytes(memory.used_bytes, units),
            ],
            vec![
                i18n.t("common.free").to_string(),
                format_bytes(memory.free_bytes, units),
            ],
            vec![
                i18n.t("common.available").to_string(),
                format_bytes(memory.available_bytes, units),
            ],
            vec![
                "Swap".to_string(),
                format!(
                    "{} / {}",
                    format_bytes(memory.swap_used_bytes, units),
                    format_bytes(memory.swap_total_bytes, units)
                ),
            ],
        ];
        output.push_str(&markdown_table(
            &[i18n.t("common.name"), i18n.t("common.value")],
            &rows,
        ));
        output.push('\n');
        if !memory.modules.is_empty() {
            let rows: Vec<Vec<String>> = memory
                .modules
                .iter()
                .map(|module| {
                    vec![
                        module.slot.clone().unwrap_or_else(|| "-".to_string()),
                        module
                            .size_bytes
                            .map(|value| format_bytes(value, units))
                            .unwrap_or_else(|| "-".to_string()),
                        module
                            .speed_mt_s
                            .map(|value| format!("{value} MT/s"))
                            .unwrap_or_else(|| "-".to_string()),
                        module
                            .memory_type
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                        module
                            .manufacturer
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[
                    "Slot",
                    i18n.t("common.capacity"),
                    i18n.t("common.speed"),
                    i18n.t("common.type"),
                    i18n.t("common.vendor"),
                ],
                &rows,
            ));
        }
        output.push('\n');
    }

    if let Some(pcie) = &snapshot.pcie {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.pcie"));
        let rows: Vec<Vec<String>> = pcie
            .devices
            .iter()
            .map(|device| {
                let link = device
                    .link
                    .as_ref()
                    .map(|link| {
                        format!(
                            "{}{} / {}{}",
                            link.current_speed.as_deref().unwrap_or("?"),
                            link.current_width
                                .map(|width| format!(" x{width}"))
                                .unwrap_or_default(),
                            link.max_speed.as_deref().unwrap_or("?"),
                            link.max_width
                                .map(|width| format!(" x{width}"))
                                .unwrap_or_default()
                        )
                    })
                    .unwrap_or_else(|| "-".to_string());
                vec![
                    device.address.clone(),
                    format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                    device
                        .vendor_name
                        .clone()
                        .or_else(|| device.device_name.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    device.class_name.clone(),
                    device.driver.clone().unwrap_or_else(|| "-".to_string()),
                    link,
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.address"),
                "VID:PID",
                i18n.t("common.name"),
                i18n.t("common.class"),
                i18n.t("common.driver"),
                i18n.t("common.link"),
            ],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(vulkan) = &snapshot.vulkan {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.vulkan"));
        let _ = writeln!(
            output,
            "- {}: {}",
            i18n.t("common.available"),
            yes_no(vulkan.available, i18n)
        );
        if let Some(version) = &vulkan.loader_version {
            let _ = writeln!(output, "- Loader: {version}");
        }
        let rows: Vec<Vec<String>> = vulkan
            .devices
            .iter()
            .map(|device| {
                vec![
                    device.name.clone(),
                    format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                    device.device_type.clone(),
                    device.api_version.clone(),
                    format_bytes(device.total_memory_bytes, units),
                    device
                        .used_memory_bytes
                        .map(|value| format_bytes(value, units))
                        .unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.device"),
                "VID:PID",
                i18n.t("common.type"),
                "API",
                i18n.t("common.memory"),
                i18n.t("common.used"),
            ],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(opengl) = &snapshot.opengl {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.opengl"));
        let rows = vec![
            vec![
                i18n.t("common.available").to_string(),
                yes_no(opengl.available, i18n).to_string(),
            ],
            vec![
                i18n.t("common.vendor").to_string(),
                opengl.vendor.clone().unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                "Renderer".to_string(),
                opengl.renderer.clone().unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                i18n.t("common.version").to_string(),
                opengl.version.clone().unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                "GLSL".to_string(),
                opengl
                    .shading_language_version
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
        ];
        output.push_str(&markdown_table(
            &[i18n.t("common.name"), i18n.t("common.value")],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(opencl) = &snapshot.opencl {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.opencl"));
        for platform in &opencl.platforms {
            let _ = writeln!(output, "### {}\n", platform.name);
            let rows: Vec<Vec<String>> = platform
                .devices
                .iter()
                .map(|device| {
                    vec![
                        device.name.clone(),
                        device.device_type.clone(),
                        device.max_compute_units.to_string(),
                        format_bytes(device.global_memory_bytes, units),
                        device.driver_version.clone(),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[
                    i18n.t("common.device"),
                    i18n.t("common.type"),
                    i18n.t("common.compute_units"),
                    i18n.t("common.global_memory"),
                    i18n.t("common.driver"),
                ],
                &rows,
            ));
            output.push('\n');
        }
    }

    if let Some(npu) = &snapshot.npu {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.npu"));
        let rows: Vec<Vec<String>> = npu
            .devices
            .iter()
            .map(|device| {
                vec![
                    device.name.clone(),
                    device.address.clone(),
                    device.vendor.clone().unwrap_or_else(|| "-".to_string()),
                    device.class_name.clone(),
                    device.driver.clone().unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.device"),
                i18n.t("common.address"),
                i18n.t("common.vendor"),
                i18n.t("common.class"),
                i18n.t("common.driver"),
            ],
            &rows,
        ));
        if !npu.runtime_libraries.is_empty() {
            let _ = writeln!(
                output,
                "\n{}: {}\n",
                i18n.t("common.libraries"),
                npu.runtime_libraries.join(", ")
            );
        }
    }

    if let Some(usb) = &snapshot.usb {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.usb"));
        let rows: Vec<Vec<String>> = usb
            .devices
            .iter()
            .map(|device| {
                vec![
                    device
                        .bus
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    device
                        .address
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    format!(
                        "{:04X}:{:04X}",
                        device.vendor_id.unwrap_or(0),
                        device.product_id.unwrap_or(0)
                    ),
                    device
                        .vendor
                        .clone()
                        .or_else(|| device.product.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    device.class.clone().unwrap_or_else(|| "-".to_string()),
                    device
                        .speed_mbps
                        .map(|value| format!("{value:.1} Mb/s"))
                        .unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("usb.bus"),
                i18n.t("usb.port"),
                "VID:PID",
                i18n.t("usb.product"),
                i18n.t("common.class"),
                i18n.t("common.speed"),
            ],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(storage) = &snapshot.storage {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.storage"));
        let rows: Vec<Vec<String>> = storage
            .devices
            .iter()
            .map(|device| {
                let health = device
                    .health
                    .as_ref()
                    .and_then(|health| health.passed)
                    .map(|passed| {
                        if passed {
                            i18n.t("storage.smart_passed")
                        } else {
                            i18n.t("storage.smart_failed")
                        }
                    })
                    .unwrap_or_else(|| i18n.t("app.unknown"));
                vec![
                    device.model.clone().unwrap_or_else(|| device.name.clone()),
                    device
                        .capacity_bytes
                        .map(|value| format_bytes(value, units))
                        .unwrap_or_else(|| "-".to_string()),
                    device.protocol.clone().unwrap_or_else(|| "-".to_string()),
                    device.firmware.clone().unwrap_or_else(|| "-".to_string()),
                    health.to_string(),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.model"),
                i18n.t("common.capacity"),
                i18n.t("common.interface"),
                i18n.t("common.firmware"),
                i18n.t("common.health"),
            ],
            &rows,
        ));
        output.push('\n');
        for device in &storage.devices {
            if let Some(health) = &device.health {
                let _ = writeln!(output, "### {}\n", device.name);
                let rows = vec![
                    vec![
                        i18n.t("common.temperature").to_string(),
                        health
                            .temperature_c
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| "-".to_string()),
                    ],
                    vec![
                        i18n.t("common.power_on").to_string(),
                        health
                            .power_on_hours
                            .map(|value| format!("{value} h"))
                            .unwrap_or_else(|| "-".to_string()),
                    ],
                    vec![
                        i18n.t("common.written").to_string(),
                        health
                            .total_written_bytes
                            .map(|value| format_bytes(value, units))
                            .unwrap_or_else(|| "-".to_string()),
                    ],
                    vec![
                        i18n.t("common.bad_blocks").to_string(),
                        health
                            .bad_blocks
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ],
                    vec![
                        i18n.t("common.percentage_used").to_string(),
                        health
                            .percentage_used
                            .map(|value| format!("{value}%"))
                            .unwrap_or_else(|| "-".to_string()),
                    ],
                ];
                output.push_str(&markdown_table(
                    &[i18n.t("common.name"), i18n.t("common.value")],
                    &rows,
                ));
                output.push('\n');
            }
        }
    }

    if let Some(motherboard) = &snapshot.motherboard {
        let _ = writeln!(output, "## {}\n", i18n.t("overview.motherboard"));
        let rows = vec![
            vec![
                i18n.t("common.vendor").to_string(),
                motherboard
                    .vendor
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                i18n.t("common.model").to_string(),
                motherboard.name.clone().unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                i18n.t("common.version").to_string(),
                motherboard
                    .version
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                i18n.t("common.serial").to_string(),
                motherboard
                    .serial
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                "BIOS".to_string(),
                motherboard
                    .bios
                    .as_ref()
                    .map(|bios| {
                        format!(
                            "{} {} ({})",
                            bios.vendor.as_deref().unwrap_or("-"),
                            bios.version.as_deref().unwrap_or("-"),
                            bios.release_date.as_deref().unwrap_or("-")
                        )
                    })
                    .unwrap_or_else(|| "-".to_string()),
            ],
        ];
        output.push_str(&markdown_table(
            &[i18n.t("common.name"), i18n.t("common.value")],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(display) = &snapshot.display {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.display"));
        let rows: Vec<Vec<String>> = display
            .monitors
            .iter()
            .map(|monitor| {
                vec![
                    monitor.name.clone(),
                    format!("{} x {}", monitor.width, monitor.height),
                    monitor
                        .refresh_hz
                        .map(|value| format!("{value} Hz"))
                        .unwrap_or_else(|| "-".to_string()),
                    format!("{}, {}", monitor.position_x, monitor.position_y),
                    yes_no(monitor.primary, i18n).to_string(),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.name"),
                i18n.t("display.resolution"),
                i18n.t("common.refresh_rate"),
                i18n.t("common.position"),
                i18n.t("common.primary"),
            ],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(directx) = &snapshot.directx {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.directx"));
        let rows: Vec<Vec<String>> = directx
            .adapters
            .iter()
            .map(|adapter| {
                vec![
                    adapter.name.clone(),
                    format!("{:04X}:{:04X}", adapter.vendor_id, adapter.device_id),
                    format_bytes(adapter.dedicated_video_memory, units),
                    format_bytes(adapter.shared_system_memory, units),
                    yes_no(adapter.software, i18n).to_string(),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.name"),
                "VID:PID",
                i18n.t("common.dedicated"),
                i18n.t("common.shared"),
                i18n.t("common.software"),
            ],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(metal) = &snapshot.metal {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.metal"));
        let rows: Vec<Vec<String>> = metal
            .devices
            .iter()
            .map(|device| {
                vec![
                    device.name.clone(),
                    device.registry_id.to_string(),
                    yes_no(device.unified_memory, i18n).to_string(),
                    format_bytes(device.recommended_max_working_set_bytes, units),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("common.name"),
                "Registry ID",
                "Unified memory",
                i18n.t("common.max"),
            ],
            &rows,
        ));
        output.push('\n');
    }

    if let Some(openvx) = &snapshot.openvx {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.openvx"));
        let _ = writeln!(
            output,
            "- {}: {}\n",
            i18n.t("common.available"),
            yes_no(openvx.available, i18n)
        );
    }

    if let Some(system) = &snapshot.system {
        let _ = writeln!(output, "## {}\n", i18n.t("overview.system"));
        let rows = vec![
            vec![
                i18n.t("common.name").to_string(),
                system.os.long_name.clone(),
            ],
            vec![
                i18n.t("common.version").to_string(),
                system.os.version.clone(),
            ],
            vec![
                i18n.t("common.kernel").to_string(),
                system.os.kernel_version.clone(),
            ],
            vec![
                i18n.t("common.hostname").to_string(),
                system.os.hostname.clone(),
            ],
            vec![
                i18n.t("common.architecture").to_string(),
                system.os.architecture.clone(),
            ],
            vec![
                i18n.t("common.uptime").to_string(),
                system
                    .os
                    .uptime_seconds
                    .map(format_duration)
                    .unwrap_or_else(|| "-".to_string()),
            ],
        ];
        output.push_str(&markdown_table(
            &[i18n.t("common.name"), i18n.t("common.value")],
            &rows,
        ));
        output.push('\n');
        if !system.networks.is_empty() {
            let rows: Vec<Vec<String>> = system
                .networks
                .iter()
                .map(|interface| {
                    vec![
                        interface.name.clone(),
                        interface.mac_address.clone(),
                        interface.addresses.join(", "),
                        format_bytes(interface.received_bytes, units),
                        format_bytes(interface.transmitted_bytes, units),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[
                    i18n.t("common.name"),
                    i18n.t("common.mac"),
                    i18n.t("common.address"),
                    i18n.t("common.received"),
                    i18n.t("common.transmitted"),
                ],
                &rows,
            ));
            output.push('\n');
        }
        if !system.audio.is_empty() {
            let rows: Vec<Vec<String>> = system
                .audio
                .iter()
                .map(|device| {
                    vec![
                        device.name.clone(),
                        device.vendor.clone().unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[i18n.t("common.name"), i18n.t("common.vendor")],
                &rows,
            ));
            output.push('\n');
        }
    }

    if let Some(sensors) = &snapshot.sensors {
        let _ = writeln!(output, "## {}\n", i18n.t("nav.sensors"));
        if !sensors.temperatures.is_empty() {
            let rows: Vec<Vec<String>> = sensors
                .temperatures
                .iter()
                .map(|sensor| {
                    vec![
                        sensor.kind.clone(),
                        sensor.label.clone(),
                        format!("{:.1} °C", sensor.celsius),
                        sensor
                            .max_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| "-".to_string()),
                        sensor
                            .critical_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[
                    i18n.t("common.type"),
                    i18n.t("common.name"),
                    i18n.t("common.temperature"),
                    i18n.t("common.max"),
                    i18n.t("common.critical"),
                ],
                &rows,
            ));
            output.push('\n');
        }
        if !sensors.fans.is_empty() {
            let rows: Vec<Vec<String>> = sensors
                .fans
                .iter()
                .map(|fan| {
                    vec![
                        fan.label.clone(),
                        fan.rpm
                            .map(|value| format!("{value} RPM"))
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[i18n.t("common.name"), i18n.t("common.rpm")],
                &rows,
            ));
            output.push('\n');
        }
        if !sensors.voltages.is_empty() {
            let rows: Vec<Vec<String>> = sensors
                .voltages
                .iter()
                .map(|voltage| {
                    vec![
                        voltage.label.clone(),
                        voltage
                            .volts
                            .map(|value| format!("{value:.3} V"))
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            output.push_str(&markdown_table(
                &[i18n.t("common.name"), i18n.t("common.volts")],
                &rows,
            ));
            output.push('\n');
        }
    }

    if let Some(bandwidth) = &snapshot.bandwidth {
        let _ = writeln!(output, "## {}\n", i18n.t("overview.bandwidth"));
        let rows = vec![
            vec![
                i18n.t("common.read").to_string(),
                bandwidth
                    .read_megabytes_per_second
                    .map(|value| format!("{value:.1} MB/s"))
                    .unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                i18n.t("common.write").to_string(),
                bandwidth
                    .write_megabytes_per_second
                    .map(|value| format!("{value:.1} MB/s"))
                    .unwrap_or_else(|| "-".to_string()),
            ],
            vec![
                i18n.t("common.copy").to_string(),
                bandwidth
                    .copy_megabytes_per_second
                    .map(|value| format!("{value:.1} MB/s"))
                    .unwrap_or_else(|| "-".to_string()),
            ],
        ];
        output.push_str(&markdown_table(
            &[i18n.t("common.operation"), i18n.t("common.speed")],
            &rows,
        ));
        output.push('\n');
    }

    if !snapshot.failures.is_empty() {
        let _ = writeln!(output, "## {}\n", i18n.t("overview.failures"));
        let rows: Vec<Vec<String>> = snapshot
            .failures
            .iter()
            .map(|failure| {
                vec![
                    failure.module.clone(),
                    failure.level.as_str().to_string(),
                    failure.message.clone(),
                    failure.detail.clone().unwrap_or_default(),
                ]
            })
            .collect();
        output.push_str(&markdown_table(
            &[
                i18n.t("warnings.module"),
                i18n.t("warnings.level"),
                i18n.t("warnings.message"),
                i18n.t("warnings.detail"),
            ],
            &rows,
        ));
    }

    output
}

fn yes_no(value: bool, i18n: I18n) -> &'static str {
    if value {
        i18n.t("app.yes")
    } else {
        i18n.t("app.no")
    }
}

fn markdown_to_text(markdown: &str) -> String {
    let mut output = String::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("| ---") || trimmed.starts_with("|---") {
            continue;
        }
        if trimmed.starts_with('|') {
            let cells: Vec<&str> = trimmed
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            output.push_str(&cells.join("\t"));
            output.push('\n');
            continue;
        }
        let cleaned = trimmed.trim_start_matches('#').trim();
        let cleaned = cleaned.replace("**", "").replace('`', "");
        output.push_str(&cleaned);
        output.push('\n');
    }
    output
}
