//! Application state, persistent configuration and snapshot management.
//!
//! The state layer does not perform hardware queries. It only stores the
//! [`HardwareSnapshot`] produced by `device_query`, remembers UI preferences,
//! and owns the portable-mode file layout.

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterSettings {
    pub pcie_text: String,
    pub usb_text: String,
    pub extension_text: String,
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
    /// Unix 毫秒时间戳：最近一次向 GitHub 发起更新检查的时刻。
    ///
    /// 未认证的 GitHub API 每小时只允许 60 次请求，所以启动时的自动检查必须
    /// 跨进程记住上次请求时间，否则频繁重启会把额度耗尽并收到 403。
    #[serde(default)]
    pub last_update_check_ms: u64,
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
            last_update_check_ms: 0,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CodeLanguage {
    C,
    #[default]
    Rust,
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

/// Severity of a transient notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

/// One transient notification; `until` is the instant it disappears.
#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub until: Instant,
    /// Page the toast refers to; clicking the toast jumps there.
    pub page: Option<Page>,
}

/// How long a toast stays on screen.
const TOAST_LIFETIME: Duration = Duration::from_secs(4);
/// At most this many toasts are kept; the oldest is dropped first.
const TOAST_LIMIT: usize = 3;

pub struct UiState {
    pub gpu_history: VecDeque<GpuSample>,
    pub request_refresh: bool,
    pub permission_notice_open: bool,
    pub selected_vulkan_device: usize,
    pub dev_language: CodeLanguage,
    pub compare_filter: String,
    /// Kind filters of the compare table; all false means no kind filter.
    pub compare_show_added: bool,
    pub compare_show_removed: bool,
    pub compare_show_changed: bool,
    pub snapshot_label: String,
    pub tag_buffer: String,
    /// Snapshot whose tags are being edited; the editor buffer lives beside it
    /// so typing survives the per-frame rebuild of the list.
    pub tag_edit: Option<String>,
    pub tag_edit_buffer: String,
    /// Snapshot awaiting delete confirmation.
    pub pending_delete: Option<String>,
    /// Snapshot whose row menu (compare/tags/delete) is open.
    pub snapshot_menu: Option<String>,
    /// 行菜单弹出的屏幕坐标，菜单跨帧保持打开时需要复用。
    pub snapshot_menu_pos: Option<egui::Pos2>,
    /// Transient notifications, oldest first.
    pub toasts: Vec<Toast>,
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
            compare_show_added: false,
            compare_show_removed: false,
            compare_show_changed: false,
            snapshot_label: String::new(),
            tag_buffer: String::new(),
            tag_edit: None,
            tag_edit_buffer: String::new(),
            pending_delete: None,
            snapshot_menu: None,
            snapshot_menu_pos: None,
            toasts: Vec::new(),
            request_update_check: false,
        }
    }
}

impl UiState {
    /// Pushes an informational toast, keeping at most `TOAST_LIMIT` entries.
    pub fn notify(&mut self, text: impl Into<String>) {
        self.notify_kind(text, ToastKind::Info);
    }

    /// Pushes a toast with an explicit severity.
    pub fn notify_kind(&mut self, text: impl Into<String>, kind: ToastKind) {
        self.push_toast(text.into(), kind, None);
    }

    /// Pushes a success toast, used by actions that completed as asked.
    pub fn notify_success(&mut self, text: impl Into<String>) {
        self.notify_kind(text, ToastKind::Success);
    }

    /// Pushes a warning toast: the action finished but something is off.
    pub fn notify_warning(&mut self, text: impl Into<String>) {
        self.notify_kind(text, ToastKind::Warning);
    }

    /// Pushes an error toast: the action did not complete.
    pub fn notify_error(&mut self, text: impl Into<String>) {
        self.notify_kind(text, ToastKind::Error);
    }

    /// Pushes a toast that jumps to `page` when clicked.
    pub fn notify_page(&mut self, text: impl Into<String>, page: Page) {
        self.push_toast(text.into(), ToastKind::Info, Some(page));
    }

    fn push_toast(&mut self, text: String, kind: ToastKind, page: Option<Page>) {
        self.toasts.push(Toast {
            text,
            kind,
            until: Instant::now() + TOAST_LIFETIME,
            page,
        });
        if self.toasts.len() > TOAST_LIMIT {
            let excess = self.toasts.len() - TOAST_LIMIT;
            self.toasts.drain(..excess);
        }
    }

