//! Minimal, dependency-free localization for English and Simplified Chinese.
//!
//! The UI looks strings up by a stable key. Missing keys fall back to the key
//! itself, which makes omissions visible during development instead of showing
//! an empty label.

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    English,
    Chinese,
}

impl Language {
    /// Picks the operating-system language when it is Chinese or English,
    /// otherwise falls back to the POSIX locale variables.
    pub fn from_system() -> Self {
        #[cfg(target_os = "windows")]
        if let Some(language) = windows_language() {
            return language;
        }

        #[cfg(target_os = "macos")]
        if let Some(language) = macos_language() {
            return language;
        }

        locale_language()
    }
}

#[cfg(target_os = "windows")]
fn windows_language() -> Option<Language> {
    use windows::Win32::Globalization::GetUserDefaultUILanguage;

    let language_id = unsafe { GetUserDefaultUILanguage() };
    match language_id & 0x03ff {
        0x04 => Some(Language::Chinese),
        0x09 => Some(Language::English),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn macos_language() -> Option<Language> {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    if value.contains("zh") {
        Some(Language::Chinese)
    } else if value.contains("en") {
        Some(Language::English)
    } else {
        None
    }
}

fn locale_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            let value = value.to_ascii_lowercase();
            if value.starts_with("zh") || value.contains("chinese") {
                return Language::Chinese;
            }
            if value.starts_with("en") || value.contains("english") {
                return Language::English;
            }
        }
    }
    Language::English
}

/// The persisted language choice. `System` resolves on startup and can be
/// revisited from Settings without losing the explicit English/Chinese choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LanguagePreference {
    #[default]
    System,
    English,
    Chinese,
}

impl LanguagePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::English, Self::Chinese];

    pub const fn label_key(self) -> &'static str {
        match self {
            Self::System => "language.system",
            Self::English => "language.english",
            Self::Chinese => "language.chinese",
        }
    }

    pub fn resolve(self) -> Language {
        match self {
            Self::System => Language::from_system(),
            Self::English => Language::English,
            Self::Chinese => Language::Chinese,
        }
    }

    pub const fn cycle(self) -> Self {
        match self {
            Self::System => Self::English,
            Self::English => Self::Chinese,
            Self::Chinese => Self::System,
        }
    }
}

