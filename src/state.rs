//! Application state, persistent configuration and snapshot management.
//!
//! The state layer does not perform hardware queries. It only stores the
//! [`HardwareSnapshot`] produced by `device_query`, remembers UI preferences,
//! and owns the portable-mode file layout.

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};
#[cfg(not(test))]
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::device_query::{GpuRuntimeStats, HardwareSnapshot, UnitSystem};
use crate::i18n::LanguagePreference;

pub const CONFIG_FILE: &str = "device-viewer.json";
pub const LOG_FILE: &str = "device-viewer.log";
pub const SCHEMA_VERSION: u32 = 1;
pub const GPU_HISTORY_LEN: usize = 120;

/// Disambiguates snapshots created in the same millisecond.
static SNAPSHOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Page {
    #[default]
    Overview,
    Cpu,
    Pcie,
    Vulkan,
    OpenGl,
    OpenCl,
    Npu,
    Usb,
    Storage,
    Display,
    DirectX,
    Metal,
    OpenVx,
    Sensors,
    Network,
    Audio,
    Checklist,
    Dev,
    Snapshots,
    Compare,
    About,
    Updates,
    Settings,
}

impl Page {
    #[allow(dead_code)] // useful for exhaustive navigation tests
    pub const ALL: [Self; 23] = [
        Self::Overview,
        Self::Cpu,
        Self::Pcie,
        Self::Vulkan,
        Self::OpenGl,
        Self::OpenCl,
        Self::Npu,
        Self::Usb,
        Self::Storage,
        Self::Display,
        Self::DirectX,
        Self::Metal,
        Self::OpenVx,
        Self::Sensors,
        Self::Network,
        Self::Audio,
        Self::Checklist,
        Self::Dev,
        Self::Snapshots,
        Self::Compare,
        Self::About,
        Self::Updates,
        Self::Settings,
    ];