    /// Drops expired toasts; called once per frame before rendering.
    pub fn prune_toasts(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|toast| toast.until > now);
    }

    /// Starts editing the tags of `file`.
    ///
    /// The buffer is only rebuilt when a different snapshot is opened, so the
    /// per-frame rebuild of the list can never wipe what the user is typing.
    pub fn begin_tag_edit(&mut self, file: &str, tags: &[String]) {
        if self.tag_edit.as_deref() != Some(file) {
            self.tag_edit = Some(file.to_string());
            self.tag_edit_buffer = tags.join(", ");
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
    pub exports_dir: PathBuf,
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

        if portable_override || used_portable_file {
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
            exports_dir: exports_dir(portable),
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
            Ok(bytes) => {
                // 同目录临时文件 + rename：崩溃或断电不会留下被截断的配置。
                let tmp = self.config_path.with_extension("json.tmp");
                let written =
                    fs::write(&tmp, &bytes).and_then(|()| fs::rename(&tmp, &self.config_path));
                match written {
                    Ok(()) => {
                        self.dirty = false;
                        self.last_save = Instant::now();
                        // Labels and tags travel with the snapshot file, so a
                        // portable-mode switch cannot lose them.
                        for entry in &self.snapshots {
                            write_snapshot_metadata(&self.snapshots_dir, entry);
                        }
                    }
                    Err(error) => {
                        // 清理失败的临时文件，旧配置仍然完整可用。
                        let _ = fs::remove_file(&tmp);
                        crate::log::warn(format!(
                            "could not write {}: {error}",
                            self.config_path.display()
                        ));
                    }
                }
            }
            Err(error) => crate::log::warn(format!("could not serialize configuration: {error}")),
        }
    }

    pub fn set_portable(&mut self, portable: bool) {
        if self.settings.portable == portable {
            return;
        }
        self.settings.portable = portable;
        self.rebase_paths(portable);
        self.dirty = true;
        self.save_config();
        crate::log::info(format!(
            "portable mode {}; config={}",
            if portable { "enabled" } else { "disabled" },
            self.config_path.display()
        ));
    }

    fn rebase_paths(&mut self, portable: bool) {
        self.config_path = config_file(portable);
        self.snapshots_dir = snapshots_dir(portable);
        self.exports_dir = exports_dir(portable);
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
                // 只有快照文件算快照；导出的 JSON 报告不再混进列表。
                if !extension.eq_ignore_ascii_case("dvsnap") {
                    continue;
                }
                let Some(file) = path.file_name().and_then(|value| value.to_str()) else {
                    continue;
                };
                let metadata = item.metadata().ok();
                let persisted = read_snapshot_metadata(&self.snapshots_dir, file);
                let mut entry = known.remove(file).unwrap_or_else(|| SnapshotEntry {
                    file: file.to_string(),
                    label: persisted
                        .as_ref()
                        .map(|metadata| metadata.label.trim().to_string())
                        .filter(|label| !label.is_empty())
                        .unwrap_or_else(|| {
                            path.file_stem()
                                .and_then(|value| value.to_str())
                                .unwrap_or(file)
                                .to_string()
                        }),
                    tags: persisted.map(|metadata| metadata.tags).unwrap_or_default(),
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

        entries.sort_by_key(|entry| std::cmp::Reverse(entry.created_at_unix_ms));
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
        let _ = fs::remove_file(metadata_path(&self.snapshots_dir, file));
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

/// Exported reports live next to, but outside, the snapshot directory so a
/// JSON export can never be mistaken for a snapshot.
pub fn exports_dir(portable: bool) -> PathBuf {
    snapshots_dir(portable)
        .parent()
        .unwrap_or(Path::new("."))
        .join("exports")
}

/// Reads the persisted window geometry before the viewport is created.
///
/// AppState::load loads the same configuration again; this helper only exposes
/// the geometry that eframe::run_native needs up front, preferring the mode the
/// user last used.
pub fn saved_window_rect(portable: bool) -> Option<[f32; 4]> {
    [config_file(portable), config_file(!portable)]
        .into_iter()
        .filter_map(|path| read_config(&path))
        .find_map(|config| sanitize_window_rect(config.settings.window_rect))
}

fn sanitize_window_rect(rect: Option<[f32; 4]>) -> Option<[f32; 4]> {
    let [x, y, width, height] = rect?;
    if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
        return None;
    }
    if width <= 0.0 || height <= 0.0 || x.abs() > 100_000.0 || y.abs() > 100_000.0 {
        return None;
    }
    Some([
        x,
        y,
        width.clamp(980.0, 16_384.0),
        height.clamp(640.0, 16_384.0),
    ])
}

pub fn log_file(portable: bool) -> PathBuf {
    config_file(portable)
        .parent()
        .unwrap_or(Path::new("."))
        .join(LOG_FILE)
}

fn read_config(path: &Path) -> Option<SavedConfig> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                crate::log::warn(format!("could not read {}: {error}", path.display()));
            }
            return None;
        }
    };
    match serde_json::from_slice(&bytes) {
        Ok(config) => Some(config),
        Err(error) => {
            // 不静默丢弃用户配置：损坏的文件改名保留，用户仍可手工抢救。
            let backup = path.with_extension("json.bak");
            let _ = fs::remove_file(&backup);
            if let Err(backup_error) = fs::rename(path, &backup) {
                crate::log::warn(format!(
                    "could not preserve unreadable {}: {backup_error}",
                    path.display()
                ));
            }
            crate::log::warn(format!(
                "configuration {} is unreadable ({error}); kept as {} and defaults were loaded",
                path.display(),
                backup.display()
            ));
            None
        }
    }
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

