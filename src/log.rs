//! Small internal logger used by the GUI and the CLI.
//!
//! The logger intentionally keeps only the most recent entries in memory and
//! appends every record to a file when one was configured. Logging is
//! best-effort: an unwritable log file must never take the application down.
//!
//! The file handle is opened once and reused. Re-opening the file for every
//! record costs milliseconds on Windows and stalls whichever thread is logging
//! (the UI thread and tokio workers both call into here). The file is rotated
//! to a single `.1` backup once it grows past [`MAX_LOG_BYTES`]; only one backup
//! is kept because multi-slot rotation is racy on Windows, where a file that is
//! still open cannot always be renamed.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::fs::{File, OpenOptions, create_dir_all, remove_file, rename};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 2_000;
const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub timestamp_ms: u64,
    pub level: Level,
    pub message: String,
}

struct Store {
    entries: VecDeque<Entry>,
    path: Option<PathBuf>,
    file: Option<File>,
}

static STORE: OnceLock<Mutex<Store>> = OnceLock::new();

fn store() -> &'static Mutex<Store> {
    STORE.get_or_init(|| {
        Mutex::new(Store {
            entries: VecDeque::new(),
            path: None,
            file: None,
        })
    })
}

/// Configures the log file. `None` keeps the log in memory only.
pub fn init(path: Option<PathBuf>) {
    let Some(path) = path else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }
    rotate(&path);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    if let Ok(mut store) = store().lock() {
        store.path = Some(path);
        store.file = file;
    }
}

/// Moves an oversized log out of the way before the handle is opened.
fn rotate(path: &Path) {
    let oversized = std::fs::metadata(path)
        .map(|metadata| metadata.len() > MAX_LOG_BYTES)
        .unwrap_or(false);
    if !oversized {
        return;
    }
    let backup = backup_path(path);
    let _ = remove_file(&backup);
    let _ = rename(path, backup);
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".1");
    PathBuf::from(name)
}

pub fn record(level: Level, message: impl Into<String>) {
    let message = message.into();
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default();
    let line = format!(
        "{timestamp_ms} [{}] {}\n",
        level.label(),
        message.replace('\n', "\\n")
    );

    if let Ok(mut store) = store().lock() {
        store.entries.push_back(Entry {
            timestamp_ms,
            level,
            message,
        });
        while store.entries.len() > MAX_ENTRIES {
            store.entries.pop_front();
        }
        if let Some(file) = store.file.as_mut() {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

pub fn debug(message: impl Into<String>) {
    record(Level::Debug, message);
}

pub fn info(message: impl Into<String>) {
    record(Level::Info, message);
}

pub fn warn(message: impl Into<String>) {
    record(Level::Warn, message);
}

pub fn error(message: impl Into<String>) {
    record(Level::Error, message);
}

pub fn entries() -> Vec<Entry> {
    store()
        .lock()
        .map(|store| store.entries.iter().cloned().collect())
        .unwrap_or_default()
}

pub fn clear() {
    if let Ok(mut store) = store().lock() {
        store.entries.clear();
        if let Some(file) = store.file.as_mut() {
            let _ = file.set_len(0);
            let _ = file.flush();
        }
    }
}

pub fn path() -> Option<PathBuf> {
    store().lock().ok().and_then(|store| store.path.clone())
}