impl From<Language> for LanguagePreference {
    fn from(language: Language) -> Self {
        match language {
            Language::English => Self::English,
            Language::Chinese => Self::Chinese,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new(preference: impl Into<LanguagePreference>) -> Self {
        Self {
            language: preference.into().resolve(),
        }
    }

    pub fn set_preference(&mut self, preference: LanguagePreference) {
        let language = preference.resolve();
        if self.language != language {
            self.language = language;
        }
    }

    pub fn t(self, key: &'static str) -> &'static str {
        for (candidate, en, zh) in STRINGS {
            if *candidate == key {
                return match self.language {
                    Language::English => en,
                    Language::Chinese => zh,
                };
            }
        }
        key
    }
}

/// Adds a CJK-capable system font as a fallback. This is deliberately
/// best-effort: a missing font only affects glyph coverage, never startup.
pub fn install_fonts(ctx: &egui::Context) {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\msyhbd.ttc",
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    } else {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
        ]
    };

    let Some(bytes) = candidates
        .iter()
        .find_map(|path| std::fs::read(Path::new(path)).ok())
    else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    let name = "device-viewer-cjk".to_owned();
    fonts.font_data.insert(
        name.clone(),
        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push(name.clone());
    }
    ctx.set_fonts(fonts);
}

const STRINGS: &[(&str, &str, &str)] = &[
    ("app.title", "Device Viewer", "设备查看器"),
    (
        "app.subtitle",
        "Cross-platform hardware inspector",
        "跨平台硬件信息查看工具",
    ),
    ("app.version", "Version", "版本"),
    ("app.refresh", "Refresh", "刷新"),
    (
        "app.refreshing",
        "Collecting hardware...",
        "正在采集硬件信息…",
    ),
    ("app.export", "Export", "导出"),
    ("app.copy", "Copy", "复制"),
    ("app.copied", "Copied to clipboard", "已复制到剪贴板"),
    ("app.close", "Close", "关闭"),
    ("app.save", "Save", "保存"),
    ("app.cancel", "Cancel", "取消"),
    ("app.delete", "Delete", "删除"),
    ("app.edit", "Edit", "编辑"),
    ("app.load", "Load", "加载"),
    ("app.compare", "Compare", "对比"),
    ("app.preview", "Preview", "预览"),
    ("app.open", "Open", "打开"),
    ("app.apply", "Apply", "应用"),
    ("app.clear", "Clear", "清空"),
    ("app.search", "Search", "搜索"),
    ("app.filter", "Filter", "筛选"),
    ("app.all", "All", "全部"),
    ("app.none", "None", "无"),
    ("app.unknown", "Unknown", "未知"),
    ("app.yes", "Yes", "是"),
    ("app.no", "No", "否"),
    ("app.enabled", "Enabled", "已启用"),
    ("app.disabled", "Disabled", "已禁用"),
    ("app.ready", "Ready", "就绪"),
    ("app.not_available", "Not available", "不可用"),
    ("app.warning", "Warning", "警告"),
    ("app.warnings", "Warnings", "警告"),
    ("app.errors", "Errors", "错误"),
    ("app.n_a", "N/A", "不适用"),
    ("app.items", "items", "项"),
    ("app.seconds", "seconds", "秒"),
    ("app.milliseconds", "ms", "毫秒"),
    ("app.platform", "Platform", "平台"),
    ("app.elevated", "Elevated", "已提权"),
    ("app.standard_user", "Standard user", "标准用户"),
    ("app.last_updated", "Updated", "更新于"),
    ("app.elapsed", "Collection time", "采集耗时"),
    ("app.bytes", "Bytes", "字节"),
    ("app.details", "Details", "详情"),
    ("app.overview", "Overview", "概览"),
    ("app.hardware", "Hardware", "硬件"),
    ("app.system", "System", "系统"),
    ("app.tools", "Tools", "工具"),
    ("app.theme", "Theme", "主题"),
    ("app.language", "Language", "语言"),
    ("app.units", "Units", "单位"),
    ("app.dark", "Dark", "深色"),
    ("app.light", "Light", "浅色"),
    ("app.portable", "Portable mode", "便携模式"),
    ("app.minimize", "Minimize", "最小化"),
    ("app.show_details", "Show details", "显示详情"),
    ("app.hide_details", "Hide details", "隐藏详情"),
    ("nav.overview", "Overview", "概览"),
    ("nav.cpu", "CPU", "处理器"),
    ("nav.pcie", "PCI / PCIe", "PCI / PCIe"),
    ("nav.vulkan", "Vulkan", "Vulkan"),
    ("nav.opengl", "OpenGL", "OpenGL"),
    ("nav.opencl", "OpenCL", "OpenCL"),
    ("nav.npu", "NPU / AI", "NPU / AI"),
    ("nav.usb", "USB", "USB"),
    ("nav.storage", "Storage", "存储"),
    ("nav.display", "Display", "显示"),
    ("nav.directx", "DirectX", "DirectX"),
    ("nav.metal", "Metal", "Metal"),
    ("nav.openvx", "OpenVX", "OpenVX"),
    ("nav.sensors", "Sensors", "传感器"),
    ("nav.network", "Network", "网络"),
    ("nav.audio", "Audio", "音频"),
    ("nav.checklist", "Checklist", "能力清单"),
    ("nav.dev", "Developer", "开发者"),
    ("nav.snapshots", "Snapshots", "快照"),
    ("nav.compare", "Compare", "对比"),
    ("nav.settings", "Settings", "设置"),
    ("group.monitoring", "Monitoring", "监测"),
    ("group.system", "System", "系统"),
    ("group.tools", "Tools", "工具"),
    (
        "page.overview.sub",
        "A live summary of this machine",
        "本机硬件实时概览",
    ),
    (
        "page.cpu.sub",
        "Topology, clocks and per-core load",
        "拓扑、频率与每核心负载",
    ),
    (
        "page.pcie.sub",
        "PCI buses, functions and link status",
        "PCI 总线、功能与链路状态",
    ),
    (
        "page.vulkan.sub",
        "Adapters, heaps, budget and live GPU curves",
        "适配器、堆、预算与 GPU 实时曲线",
    ),
    (
        "page.opengl.sub",
        "Renderer capabilities and limits",
        "渲染器能力与限制",
    ),
    (
        "page.opencl.sub",
        "Platforms, devices and compute capabilities",
        "平台、设备与计算能力",
    ),
    (
        "page.npu.sub",
        "AI accelerators and runtime libraries",
        "AI 加速器与运行时库",
    ),
    (
        "page.usb.sub",
        "Buses, devices and descriptors",
        "总线、设备与描述符",
    ),
    (
        "page.storage.sub",
        "NVMe / SATA identity and SMART health",
        "NVMe / SATA 标识与 SMART 健康",
    ),
    (
        "page.display.sub",
        "Monitors, modes and adapters",
        "显示器、模式与适配器",
    ),
    (
        "page.directx.sub",
        "DXGI adapters and dedicated memory",
        "DXGI 适配器与专用显存",
    ),
    ("page.metal.sub", "Apple Metal devices", "Apple Metal 设备"),
    (
        "page.openvx.sub",
        "OpenVX runtime availability",
        "OpenVX 运行时可用性",
    ),
    (
        "page.sensors.sub",
        "Temperatures, fans and voltages",
        "温度、风扇与电压",
    ),
    (
        "page.network.sub",
        "Interfaces, addresses and counters",
        "网卡、地址与计数器",
    ),
    (
        "page.audio.sub",
        "Playback and capture devices",
        "播放与采集设备",
    ),
    (
        "page.checklist.sub",
        "Capability and permission matrix",
        "能力与权限矩阵",
    ),
    (
        "page.dev.sub",
        "Vulkan snippets, extension notes and logs",
        "Vulkan 代码、扩展说明与日志",
    ),
    (
        "page.snapshots.sub",
        "Saved .dvsnap snapshots and tags",
        "已保存的 .dvsnap 快照与标签",
    ),
    (
        "page.compare.sub",
        "Field-level differences between snapshots",
        "快照之间的字段级差异",
    ),
    (
        "page.settings.sub",
        "Appearance, language, units and storage",
        "外观、语言、单位与存储",
    ),
    ("common.vendor", "Vendor", "厂商"),
    ("common.model", "Model", "型号"),
    ("common.name", "Name", "名称"),
    ("common.version", "Version", "版本"),
    ("common.driver", "Driver", "驱动"),
    ("common.memory", "Memory", "内存"),
    ("common.usage", "Usage", "使用率"),
    ("common.temperature", "Temperature", "温度"),
    ("common.fan", "Fan", "风扇"),
    ("common.voltage", "Voltage", "电压"),
    ("common.serial", "Serial", "序列号"),
    ("common.capacity", "Capacity", "容量"),
    ("common.firmware", "Firmware", "固件"),
    ("common.interface", "Interface", "接口"),
    ("common.address", "Address", "地址"),
    ("common.class", "Class", "类别"),
    ("common.status", "Status", "状态"),
    ("common.health", "Health", "健康"),
    ("common.speed", "Speed", "速度"),
    ("common.width", "Width", "宽度"),
    ("common.architecture", "Architecture", "架构"),
    ("common.cores", "Cores", "核心"),
    ("common.threads", "Threads", "线程"),
    ("common.frequency", "Frequency", "频率"),
    ("common.load_average", "Load average", "平均负载"),
    ("common.extensions", "Extensions", "扩展"),
    ("common.platform", "Platform", "平台"),
    ("common.device", "Device", "设备"),
    ("common.devices", "Devices", "设备"),
    ("common.available", "Available", "可用"),
    ("common.total", "Total", "总计"),
    ("common.used", "Used", "已用"),
    ("common.free", "Free", "可用"),
    ("common.type", "Type", "类型"),
    ("common.count", "Count", "数量"),
    ("common.date", "Date", "日期"),
    ("common.hostname", "Hostname", "主机名"),
    ("common.kernel", "Kernel", "内核"),
    ("common.uptime", "Uptime", "运行时间"),
    ("common.mac", "MAC address", "MAC 地址"),
    ("common.received", "Received", "接收"),
    ("common.transmitted", "Transmitted", "发送"),
    ("common.refresh_rate", "Refresh rate", "刷新率"),
    ("common.primary", "Primary", "主显示器"),
    ("common.position", "Position", "位置"),
    ("common.revision", "Revision", "修订"),
    ("common.subsystem", "Subsystem", "子系统"),
    ("common.link", "Link", "链路"),
    ("common.generation", "Generation", "代际"),
    ("common.power_on", "Power-on hours", "通电时间"),
    ("common.written", "Total written", "总写入量"),
    ("common.bad_blocks", "Bad blocks", "坏块"),
    ("common.media_errors", "Media errors", "介质错误"),
    ("common.percentage_used", "Wear level", "磨损度"),
    ("common.power_cycles", "Power cycles", "通电次数"),
    ("common.budget", "Memory budget", "内存预算"),
    ("common.heaps", "Heaps", "堆"),
    ("common.device_local", "Device local", "设备本地"),
    ("common.shared", "Shared", "共享"),
    ("common.software", "Software", "软件"),
    ("common.remote", "Remote", "远程"),
    ("common.dedicated", "Dedicated", "专用"),
    ("common.compute_units", "Compute units", "计算单元"),
    ("common.work_group", "Max work group", "最大工作组"),
    ("common.global_memory", "Global memory", "全局内存"),
    ("common.local_memory", "Local memory", "本地内存"),
    ("common.opencl_c", "OpenCL C", "OpenCL C"),
    ("common.runtime", "Runtime", "运行时"),
    ("common.libraries", "Libraries", "库"),
    ("common.compiler", "Compiler", "编译器"),
    ("common.max", "Max", "最大"),
    ("common.critical", "Critical", "临界"),
    ("common.rpm", "RPM", "转速"),
    ("common.volts", "Volts", "电压"),
    ("common.buffer", "Buffer", "缓冲区"),
    ("common.read", "Read", "读取"),
    ("common.write", "Write", "写入"),
    ("common.copy", "Copy", "复制"),
    ("common.operation", "Operation", "操作"),
    ("common.value", "Value", "数值"),
    ("common.path", "Path", "路径"),
    ("common.size", "Size", "大小"),
    ("common.tags", "Tags", "标签"),
    ("common.result", "Result", "结果"),
    ("common.source", "Source", "来源"),
    ("common.target", "Target", "目标"),
    ("common.change", "Change", "变更"),
    ("common.added", "Added", "新增"),
    ("common.removed", "Removed", "删除"),
    ("common.changed", "Changed", "变更"),
    ("common.identical", "Identical", "相同"),
    ("sensor.cpu", "CPU", "处理器"),
    ("sensor.gpu", "GPU", "显卡"),
    ("sensor.storage", "Storage", "存储"),
    ("sensor.motherboard", "Motherboard", "主板"),
    ("sensor.other", "Other", "其他"),
    (
        "sensor.no_temperatures",
        "No temperature sensors were reported",
        "未报告温度传感器",
    ),
    (
        "sensor.no_fans",
        "No fan sensors were reported",
        "未报告风扇传感器",
    ),
    (
        "sensor.no_voltages",
        "No voltage sensors were reported",
        "未报告电压传感器",
    ),
    (
        "storage.smart_unavailable",
        "SMART data is unavailable",
        "SMART 数据不可用",
    ),
    ("storage.smart_passed", "SMART passed", "SMART 通过"),
    ("storage.smart_failed", "SMART failed", "SMART 失败"),
    (
        "storage.no_devices",
        "No storage devices were reported",
        "未报告存储设备",
    ),
    ("storage.mount", "Mount point", "挂载点"),
    ("storage.file_system", "File system", "文件系统"),
    ("storage.removable", "Removable", "可移动"),
    (
        "pcie.no_devices",
        "No PCI devices were reported",
        "未报告 PCI 设备",
    ),
    (
        "usb.no_devices",
        "No USB devices were reported",
        "未报告 USB 设备",
    ),
    ("usb.bus", "Bus", "总线"),
    ("usb.port", "Port", "端口"),
    ("usb.vid_pid", "VID:PID", "VID:PID"),
    ("usb.product", "Product", "产品"),
    ("usb.manufacturer", "Manufacturer", "制造商"),
    (
        "display.no_monitors",
        "No monitors were reported",
        "未报告显示器",
    ),
    ("display.resolution", "Resolution", "分辨率"),
    ("display.mode", "Mode", "模式"),
    (
        "network.no_interfaces",
        "No network interfaces were reported",
        "未报告网络接口",
    ),
    (
        "audio.no_devices",
        "No audio devices were reported",
        "未报告音频设备",
    ),
    (
        "gpu.no_runtime",
        "Live GPU counters are not available on this system",
        "此系统不提供 GPU 实时计数器",
    ),
    ("gpu.utilization", "GPU utilization", "GPU 使用率"),
    ("gpu.vram", "VRAM usage", "显存占用"),
    ("gpu.history", "Recent samples", "最近采样"),
    (
        "gpu.no_device",
        "No Vulkan device was reported",
        "未报告 Vulkan 设备",
    ),
    ("gpu.no_vulkan_support", "No Vulkan support", "无Vulkan支持"),
    (
        "gpu.no_vulkan_hint",
        "This GPU is not Vulkan-compatible; basic hardware inspection is unaffected.",
        "该GPU不兼容Vulkan，不影响基础硬件读取。",
    ),
    (
        "gpu.budget_supported",
        "Memory budget extension",
        "内存预算扩展",
    ),
    ("gpu.favorite", "Favorite", "收藏"),
    (
        "gpu.favorite_added",
        "Extension added to favorites",
        "扩展已加入收藏",
    ),
    (
        "gpu.favorite_removed",
        "Extension removed from favorites",
        "扩展已从收藏移除",
    ),
    ("checklist.module", "Module", "模块"),
    ("checklist.state", "State", "状态"),
    ("checklist.permission", "Permission", "权限"),
    ("checklist.ok", "Collected", "已采集"),
    ("checklist.warning", "Partial", "部分可用"),
    ("checklist.failed", "Failed", "失败"),
    ("checklist.missing", "No data", "无数据"),
    (
        "checklist.admin",
        "Administrator / root recommended",
        "建议管理员 / root",
    ),
    ("checklist.none", "Standard user", "标准用户"),
    (
        "checklist.refresh_hint",
        "Re-run collection to update every module",
        "重新采集以更新所有模块",
    ),
    ("permission.title", "Permission pre-check", "权限预检"),
    (
        "permission.elevated",
        "The process is elevated. Protected hardware data should be readable.",
        "进程已提权，受保护的硬件数据应可读取。",
    ),
    (
        "permission.standard",
        "The process is not elevated. SMART, low-level PCI and some sensors may be limited.",
        "进程未提权。SMART、底层 PCI 与部分传感器数据可能受限。",
    ),
    (
        "permission.items",
        "Items that may need Administrator / root:",
        "可能需要管理员 / root 的项目：",
    ),
    ("permission.dismiss", "Dismiss", "知道了"),
    (
        "loading.title",
        "Inspecting this machine",
        "正在检测本机硬件",
    ),
    (
        "loading.sub",
        "Collecting CPU, graphics, storage and sensors in parallel",
        "正在并行采集处理器、图形、存储与传感器信息",
    ),
    ("loading.modules", "Active modules", "采集模块"),
    (
        "loading.finalizing",
        "Finalizing snapshot...",
        "正在生成快照…",
    ),
    (
        "snapshot.empty",
        "No snapshots yet. Save the current machine to create one.",
        "尚无快照。保存当前机器信息以创建快照。",
    ),
    (
        "snapshot.save_current",
        "Save current snapshot",
        "保存当前快照",
    ),
    ("snapshot.label", "Label", "标签名称"),
    (
        "snapshot.tags_hint",
        "comma, separated, tags",
        "逗号分隔的标签",
    ),
    ("snapshot.saved", "Snapshot saved", "快照已保存"),
    ("snapshot.deleted", "Snapshot deleted", "快照已删除"),
    (
        "snapshot.import",
        "Import .dvsnap / JSON",
        "导入 .dvsnap / JSON",
    ),
    ("snapshot.import_path", "Path to a snapshot", "快照文件路径"),
    ("snapshot.imported", "Snapshot imported", "快照已导入"),
    (
        "snapshot.drop_hint",
        "Drag a .dvsnap or JSON file here to import it",
        "将 .dvsnap 或 JSON 文件拖到此处以导入",
    ),
    (
        "snapshot.double_click",
        "Double-click a row to compare it with the current machine",
        "双击一行，与当前机器信息对比",
    ),
    ("snapshot.set_compare", "Set as baseline", "设为基准"),
    ("snapshot.baseline", "Baseline", "基准"),
    (
        "snapshot.none_selected",
        "Select a snapshot from the list first",
        "请先从列表中选择快照",
    ),
    ("snapshot.export", "Export snapshot", "导出快照"),
    ("snapshot.size", "Size", "大小"),
    ("snapshot.created", "Created", "创建于"),
    ("snapshot.count", "snapshots", "个快照"),
    ("snapshot.storage", "Snapshot folder", "快照目录"),
    (
        "compare.no_baseline",
        "Load a baseline snapshot to see field-level changes.",
        "加载基准快照以查看字段级变更。",
    ),
    ("compare.filter", "Filter changed paths", "筛选变更路径"),
    ("compare.added_current", "Added in current", "当前新增"),
    (
        "compare.removed_current",
        "Removed since baseline",
        "相对基准删除",
    ),
    ("compare.changed", "Changed", "已变更"),
    ("compare.summary", "Summary", "摘要"),
    (
        "compare.copy_table",
        "Copy Markdown table",
        "复制 Markdown 表格",
    ),
    (
        "compare.no_changes",
        "No field changes detected",
        "未检测到字段变更",
    ),
    ("export.title", "Export snapshot", "导出快照"),
    ("export.format", "Format", "格式"),
    ("export.path", "Destination", "目标文件"),
    ("export.generate", "Generate preview", "生成预览"),
    ("export.save", "Save file", "保存文件"),
    ("export.copy", "Copy to clipboard", "复制到剪贴板"),
    ("export.saved", "Export written", "导出已写入"),
    ("export.failed", "Export failed", "导出失败"),
    ("export.preview", "Preview", "预览"),
    (
        "export.markdown_table",
        "Copy Markdown table",
        "复制 Markdown 表格",
    ),
    ("export.txt", "Plain text", "纯文本"),
    ("export.md", "Markdown", "Markdown"),
    ("export.json", "JSON", "JSON"),
    ("settings.appearance", "Appearance", "外观"),
    ("settings.language", "Language", "语言"),
    ("settings.units", "Unit system", "单位制"),
    ("settings.binary", "Binary (1024)", "二进制（1024）"),
    ("settings.decimal", "Decimal (1000)", "十进制（1000）"),
    ("settings.portable", "Portable mode", "便携模式"),
    (
        "settings.portable_hint",
        "Store configuration and snapshots next to the executable.",
        "将配置与快照保存在程序同级目录。",
    ),
    ("settings.storage", "Storage locations", "存储位置"),
    ("settings.config", "Configuration file", "配置文件"),
    ("settings.snapshots", "Snapshot directory", "快照目录"),
    ("settings.query", "Collection", "采集"),
    ("settings.timeout", "Default module timeout", "默认模块超时"),
    ("settings.filters", "Saved filters", "已保存筛选"),
    (
        "settings.favorites",
        "Favorite Vulkan extensions",
        "收藏的 Vulkan 扩展",
    ),
    ("settings.reset", "Reset settings", "重置设置"),
    ("settings.reset_done", "Settings reset", "设置已重置"),
    (
        "settings.theme_hint",
        "The theme applies immediately.",
        "主题会立即生效。",
    ),
    (
        "settings.path_hint",
        "Portable mode changes where new files are written; existing files are not moved.",
        "便携模式只改变新文件的写入位置，不会移动已有文件。",
    ),
    ("dev.title", "Vulkan developer tools", "Vulkan 开发者工具"),
    ("dev.device", "Target device", "目标设备"),
    ("dev.language", "Snippet language", "代码语言"),
    ("dev.code", "Generated snippet", "生成的代码"),
    ("dev.extensions", "Extension reference", "扩展说明"),
    (
        "dev.no_device",
        "Select a Vulkan device to generate code.",
        "请选择 Vulkan 设备以生成代码。",
    ),
    ("dev.log", "Internal log", "内部日志"),
    ("dev.log_empty", "No log entries yet.", "暂无日志。"),
    ("dev.log_clear", "Clear log", "清空日志"),
    (
        "dev.extension_unknown",
        "Vendor or driver extension. Consult the Vulkan registry for its specification.",
        "厂商或驱动扩展。请查阅 Vulkan 注册表了解规范。",
    ),
    ("dev.favorite", "Favorite", "收藏"),
    (
        "dev.favorite_hint",
        "Favorites are persisted and shown first in extension lists.",
        "收藏会持久保存，并在扩展列表中优先显示。",
    ),
    ("dev.copy_code", "Copy code", "复制代码"),
    ("dev.copy_extensions", "Copy extension list", "复制扩展列表"),
    ("overview.quick", "Quick summary", "快速摘要"),
    ("overview.system", "Operating system", "操作系统"),
    ("overview.memory", "Memory", "内存"),
    ("overview.motherboard", "Motherboard & BIOS", "主板与 BIOS"),
    ("overview.bandwidth", "Memory bandwidth", "内存带宽"),
    ("overview.warnings", "Collection notes", "采集说明"),
    (
        "overview.no_data",
        "No data collected for this section",
        "此部分尚未采集到数据",
    ),
    ("overview.storage_summary", "Storage health", "存储健康"),
    ("overview.gpu_summary", "Graphics", "图形"),
    ("overview.failures", "Module failures", "模块失败"),
    ("warnings.none", "No warnings reported", "无警告"),
    ("warnings.module", "Module", "模块"),
    ("warnings.level", "Level", "级别"),
    ("warnings.message", "Message", "消息"),
    ("warnings.detail", "Detail", "详情"),
    (
        "toast.refreshed",
        "Hardware snapshot refreshed",
        "硬件快照已刷新",
    ),
    ("toast.query_failed", "Collection failed", "采集失败"),
    ("toast.snapshot_loaded", "Snapshot loaded", "快照已加载"),
    (
        "toast.invalid_snapshot",
        "The file is not a valid snapshot",
        "文件不是有效的快照",
    ),
    ("toast.config_saved", "Configuration saved", "配置已保存"),
    (
        "toast.portable_on",
        "Portable mode enabled",
        "便携模式已启用",
    ),
    (
        "toast.portable_off",
        "Portable mode disabled",
        "便携模式已禁用",
    ),
    ("toast.copied", "Copied", "已复制"),
    ("toast.no_data", "Nothing to copy yet", "暂无可复制内容"),
    ("status.ok", "OK", "正常"),
    ("status.warning", "Warning", "警告"),
    ("status.error", "Error", "错误"),
    ("status.unsupported", "Unsupported", "不支持"),
    ("status.unavailable", "Unavailable", "不可用"),
    // Aliases used by panel widgets. Kept here so every visible label resolves.
    ("app.available", "Available", "可用"),
    ("app.collecting", "Collecting", "正在采集"),
    ("checklist.items", "Capability items", "能力项"),
    ("common.adapters", "Adapters", "适配器"),
    ("common.apply", "Apply", "应用"),
    ("common.clear", "Clear", "清除"),
    ("common.delete", "Delete", "删除"),
    ("common.detail", "Detail", "详情"),
    ("common.kind", "Kind", "类型"),
    ("common.label", "Label", "标签"),
    ("common.page", "Page", "页面"),
    ("common.protocol", "Protocol", "协议"),
    ("common.recommended_vram", "Recommended VRAM", "建议显存"),
    ("common.refresh", "Refresh", "刷新"),
    ("common.save", "Save", "保存"),
    ("common.sensors", "Sensors", "传感器"),
    ("common.shared_memory", "Shared memory", "共享内存"),
    ("common.snapshots", "Snapshots", "快照"),
    ("common.system_memory", "System memory", "系统内存"),
    ("common.threadgroup", "Threadgroup", "线程组"),
    ("common.vid_pid", "VID:PID", "厂商:设备"),
    ("common.vram", "VRAM", "显存"),
    ("common.warnings", "Warnings", "警告"),
    ("compare.added", "Added", "新增"),
    ("compare.after", "After", "对比后"),
    ("compare.baseline", "Baseline", "基线"),
    ("compare.before", "Before", "对比前"),
    ("compare.change", "Change", "变化"),
    ("compare.field", "Field", "字段"),
    ("compare.identical", "No differences", "无差异"),
    ("compare.removed", "Removed", "移除"),
    ("dev.snippets", "Code snippets", "代码片段"),
    ("display.adapters", "Display adapters", "显示适配器"),
    ("display.position", "Position", "位置"),
    ("display.primary", "Primary", "主显示器"),
    (
        "loading.hint",
        "This usually takes a few seconds",
        "通常需要几秒钟",
    ),
    (
        "loading.subtitle",
        "Reading hardware in parallel",
        "并行读取硬件信息",
    ),
    ("metal.unified", "Unified memory", "统一内存"),
    ("nav.export", "Export", "导出"),
    ("network.addresses", "Addresses", "地址"),
    ("network.mac", "MAC address", "MAC 地址"),
    ("network.received", "Received", "接收"),
    ("network.transmitted", "Transmitted", "发送"),
    ("openvx.runtime", "OpenVX runtime", "OpenVX 运行时"),
    (
        "permission.standard_hint",
        "Some hardware details need administrator rights",
        "部分硬件信息需要管理员权限",
    ),
    ("sensors.fans", "Fans", "风扇"),
    ("sensors.temperature", "Temperature", "温度"),
    ("sensors.temperatures", "Temperatures", "温度"),
    ("sensors.voltages", "Voltages", "电压"),
    ("snapshot.choose", "Choose a snapshot", "选择快照"),
    ("snapshot.compare", "Compare", "对比"),
    ("snapshot.directory", "Snapshot directory", "快照目录"),
    ("snapshot.save", "Save snapshot", "保存快照"),
    ("snapshot.tags", "Tags", "标签"),
    ("storage.bad_blocks", "Bad blocks", "坏块"),
    ("storage.capacity", "Capacity", "容量"),
    ("storage.failing", "Failing", "故障"),
    ("storage.health", "Health", "健康状态"),
    ("storage.healthy", "Healthy", "健康"),
    ("storage.model", "Model", "型号"),
    ("storage.power_on", "Power-on hours", "通电时间"),
    ("storage.smart", "SMART", "SMART"),
    ("storage.type", "Type", "类型"),
    ("storage.used", "Used", "已用"),
    ("storage.written", "Total written", "总写入量"),
    ("language.system", "Follow system", "跟随系统"),
    ("language.english", "English", "English"),
    ("language.chinese", "简体中文", "简体中文"),
    ("theme.system", "Follow system", "跟随系统"),
    ("theme.dark", "Dark", "深色"),
    ("theme.light", "Light", "浅色"),
    ("settings.theme", "Theme", "主题"),
    (
        "settings.language_hint",
        "Follow system applies the OS language on next launch",
        "跟随系统会在下次启动时应用系统语言",
    ),
    ("settings.updates", "Updates", "更新"),
    ("app.saved", "Saved", "已保存"),
    ("nav.about", "About", "关于"),
    ("nav.updates", "Updates", "检查更新"),
    (
        "page.about.sub",
        "Application, build environment and legal information",
        "应用、构建环境与法律信息",
    ),
    (
        "page.updates.sub",
        "Compare the installed version with GitHub releases",
        "将当前版本与 GitHub 发布版本进行对比",
    ),
    (
        "update.auto",
        "Check for updates automatically on startup",
        "启动时自动检查更新",
    ),
    (
        "update.include_prerelease",
        "Include pre-releases",
        "包含预发布版本",
    ),
    (
        "update.hint",
        "Uses the public GitHub API; no account or telemetry is involved.",
        "使用公开的 GitHub API，不需要账号，也不会上传遥测数据。",
    ),
    ("update.open_updates", "Open update center", "打开更新中心"),
    ("update.current", "Current version", "当前版本"),
    ("update.check_now", "Check now", "立即检查"),
    (
        "update.idle",
        "No update check has run yet.",
        "尚未执行更新检查。",
    ),
    (
        "update.checking",
        "Checking GitHub releases…",
        "正在检查 GitHub 发布版本…",
    ),
    ("update.failed", "Update check failed", "更新检查失败"),
    (
        "update.failure_hint",
        "This is non-blocking. Check your network or try again later.",
        "此失败不会影响使用，请检查网络或稍后重试。",
    ),
    ("update.available", "Update available", "有可用更新"),
    ("update.up_to_date", "Up to date", "已是最新版本"),
    ("update.prerelease", "Pre-release", "预发布"),
    ("update.checked", "Checked", "检查时间"),
    ("update.latest", "Latest version", "最新版本"),
    ("update.tag", "Release tag", "发布标签"),
    ("update.name", "Release name", "发布名称"),
    ("update.published", "Published", "发布时间"),
    ("update.unknown", "Unknown", "未知"),
    ("update.open_release", "Open release page", "打开发布页面"),
    ("about.application", "Application", "应用信息"),
    ("about.author", "Author", "作者"),
    ("about.version", "Version", "版本"),
    ("about.license", "License", "许可协议"),
    ("about.github", "GitHub profile", "GitHub 主页"),
    ("about.repository", "Source repository", "源码仓库"),
    ("about.rust", "Rust build environment", "Rust 构建环境"),
    ("about.rustc", "Rust compiler", "Rust 编译器"),
    ("about.target", "Build target", "构建目标"),
    ("about.profile", "Build profile", "构建配置"),
    ("about.commit", "Git commit", "Git 提交"),
    ("about.built", "Build time", "构建时间"),
    ("about.runtime", "Runtime environment", "运行环境"),
    ("about.os", "Operating system", "操作系统"),
    ("about.arch", "Architecture", "架构"),
    ("about.family", "Platform family", "平台系列"),
    ("about.pointer_width", "Pointer width", "指针宽度"),
    ("about.legal", "Legal", "法律信息"),
    (
        "about.legal_hint",
        "Source-visible proprietary software. Commercial use requires written permission.",
        "源码可见的专有软件，商业使用必须获得书面授权。",
    ),
    ("about.legal_statement", "Legal statement", "法律声明"),
    ("about.privacy", "Privacy statement", "隐私声明"),
    ("about.disclaimer", "Disclaimer", "免责声明"),
    ("about.contact", "Contact", "联系方式"),
];