type CivilTime = (i64, u32, u32, u32, u32, u32);

/// Formats a Unix timestamp in milliseconds as the user's local civil time.
///
/// When the platform cannot resolve a time zone the value falls back to UTC and
/// says so explicitly, so a UTC value is never rendered as if it were local.
pub fn format_timestamp(ms: u64) -> String {
    let seconds = ms / 1000;
    let days = (seconds / 86_400) as i64;
    let time = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let utc: CivilTime = (
        year,
        month,
        day,
        (time / 3_600) as u32,
        ((time % 3_600) / 60) as u32,
        (time % 60) as u32,
    );
    match local_civil_time(utc) {
        Some((year, month, day, hour, minute, second)) => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
        }
        None => format!(
            "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
            utc.3, utc.4, utc.5
        ),
    }
}

#[cfg(windows)]
fn local_civil_time(utc: CivilTime) -> Option<CivilTime> {
    #[repr(C)]
    #[allow(dead_code)]
    struct SystemTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }

    // SAFETY: the API only reads `universal_time` and writes `local_time`;
    // both pointers refer to live SYSTEMTIME values for the whole call.
    unsafe extern "system" {
        fn SystemTimeToTzSpecificLocalTime(
            time_zone_information: *const core::ffi::c_void,
            universal_time: *const SystemTime,
            local_time: *mut SystemTime,
        ) -> i32;
    }

    let (year, month, day, hour, minute, second) = utc;
    let universal = SystemTime {
        year: u16::try_from(year).ok()?,
        month: u16::try_from(month).ok()?,
        day_of_week: 0,
        day: u16::try_from(day).ok()?,
        hour: u16::try_from(hour).ok()?,
        minute: u16::try_from(minute).ok()?,
        second: u16::try_from(second).ok()?,
        milliseconds: 0,
    };
    let mut local = SystemTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    // SAFETY: a null time zone asks for the current zone; both buffers are valid.
    let converted =
        unsafe { SystemTimeToTzSpecificLocalTime(std::ptr::null(), &universal, &mut local) };
    if converted == 0 {
        return None;
    }
    Some((
        local.year as i64,
        local.month as u32,
        local.day as u32,
        local.hour as u32,
        local.minute as u32,
        local.second as u32,
    ))
}

#[cfg(unix)]
fn local_civil_time(utc: CivilTime) -> Option<CivilTime> {
    use std::os::raw::{c_char, c_int, c_long};

    #[repr(C)]
    #[allow(dead_code)]
    struct Tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        tm_gmtoff: c_long,
        tm_zone: *const c_char,
    }

    // SAFETY: `localtime_r` writes into the supplied buffer, which outlives the
    // call, and returns null on failure.
    unsafe extern "C" {
        fn localtime_r(time: *const c_long, result: *mut Tm) -> *mut Tm;
    }

    let (year, month, day, hour, minute, second) = utc;
    let seconds = days_from_civil(year, month, day)
        .checked_mul(86_400)?
        .checked_add(hour as i64 * 3_600 + minute as i64 * 60 + second as i64)?;
    let timestamp = c_long::try_from(seconds).ok()?;
    let mut local = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null(),
    };
    // SAFETY: the timestamp and the output buffer are valid for the call.
    let result = unsafe { localtime_r(&timestamp, &mut local) };
    if result.is_null() {
        return None;
    }
    Some((
        local.tm_year as i64 + 1_900,
        (local.tm_mon + 1) as u32,
        local.tm_mday as u32,
        local.tm_hour as u32,
        local.tm_min as u32,
        local.tm_sec as u32,
    ))
}

