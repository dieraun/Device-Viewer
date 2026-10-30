//! Small internal logger used by the GUI and the CLI.
//!
//! The logger intentionally keeps only the most recent entries in memory and
//! appends every record to a file when one was configured. Logging is
//! best-effort: an unwritable log file must never take the application down.

use std::collections::VecDeque;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 2_000;

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
}

static STORE: OnceLock<Mutex<Store>> = OnceLock::new();

fn store() -> &'static Mutex<Store> {
    STORE.get_or_init(|| {
        Mutex::new(Store {
            entries: VecDeque::new(),
            path: None,
        })
    })
}

/// Configures the log file. `None` keeps the log in memory only.
pub fn init(path: Option<PathBuf>) {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        if let Ok(mut store) = store().lock() {
            store.path = Some(path);
        }
    }
}

pub fn record(level: Level, message: impl Into<String>) {
    let message = message.into();
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default();

    if let Ok(mut store) = store().lock() {
        store.entries.push_back(Entry {
            timestamp_ms,
            level,
            message: message.clone(),
        });
        while store.entries.len() > MAX_ENTRIES {
            store.entries.pop_front();
        }

        if let Some(path) = &store.path {
            append_line(path, timestamp_ms, level, &message);
        }
    }
}

fn append_line(path: &Path, timestamp_ms: u64, level: Level, message: &str) {
    let line = format!(
        "{timestamp_ms} [{}] {}\n",
        level.label(),
        message.replace('\n', "\\n")
    );
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
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
        if let Some(path) = &store.path {
            let _ = std::fs::write(path, b"");
        }
    }
}

pub fn path() -> Option<PathBuf> {
    store().lock().ok().and_then(|store| store.path.clone())
}