    pub const fn title_key(self) -> &'static str {
        match self {
            Self::Overview => "nav.overview",
            Self::Cpu => "nav.cpu",
            Self::Pcie => "nav.pcie",
            Self::Vulkan => "nav.vulkan",
            Self::OpenGl => "nav.opengl",
            Self::OpenCl => "nav.opencl",
            Self::Npu => "nav.npu",
            Self::Usb => "nav.usb",
            Self::Storage => "nav.storage",
            Self::Display => "nav.display",
            Self::DirectX => "nav.directx",
            Self::Metal => "nav.metal",
            Self::OpenVx => "nav.openvx",
            Self::Sensors => "nav.sensors",
            Self::Network => "nav.network",
            Self::Audio => "nav.audio",
            Self::Checklist => "nav.checklist",
            Self::Dev => "nav.dev",
            Self::Snapshots => "nav.snapshots",
            Self::Compare => "nav.compare",
            Self::About => "nav.about",
            Self::Updates => "nav.updates",
            Self::Settings => "nav.settings",
        }
    }

    #[allow(dead_code)] // panel headers currently pass their keys explicitly
    pub const fn subtitle_key(self) -> &'static str {
        match self {
            Self::Overview => "page.overview.sub",
            Self::Cpu => "page.cpu.sub",
            Self::Pcie => "page.pcie.sub",
            Self::Vulkan => "page.vulkan.sub",
            Self::OpenGl => "page.opengl.sub",
            Self::OpenCl => "page.opencl.sub",
            Self::Npu => "page.npu.sub",
            Self::Usb => "page.usb.sub",
            Self::Storage => "page.storage.sub",
            Self::Display => "page.display.sub",
            Self::DirectX => "page.directx.sub",
            Self::Metal => "page.metal.sub",
            Self::OpenVx => "page.openvx.sub",
            Self::Sensors => "page.sensors.sub",
            Self::Network => "page.network.sub",
            Self::Audio => "page.audio.sub",
            Self::Checklist => "page.checklist.sub",
            Self::Dev => "page.dev.sub",
            Self::Snapshots => "page.snapshots.sub",
            Self::Compare => "page.compare.sub",
            Self::About => "page.about.sub",
            Self::Updates => "page.updates.sub",
            Self::Settings => "page.settings.sub",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::Overview => "◈",
            Self::Cpu => "▣",
            Self::Pcie => "▤",
            Self::Vulkan => "◆",
            Self::OpenGl => "◇",
            Self::OpenCl => "◉",
            Self::Npu => "✦",
            Self::Usb => "⌘",
            Self::Storage => "▥",
            Self::Display => "▭",
            Self::DirectX => "▷",
            Self::Metal => "◐",
            Self::OpenVx => "◍",
            Self::Sensors => "◔",
            Self::Network => "⇄",
            Self::Audio => "♪",
            Self::Checklist => "☑",
            Self::Dev => "⌨",
            Self::Snapshots => "❐",
            Self::Compare => "⇋",
            Self::About => "ⓘ",
            Self::Updates => "⇧",
            Self::Settings => "⚙",
        }
    }

    /// Module that backs a hardware page, used by the checklist and by the
    /// page-scoped refresh action. Pages without a module return `None`.
    #[allow(dead_code)] // reserved for page-scoped refresh wiring
    pub const fn module(self) -> Option<&'static str> {
        match self {
            Self::Cpu => Some("cpu"),
            Self::Pcie => Some("pcie"),
            Self::Vulkan => Some("vulkan"),
            Self::OpenGl => Some("opengl"),
            Self::OpenCl => Some("opencl"),
            Self::Npu => Some("npu"),
            Self::Usb => Some("usb"),
            Self::Storage => Some("storage"),
            Self::Display => Some("display"),
            Self::DirectX => Some("directx"),
            Self::Metal => Some("metal"),
            Self::OpenVx => Some("openvx"),
            Self::Sensors => Some("sensors"),
            Self::Network | Self::Audio => Some("system"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterSettings {
    pub pcie_text: String,
    pub usb_text: String,
    pub extension_text: String,
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            pcie_text: String::new(),
            usb_text: String::new(),
            extension_text: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];

    pub const fn label_key(self) -> &'static str {
        match self {
            Self::System => "theme.system",
            Self::Dark => "theme.dark",
            Self::Light => "theme.light",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::System => "◐",
            Self::Dark => "☾",
            Self::Light => "☀",
        }
    }

    pub const fn cycle(self) -> Self {
        match self {
            Self::System => Self::Dark,
            Self::Dark => Self::Light,
            Self::Light => Self::System,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Persisted language choice; `System` is the first-launch default.
    pub language: LanguagePreference,
    /// Persisted theme choice; `System` is the first-launch default.
    pub theme: ThemeMode,
    /// Legacy boolean from versions before the System theme mode existed.
    /// Kept only so old configuration files migrate without losing the choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dark: Option<bool>,
    pub units: UnitSystem,
    pub portable: bool,
    pub page: Page,
    pub window_rect: Option<[f32; 4]>,
    pub filters: FilterSettings,
    pub favorite_extensions: Vec<String>,
    pub auto_check_updates: bool,
    pub include_prerelease: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: LanguagePreference::System,
            theme: ThemeMode::System,
            dark: None,
            units: UnitSystem::Binary,
            portable: false,
            page: Page::Overview,
            window_rect: None,
            filters: FilterSettings::default(),
            favorite_extensions: Vec::new(),
            auto_check_updates: true,
            include_prerelease: false,
        }
    }
}