#[cfg(not(any(unix, windows)))]
fn local_civil_time(_utc: CivilTime) -> Option<CivilTime> {
    None
}

#[cfg_attr(not(unix), allow(dead_code))]
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let month = month as i64;
    let day = day as i64;
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
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
    let snapshot: HardwareSnapshot = if value.get("snapshot").is_some() {
        serde_json::from_value(value["snapshot"].clone()).map_err(|error| error.to_string())?
    } else {
        serde_json::from_value(value).map_err(|error| error.to_string())?
    };

    // Schema versions are checked in both directions. A newer file would be
    // accepted field by field by `serde(default)` and silently open with half
    // of its content missing, so refuse it; an older file is readable, but the
    // caller is told so that the missing sections are not mistaken for absent
    // hardware.
    if snapshot.schema_version > SCHEMA_VERSION {
        return Err(format!(
            "snapshot uses schema version {}, but this build understands up to {}; update Device Viewer to open it",
            snapshot.schema_version, SCHEMA_VERSION
        ));
    }
    if snapshot.schema_version < SCHEMA_VERSION {
        crate::log::warn(format!(
            "snapshot uses schema version {}; reading it as a {} snapshot",
            snapshot.schema_version, SCHEMA_VERSION
        ));
    }
    Ok(snapshot)
}

/// Label and tags used to live only in the configuration file, which is scoped
/// to one directory: enabling portable mode rebuilt the list from the other
/// directory and dropped them without a trace. They are mirrored into a
/// sidecar beside the snapshot so that they follow the file. The sidecar ends
/// in `.json`, so the scanner (which only accepts `.dvsnap`) ignores it.
#[derive(Debug, Default, Serialize, Deserialize)]
struct SnapshotMetadata {
    #[serde(default)]
    label: String,
    #[serde(default)]
    tags: Vec<String>,
}

fn metadata_path(directory: &Path, file: &str) -> PathBuf {
    directory.join(format!("{file}.meta.json"))
}

fn read_snapshot_metadata(directory: &Path, file: &str) -> Option<SnapshotMetadata> {
    let bytes = fs::read(metadata_path(directory, file)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_snapshot_metadata(directory: &Path, entry: &SnapshotEntry) {
    if entry.label.trim().is_empty() && entry.tags.is_empty() {
        return;
    }
    let metadata = SnapshotMetadata {
        label: entry.label.clone(),
        tags: entry.tags.clone(),
    };
    if let Ok(bytes) = serde_json::to_vec_pretty(&metadata) {
        let _ = fs::write(metadata_path(directory, &entry.file), bytes);
    }
}

fn is_elevated() -> bool {
    #[cfg(test)]
    {
        // Keep the probe out of unit tests: it reads the process token and its
        // result is irrelevant for the path/snapshot tests below.
        false
    }
    #[cfg(not(test))]
    {
        #[cfg(target_os = "windows")]
        {
            windows_elevated()
        }

        #[cfg(not(target_os = "windows"))]
        {
            // SAFETY: geteuid 无参数、不会失败，返回调用进程的有效用户 ID。
            unsafe extern "C" {
                fn geteuid() -> u32;
            }
            // SAFETY: 同上；比较结果只是布尔判断，不涉及其他内存。
            unsafe { geteuid() == 0 }
        }
    }
}

/// Asks the process token whether this process runs with an elevated token.
///
/// The old probe ran `net session`, which is missing on trimmed Windows images
/// and therefore reported a plain wrong answer; the token class is authoritative.
#[cfg(all(not(test), target_os = "windows"))]
fn windows_elevated() -> bool {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr;

    #[repr(C)]
    struct TokenElevation {
        token_is_elevated: u32,
    }

    const TOKEN_QUERY: u32 = 0x0000_0008;
    const TOKEN_ELEVATION_CLASS: u32 = 20;

    #[link(name = "kernel32")]
    // SAFETY: 这三个导入只做声明；调用点各自保证参数有效，且均不会保存
    // 传入的指针。
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn GetLastError() -> u32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    #[link(name = "advapi32")]
    // SAFETY: 同上；令牌查询的参数契约在调用点的 SAFETY 注释里说明。
    unsafe extern "system" {
        fn OpenProcessToken(process: *mut c_void, access: u32, token: *mut *mut c_void) -> i32;
        fn GetTokenInformation(
            token: *mut c_void,
            class: u32,
            info: *mut c_void,
            len: u32,
            returned: *mut u32,
        ) -> i32;
    }

    // SAFETY: 只查询当前进程的令牌：句柄由 OpenProcessToken 返回并在读取
    // 后立刻 CloseHandle；TokenElevation 是 TOKEN_ELEVATION 的 repr(C) 布局，
    // 缓冲区大小由 size_of 给出，GetTokenInformation 不会越界写入。
    unsafe {
        let mut token: *mut c_void = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            report_token_error("OpenProcessToken", GetLastError());
            return false;
        }
        let mut elevation = TokenElevation {
            token_is_elevated: 0,
        };
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TOKEN_ELEVATION_CLASS,
            (&mut elevation as *mut TokenElevation).cast(),
            size_of::<TokenElevation>() as u32,
            &mut returned,
        );
        CloseHandle(token);
        if ok == 0 {
            report_token_error("GetTokenInformation", GetLastError());
            return false;
        }
        elevation.token_is_elevated != 0
    }
}

/// Logs a token query failure, except for ERROR_ACCESS_DENIED.
///
/// Access denied is not an error here: it means the process has a restricted
/// token and therefore is definitely not elevated.
#[cfg(all(not(test), target_os = "windows"))]
fn report_token_error(call: &str, code: u32) {
    const ERROR_ACCESS_DENIED: u32 = 5;
    if code != ERROR_ACCESS_DENIED {
        crate::log::warn(format!("{call} failed with error {code}"));
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
        // `\` is only a separator on Windows; on Unix it is an ordinary file
        // name character, so the path must still resolve inside the directory.
        let backslash = snapshot_path_in(directory, r"..\capture.dvsnap");
        if cfg!(windows) {
            assert!(backslash.is_none());
        } else {
            assert!(backslash.is_some_and(|path| path.starts_with(directory)));
        }
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
    fn newer_schema_versions_are_refused_with_an_actionable_message() {
        // 未来版本的快照会被 serde(default) 部分解析，缺的段落看起来就像硬件不存在，
        // 所以必须在读入时拒绝，并告诉用户该怎么办。
        let raw = br#"{"schema_version": 2, "captured_at_unix_ms": 1}"#;
        let error = parse_snapshot(raw).expect_err("future snapshot must be refused");
        assert!(error.contains("update Device Viewer"), "{error}");
    }

    #[test]
    fn older_schema_versions_still_open() {
        let raw = br#"{"schema_version": 0, "captured_at_unix_ms": 1}"#;
        let snapshot = parse_snapshot(raw).expect("older snapshot is readable");
        assert_eq!(snapshot.schema_version, 0);
        assert_eq!(snapshot.captured_at_unix_ms, 1);
    }

    #[test]
    fn timestamps_are_rendered_as_civil_dates() {
        // 本地时区随机器变化，这里只校验渲染形状，具体换算由下面一条测试覆盖。
        for stamp in [0u64, 3_661_000, 1_700_000_000_000] {
            let rendered = format_timestamp(stamp);
            assert_eq!(rendered.len(), 19, "unexpected timestamp: {rendered}");
            let bytes = rendered.as_bytes();
            assert_eq!(bytes[4], b'-');
            assert_eq!(bytes[7], b'-');
            assert_eq!(bytes[10], b' ');
            assert_eq!(bytes[13], b':');
            assert_eq!(bytes[16], b':');
            for (index, byte) in bytes.iter().enumerate() {
                let expected = matches!(index, 4 | 7) && *byte == b'-'
                    || index == 10 && *byte == b' '
                    || matches!(index, 13 | 16) && *byte == b':'
                    || byte.is_ascii_digit();
                assert!(expected, "unexpected byte {byte} at {index} in {rendered}");
            }
        }
    }

    #[test]
    fn timestamps_use_the_local_time_zone() {
        let utc: CivilTime = (2024, 3, 5, 12, 30, 0);
        let Some(local) = local_civil_time(utc) else {
            // 平台无法解析时区时必须显式标注 UTC，而不是冒充本地时间。
            assert!(format_timestamp(0).ends_with(" UTC"));
            return;
        };
        let stamp = (days_from_civil(utc.0, utc.1, utc.2) as u64) * 86_400_000
            + u64::from(utc.3) * 3_600_000
            + u64::from(utc.4) * 60_000
            + u64::from(utc.5) * 1_000;
        let rendered = format_timestamp(stamp);
        let expected = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            local.0, local.1, local.2, local.3, local.4, local.5
        );
        assert_eq!(rendered, expected);
        assert!(!rendered.ends_with(" UTC"));
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("device-viewer-{name}-{}", unique_stamp()));
        fs::create_dir_all(&dir).expect("create scratch directory");
        dir
    }

    fn scratch_state(dir: &Path) -> AppState {
        AppState {
            settings: Settings::default(),
            snapshot: HardwareSnapshot::default(),
            snapshots: Vec::new(),
            compare: None,
            compare_label: String::new(),
            compare_file: None,
            gpu_runtime: Vec::new(),
            permission: PermissionReport {
                elevated: false,
                items: Vec::new(),
            },
            update: UpdateState::Idle,
            ui: UiState::default(),
            config_path: dir.join(CONFIG_FILE),
            snapshots_dir: dir.join("snapshots"),
            exports_dir: dir.join("exports"),
            dirty: false,
            last_save: Instant::now(),
        }
    }

    #[test]
    fn corrupt_configuration_is_preserved_next_to_the_original() {
        let dir = scratch_dir("corrupt-config");
        let path = dir.join(CONFIG_FILE);
        fs::write(&path, b"{ not valid json").expect("write corrupt config");
        assert!(read_config(&path).is_none());
        let backup = path.with_extension("json.bak");
        assert!(!path.exists(), "corrupt config should be moved aside");
        assert_eq!(
            fs::read(&backup).expect("backup exists"),
            b"{ not valid json".to_vec()
        );
        fs::remove_dir_all(&dir).expect("clean scratch directory");
    }

    #[test]
    fn configuration_is_replaced_without_leaving_a_temp_file() {
        let dir = scratch_dir("atomic-config");
        let mut state = scratch_state(&dir);
        state.settings.language = LanguagePreference::Chinese;
        state.dirty = true;
        state.save_config();
        let saved = read_config(&state.config_path).expect("configuration is readable again");
        assert_eq!(saved.settings.language, LanguagePreference::Chinese);
        assert!(
            !state.config_path.with_extension("json.tmp").exists(),
            "temporary file must not survive a successful save"
        );
        fs::remove_dir_all(&dir).expect("clean scratch directory");
    }

    #[test]
    fn exports_are_written_outside_the_snapshot_directory() {
        let normal_exports = exports_dir(false);
        assert_ne!(normal_exports, snapshots_dir(false));
        assert_eq!(
            normal_exports.file_name().and_then(|name| name.to_str()),
            Some("exports")
        );
        assert_eq!(normal_exports.parent(), snapshots_dir(false).parent());
    }

    #[test]
    fn refresh_snapshots_ignores_exported_json() {
        let dir = scratch_dir("snapshot-filter");
        let mut state = scratch_state(&dir);
        fs::create_dir_all(&state.snapshots_dir).expect("create snapshot directory");
        fs::write(state.snapshots_dir.join("capture.dvsnap"), b"{}").expect("write snapshot");
        fs::write(
            state.snapshots_dir.join("device-viewer-export-1.json"),
            b"{}",
        )
        .expect("write export");
        state.refresh_snapshots();
        assert_eq!(state.snapshots.len(), 1);
        assert_eq!(state.snapshots[0].file, "capture.dvsnap");
        fs::remove_dir_all(&dir).expect("clean scratch directory");
    }

    #[test]
    fn tag_editor_buffer_survives_repaints() {
        let mut ui = UiState::default();
        ui.begin_tag_edit("a.dvsnap", &["one".to_string()]);
        assert_eq!(ui.tag_edit_buffer, "one");
        // 模拟用户输入后列表重建：同一快照再次渲染不得重置缓冲。
        ui.tag_edit_buffer = "one, two".to_string();
        ui.begin_tag_edit("a.dvsnap", &["one".to_string()]);
        assert_eq!(ui.tag_edit_buffer, "one, two");
        // 打开另一个快照时才重新初始化。
        ui.begin_tag_edit("b.dvsnap", &["three".to_string()]);
        assert_eq!(ui.tag_edit_buffer, "three");
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