impl Settings {
    /// Migrates the pre-1.1 `dark: bool` field into an explicit theme choice.
    /// A configuration written by the new version has `theme` and no `dark`,
    /// so this is a no-op for fresh files.
    fn normalize(&mut self) {
        if let Some(dark) = self.dark.take()
            && self.theme == ThemeMode::System
        {
            self.theme = if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateReport {
    pub current: String,
    pub latest: String,
    pub tag: String,
    pub name: String,
    pub url: String,
    pub prerelease: bool,
    pub available: bool,
    pub published_at: Option<String>,
    pub checked_at_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    Ready(UpdateReport),
    Failed(String),
}

impl UpdateState {
    pub fn is_checking(&self) -> bool {
        matches!(self, Self::Checking)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedConfig {
    pub schema_version: u32,
    pub settings: Settings,
    pub snapshots: Vec<SnapshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    /// File name inside the snapshot directory, never an absolute path, so the
    /// library can move with the application in portable mode.
    pub file: String,
    pub label: String,
    pub tags: Vec<String>,
    pub created_at_unix_ms: u64,
    pub size_bytes: u64,
}

impl SnapshotEntry {
    pub fn path(&self, directory: &Path) -> PathBuf {
        let name = Path::new(&self.file)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""));
        directory.join(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeLanguage {
    C,
    Rust,
}

impl Default for CodeLanguage {
    fn default() -> Self {
        Self::Rust
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuSample {
    pub utilization: f32,
    pub vram_percent: f32,
}

impl Default for GpuSample {
    fn default() -> Self {
        Self {
            utilization: 0.0,
            vram_percent: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionReport {
    pub elevated: bool,
    pub items: Vec<String>,
}

impl PermissionReport {
    fn detect() -> Self {
        let elevated = is_elevated();
        let mut items = Vec::new();
        if !elevated {
            items.push("storage: SMART / NVMe health".to_string());
            items.push("pcie: low-level PCI configuration".to_string());
            items.push("sensors: motherboard and fan telemetry".to_string());
            items.push("display: full monitor capability data".to_string());
        }
        Self { elevated, items }
    }
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub gpu_history: VecDeque<GpuSample>,
    pub request_refresh: bool,
    pub permission_notice_open: bool,
    pub selected_vulkan_device: usize,
    pub dev_language: CodeLanguage,
    pub compare_filter: String,
    pub snapshot_label: String,
    pub tag_buffer: String,
    pub tag_edit: Option<String>,
    pub toast: Option<String>,
    pub request_update_check: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            gpu_history: VecDeque::with_capacity(GPU_HISTORY_LEN),
            request_refresh: false,
            permission_notice_open: true,
            selected_vulkan_device: 0,
            dev_language: CodeLanguage::Rust,
            compare_filter: String::new(),
            snapshot_label: String::new(),
            tag_buffer: String::new(),
            tag_edit: None,
            toast: None,
            request_update_check: false,
        }
    }
}

pub struct AppState {
    pub settings: Settings,
    pub snapshot: HardwareSnapshot,
    pub snapshots: Vec<SnapshotEntry>,
    pub compare: Option<HardwareSnapshot>,
    pub compare_label: String,
    pub compare_file: Option<String>,
    pub gpu_runtime: Vec<GpuRuntimeStats>,
    pub permission: PermissionReport,
    pub update: UpdateState,
    pub ui: UiState,
    pub config_path: PathBuf,
    pub snapshots_dir: PathBuf,
    dirty: bool,
    last_save: Instant,
}

impl AppState {
    pub fn load(portable_override: bool) -> Self {
        let normal_path = config_file(false);
        let portable_path = config_file(true);
        let normal = read_config(&normal_path).map(|config| (config, modified_at(&normal_path)));
        let portable =
            read_config(&portable_path).map(|config| (config, modified_at(&portable_path)));
        let (mut config, used_portable_file) = match (normal, portable) {
            (Some((normal, normal_time)), Some((portable, portable_time))) => {
                if portable_time > normal_time {
                    (portable, true)
                } else {
                    (normal, false)
                }
            }
            (Some((normal, _)), None) => (normal, false),
            (None, Some((portable, _))) => (portable, true),
            (None, None) => (SavedConfig::default(), false),
        };

        if portable_override {
            config.settings.portable = true;
        } else if used_portable_file {
            config.settings.portable = true;
        }
        config.settings.normalize();

        let portable = config.settings.portable;
        let mut state = Self {
            settings: config.settings,
            snapshot: HardwareSnapshot::default(),
            snapshots: config.snapshots,
            compare: None,
            compare_label: String::new(),
            compare_file: None,
            gpu_runtime: Vec::new(),
            permission: PermissionReport::detect(),
            update: UpdateState::Idle,
            ui: UiState::default(),
            config_path: config_file(portable),
            snapshots_dir: snapshots_dir(portable),
            dirty: portable_override,
            last_save: Instant::now(),
        };
        state.refresh_snapshots();
        state
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn save_if_due(&mut self) {
        if self.dirty && self.last_save.elapsed() >= Duration::from_millis(1200) {
            self.save_config();
        }
    }

    pub fn save_config(&mut self) {
        let config = SavedConfig {
            schema_version: SCHEMA_VERSION,
            settings: self.settings.clone(),
            snapshots: self.snapshots.clone(),
        };
        if let Some(parent) = self.config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match serde_json::to_vec_pretty(&config) {
            Ok(bytes) => match fs::write(&self.config_path, bytes) {
                Ok(()) => {
                    self.dirty = false;
                    self.last_save = Instant::now();
                }
                Err(error) => crate::log::warn(format!(
                    "could not write {}: {error}",
                    self.config_path.display()
                )),
            },
            Err(error) => crate::log::warn(format!("could not serialize configuration: {error}")),
        }
    }

    pub fn set_portable(&mut self, portable: bool) {
        if self.settings.portable == portable {
            return;
        }
        self.settings.portable = portable;
        self.rebase_paths(config_file(portable), snapshots_dir(portable));
        self.dirty = true;
        self.save_config();
        crate::log::info(format!(
            "portable mode {}; config={}",
            if portable { "enabled" } else { "disabled" },
            self.config_path.display()
        ));
    }

    fn rebase_paths(&mut self, config_path: PathBuf, snapshots_dir: PathBuf) {
        self.config_path = config_path;
        self.snapshots_dir = snapshots_dir;
        self.refresh_snapshots();
    }

    pub fn refresh_snapshots(&mut self) {
        let mut known: BTreeMap<String, SnapshotEntry> = self
            .snapshots
            .drain(..)
            .map(|entry| (entry.file.clone(), entry))
            .collect();
        let mut entries = Vec::new();

        if let Ok(directory) = fs::read_dir(&self.snapshots_dir) {
            for item in directory.flatten() {
                let path = item.path();
                if !path.is_file() {
                    continue;
                }
                let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
                    continue;
                };
                if !extension.eq_ignore_ascii_case("dvsnap")
                    && !extension.eq_ignore_ascii_case("json")
                {
                    continue;
                }
                let Some(file) = path.file_name().and_then(|value| value.to_str()) else {
                    continue;
                };
                let metadata = item.metadata().ok();
                let mut entry = known.remove(file).unwrap_or_else(|| SnapshotEntry {
                    file: file.to_string(),
                    label: path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or(file)
                        .to_string(),
                    tags: Vec::new(),
                    created_at_unix_ms: metadata
                        .as_ref()
                        .and_then(|metadata| metadata.modified().ok())
                        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
                        .unwrap_or_default(),
                    size_bytes: 0,
                });
                entry.size_bytes = metadata.map(|metadata| metadata.len()).unwrap_or(0);
                entries.push(entry);
            }
        }

        entries.sort_by(|left, right| right.created_at_unix_ms.cmp(&left.created_at_unix_ms));
        self.snapshots = entries;
    }

    pub fn save_current_snapshot(
        &mut self,
        label: impl Into<String>,
        tags: Vec<String>,
    ) -> Result<SnapshotEntry, String> {
        if let Some(parent) = self.snapshots_dir.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::create_dir_all(&self.snapshots_dir)
            .map_err(|error| format!("cannot create snapshot directory: {error}"))?;

        let unique = unique_stamp();
        let file = format!("snapshot-{unique}.dvsnap");
        let path = self.snapshots_dir.join(&file);
        let mut snapshot = self.snapshot.clone();
        snapshot.sanitize();
        let bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|error| format!("cannot serialize snapshot: {error}"))?;
        fs::write(&path, &bytes).map_err(|error| format!("cannot write snapshot: {error}"))?;

        let label = label.into();
        let label = if label.trim().is_empty() {
            format!("Snapshot {}", format_timestamp(now_ms()))
        } else {
            label.trim().to_string()
        };
        let entry = SnapshotEntry {
            file,
            label,
            tags,
            created_at_unix_ms: now_ms(),
            size_bytes: bytes.len() as u64,
        };
        self.snapshots.insert(0, entry.clone());
        self.dirty = true;
        self.save_config();
        crate::log::info(format!("saved snapshot {}", path.display()));
        Ok(entry)
    }

    pub fn import_snapshot_file(&mut self, source: &Path) -> Result<SnapshotEntry, String> {
        let bytes = fs::read(source)
            .map_err(|error| format!("cannot read {}: {error}", source.display()))?;
        parse_snapshot(&bytes).map_err(|error| format!("invalid snapshot: {error}"))?;
        fs::create_dir_all(&self.snapshots_dir)
            .map_err(|error| format!("cannot create snapshot directory: {error}"))?;
        let file = format!("import-{}.dvsnap", unique_stamp());
        let destination = self.snapshots_dir.join(&file);
        fs::write(&destination, &bytes)
            .map_err(|error| format!("cannot import snapshot: {error}"))?;

        let label = source
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Imported snapshot")
            .to_string();
        let entry = SnapshotEntry {
            file,
            label,
            tags: Vec::new(),
            created_at_unix_ms: now_ms(),
            size_bytes: bytes.len() as u64,
        };
        self.snapshots.insert(0, entry.clone());
        self.dirty = true;
        self.save_config();
        crate::log::info(format!("imported snapshot {}", source.display()));
        Ok(entry)
    }

    pub fn load_compare(&mut self, file: &str) -> Result<(), String> {
        let path = self
            .snapshot_path(file)
            .ok_or_else(|| "invalid snapshot file name".to_string())?;
        let bytes =
            fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let snapshot =
            parse_snapshot(&bytes).map_err(|error| format!("invalid snapshot: {error}"))?;
        self.compare = Some(snapshot);
        self.compare_label = self
            .snapshots
            .iter()
            .find(|entry| entry.file == file)
            .map(|entry| entry.label.clone())
            .unwrap_or_else(|| file.to_string());
        self.compare_file = Some(file.to_string());
        crate::log::info(format!("baseline snapshot loaded: {}", path.display()));
        Ok(())
    }

    fn snapshot_path(&self, file: &str) -> Option<PathBuf> {
        snapshot_path_in(&self.snapshots_dir, file)
    }

    pub fn delete_snapshot(&mut self, file: &str) -> Result<(), String> {
        let path = self
            .snapshot_path(file)
            .ok_or_else(|| "invalid snapshot file name".to_string())?;
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|error| format!("cannot delete {}: {error}", path.display()))?;
        }
        self.snapshots.retain(|entry| entry.file != file);
        if self.compare_file.as_deref() == Some(file) {
            self.compare = None;
            self.compare_file = None;
            self.compare_label.clear();
        }
        self.dirty = true;
        self.save_config();
        Ok(())
    }

    pub fn set_snapshot_tags(&mut self, file: &str, tags: Vec<String>) {
        if let Some(entry) = self.snapshots.iter_mut().find(|entry| entry.file == file) {
            entry.tags = tags;
            self.dirty = true;
            self.save_config();
        }
    }

    pub fn record_gpu_sample(&mut self) {
        let from_runtime = self.gpu_runtime.first().map(|stats| {
            (
                stats.utilization_percent.unwrap_or(0.0),
                stats.memory_used_bytes,
                stats.memory_total_bytes,
            )
        });
        let from_vulkan = self
            .snapshot
            .vulkan
            .as_ref()
            .and_then(|info| info.devices.first())
            .map(|device| {
                (
                    0.0,
                    device.used_memory_bytes,
                    Some(device.total_memory_bytes),
                )
            });
        let Some((utilization, used, total)) = from_runtime.or(from_vulkan) else {
            return;
        };
        let vram_percent = match (used, total) {
            (Some(used), Some(total)) if total > 0 => (used as f64 / total as f64 * 100.0) as f32,
            _ => 0.0,
        };
        self.ui.gpu_history.push_back(GpuSample {
            utilization: if utilization.is_finite() {
                utilization.clamp(0.0, 100.0)
            } else {
                0.0
            },
            vram_percent: if vram_percent.is_finite() {
                vram_percent.clamp(0.0, 100.0)
            } else {
                0.0
            },
        });
        while self.ui.gpu_history.len() > GPU_HISTORY_LEN {
            self.ui.gpu_history.pop_front();
        }
    }

    pub fn total_warnings(&self) -> usize {
        let module_warnings = [
            self.snapshot.cpu.as_ref().map(|value| value.warnings.len()),
            self.snapshot
                .memory
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .pcie
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .vulkan
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .opengl
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .opencl
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot.npu.as_ref().map(|value| value.warnings.len()),
            self.snapshot.usb.as_ref().map(|value| value.warnings.len()),
            self.snapshot
                .storage
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .motherboard
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .display
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .directx
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .metal
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .openvx
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .system
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .sensors
                .as_ref()
                .map(|value| value.warnings.len()),
            self.snapshot
                .bandwidth
                .as_ref()
                .map(|value| value.warnings.len()),
        ];
        module_warnings.into_iter().flatten().sum::<usize>() + self.snapshot.warning_count()
    }
}

pub fn default_config_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DeviceViewer")
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("DeviceViewer")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("device-viewer")
    }
}

pub fn executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_file(portable: bool) -> PathBuf {
    let directory = if portable {
        executable_dir()
    } else {
        default_config_dir()
    };
    directory.join(CONFIG_FILE)
}

pub fn snapshots_dir(portable: bool) -> PathBuf {
    let directory = if portable {
        executable_dir()
    } else {
        default_config_dir()
    };
    directory.join("snapshots")
}

pub fn log_file(portable: bool) -> PathBuf {
    config_file(portable)
        .parent()
        .unwrap_or(Path::new("."))
        .join(LOG_FILE)
}

fn read_config(path: &Path) -> Option<SavedConfig> {
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn modified_at(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default()
}

/// Joins a snapshot file name into `directory` while rejecting any path
/// traversal. Only one plain file-name component is accepted.
fn snapshot_path_in(directory: &Path, file: &str) -> Option<PathBuf> {
    let mut components = Path::new(file).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Some(directory.join(file)),
        _ => None,
    }
}

fn unique_stamp() -> String {
    let millis = now_ms();
    let sequence = SNAPSHOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{millis}-{sequence}")
}

pub fn format_timestamp(ms: u64) -> String {
    let seconds = ms / 1000;
    let days = (seconds / 86_400) as i64;
    let time = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        time / 3_600,
        (time % 3_600) / 60,
        time % 60
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month as u32, day as u32)
}

fn parse_snapshot(bytes: &[u8]) -> Result<HardwareSnapshot, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    if value.get("snapshot").is_some() {
        serde_json::from_value(value["snapshot"].clone()).map_err(|error| error.to_string())
    } else {
        serde_json::from_value(value).map_err(|error| error.to_string())
    }
}

fn is_elevated() -> bool {
    #[cfg(test)]
    {
        // Keep the probe out of unit tests: spawning a shell command is slow
        // and its result is irrelevant for the path/snapshot tests below.
        return false;
    }
    #[cfg(not(test))]
    {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            return Command::new("net")
                .arg("session")
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
        }

        #[cfg(not(target_os = "windows"))]
        {
            Command::new("id")
                .arg("-u")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|uid| uid.trim() == "0")
                .unwrap_or(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_stamps_are_unique_within_one_millisecond() {
        let stamps: std::collections::HashSet<String> =
            (0..1_000).map(|_| unique_stamp()).collect();
        assert_eq!(stamps.len(), 1_000);
        assert!(stamps.iter().all(|stamp| stamp.contains('-')));
    }

    #[test]
    fn snapshot_file_names_cannot_escape_the_directory() {
        let directory = Path::new("snapshots");
        assert_eq!(
            snapshot_path_in(directory, "capture.dvsnap"),
            Some(directory.join("capture.dvsnap"))
        );
        assert!(snapshot_path_in(directory, "../capture.dvsnap").is_none());
        assert!(snapshot_path_in(directory, "nested/capture.dvsnap").is_none());
        assert!(snapshot_path_in(directory, r"..\capture.dvsnap").is_none());
        assert!(snapshot_path_in(directory, "").is_none());
    }

    #[test]
    fn portable_mode_uses_a_different_config_and_snapshot_root() {
        let normal_config = config_file(false);
        let portable_config = config_file(true);
        let normal_snapshots = snapshots_dir(false);
        let portable_snapshots = snapshots_dir(true);

        assert_ne!(normal_config, portable_config);
        assert_ne!(normal_snapshots, portable_snapshots);
        assert_eq!(
            portable_config.file_name().and_then(|name| name.to_str()),
            Some(CONFIG_FILE)
        );
        assert_eq!(
            portable_snapshots
                .file_name()
                .and_then(|name| name.to_str()),
            Some("snapshots")
        );
    }

    #[test]
    fn wrapped_and_raw_snapshots_parse_identically() {
        let snapshot = HardwareSnapshot::default();
        let raw = serde_json::to_vec(&snapshot).expect("snapshot serializes");
        let wrapped = serde_json::to_vec(&serde_json::json!({ "snapshot": snapshot }))
            .expect("wrapper serializes");
        assert_eq!(
            parse_snapshot(&raw).expect("raw snapshot").schema_version,
            1
        );
        assert_eq!(
            parse_snapshot(&wrapped)
                .expect("wrapped snapshot")
                .schema_version,
            1
        );
    }

    #[test]
    fn timestamps_are_rendered_as_civil_dates() {
        assert_eq!(format_timestamp(0), "1970-01-01 00:00:00");
        assert_eq!(format_timestamp(3_661_000), "1970-01-01 01:01:01");
    }

    #[test]
    fn first_launch_defaults_follow_system_and_enable_update_checks() {
        let settings = Settings::default();
        assert_eq!(settings.language, LanguagePreference::System);
        assert_eq!(settings.theme, ThemeMode::System);
        assert!(settings.dark.is_none());
        assert!(settings.auto_check_updates);
        assert!(!settings.include_prerelease);
    }

    #[test]
    fn legacy_dark_choice_migrates_into_theme_mode() {
        let mut legacy = Settings {
            dark: Some(true),
            ..Settings::default()
        };
        legacy.normalize();
        assert_eq!(legacy.theme, ThemeMode::Dark);
        assert!(legacy.dark.is_none());

        let mut explicit = Settings {
            dark: Some(false),
            theme: ThemeMode::Light,
            ..Settings::default()
        };
        explicit.normalize();
        assert_eq!(explicit.theme, ThemeMode::Light);
        assert!(explicit.dark.is_none());
    }

    #[test]
    fn settings_round_trip_keeps_preferences() {
        let settings = Settings {
            language: LanguagePreference::Chinese,
            theme: ThemeMode::Dark,
            auto_check_updates: false,
            include_prerelease: true,
            ..Settings::default()
        };
        let encoded = serde_json::to_string(&settings).expect("settings serialize");
        let value: serde_json::Value = serde_json::from_str(&encoded).expect("settings JSON");
        assert!(value.get("dark").is_none());
        let decoded: Settings = serde_json::from_str(&encoded).expect("settings deserialize");
        assert_eq!(decoded.language, LanguagePreference::Chinese);
        assert_eq!(decoded.theme, ThemeMode::Dark);
        assert!(!decoded.auto_check_updates);
        assert!(decoded.include_prerelease);
    }
}
