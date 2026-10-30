//! Unified error model for every hardware query module.
//!
//! A query may fail in two different ways:
//!
//! * [`ErrorLevel::Warning`] – the data is unavailable because the platform does
//!   not expose it, the user lacks permission, or an optional tool is missing.
//!   Collection continues and the rest of the snapshot stays usable.
//! * [`ErrorLevel::Error`] – the module itself failed unexpectedly.
//!
//! Keeping the severity inside the error lets the scheduler forward every
//! failure to the UI without ever aborting the whole collection pass.

use std::fmt;
use std::io;

use serde::{Deserialize, Serialize};

/// Severity attached to a failed query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErrorLevel {
    /// Non-fatal: the remaining modules keep running.
    #[default]
    Warning,
    /// The module could not produce data at all.
    Error,
    /// A level written by a newer build. Treated as an error, not a warning.
    #[serde(other)]
    Unknown,
}

impl ErrorLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Unknown => "unknown",
        }
    }

    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }
}

impl fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Machine readable reason for a failed query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueryErrorKind {
    /// The feature is not implemented or not exposed on this platform.
    #[default]
    Unsupported,
    /// The operating system refused access (privileges, sandbox, ACL).
    PermissionDenied,
    /// The per-module deadline expired.
    Timeout,
    /// The caller cancelled the collection token.
    Cancelled,
    /// An operating system or device I/O call failed.
    Io,
    /// A backend returned data that could not be parsed.
    Parse,
    /// A graphics / compute backend reported an error.
    Backend,
    /// An optional external tool (for example `smartctl`) was not found.
    NotFound,
    /// An internal invariant was violated.
    Internal,
    /// A kind written by a newer build. Keeps old readers forward compatible.
    #[serde(other)]
    Unknown,
}

impl QueryErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::PermissionDenied => "permission_denied",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
            Self::Io => "io",
            Self::Parse => "parse",
            Self::Backend => "backend",
            Self::NotFound => "not_found",
            Self::Internal => "internal",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for QueryErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned by a single hardware query.
///
/// The struct is serializable because the snapshot exporter keeps the errors
/// next to the collected data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryError {
    /// Stable module name, for example `cpu` or `vulkan`.
    pub module: String,
    pub kind: QueryErrorKind,
    pub level: ErrorLevel,
    /// Short human readable message.
    pub message: String,
    /// Optional low level detail (driver message, OS error, ...).
    pub detail: Option<String>,
}

impl QueryError {
    pub fn new(
        module: impl Into<String>,
        kind: QueryErrorKind,
        level: ErrorLevel,
        message: impl Into<String>,
    ) -> Self {
        Self {
            module: module.into(),
            kind,
            level,
            message: message.into(),
            detail: None,
        }
    }

    /// A non-fatal condition. Collection continues.
    pub fn warning(
        module: impl Into<String>,
        kind: QueryErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self::new(module, kind, ErrorLevel::Warning, message)
    }

    /// A hard failure of a single module. Collection still continues.
    pub fn error(
        module: impl Into<String>,
        kind: QueryErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self::new(module, kind, ErrorLevel::Error, message)
    }

    pub fn unsupported(module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::warning(module, QueryErrorKind::Unsupported, message)
    }

    #[allow(dead_code)] // exercised by the integration test suite
    pub fn permission_denied(module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::warning(module, QueryErrorKind::PermissionDenied, message)
    }

    #[allow(dead_code)] // used by platform-specific parsers
    pub fn not_found(module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::warning(module, QueryErrorKind::NotFound, message)
    }

    pub fn timeout(module: impl Into<String>, timeout: std::time::Duration) -> Self {
        let module = module.into();
        Self::new(
            module,
            QueryErrorKind::Timeout,
            ErrorLevel::Error,
            format!("query exceeded its deadline of {timeout:?}"),
        )
    }

    pub fn cancelled(module: impl Into<String>) -> Self {
        let module = module.into();
        Self::new(
            module,
            QueryErrorKind::Cancelled,
            ErrorLevel::Warning,
            "query cancelled by caller",
        )
    }

    pub fn backend(module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::warning(module, QueryErrorKind::Backend, message)
    }

    #[allow(dead_code)] // used by platform parsers and the integration test suite
    pub fn parse(module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::warning(module, QueryErrorKind::Parse, message)
    }

    #[allow(dead_code)] // used by platform-specific parsers
    pub fn io(module: impl Into<String>, message: impl Into<String>, source: &io::Error) -> Self {
        let level = match source.kind() {
            io::ErrorKind::PermissionDenied => ErrorLevel::Warning,
            io::ErrorKind::NotFound => ErrorLevel::Warning,
            _ => ErrorLevel::Error,
        };
        let kind = match source.kind() {
            io::ErrorKind::PermissionDenied => QueryErrorKind::PermissionDenied,
            io::ErrorKind::NotFound => QueryErrorKind::NotFound,
            _ => QueryErrorKind::Io,
        };
        let mut error = Self::new(module, kind, level, message);
        error.detail = Some(source.to_string());
        error
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub const fn is_warning(&self) -> bool {
        self.level.is_warning()
    }

    pub const fn is_error(&self) -> bool {
        !self.level.is_warning()
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} ({})",
            self.level, self.module, self.message, self.kind
        )?;
        if let Some(detail) = &self.detail {
            write!(f, " – {detail}")?;
        }
        Ok(())
    }
}

impl std::error::Error for QueryError {}

/// Every query module returns this result type.
pub type QueryResult<T> = Result<T, QueryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_denied_is_a_warning() {
        let source = io::Error::from(io::ErrorKind::PermissionDenied);
        let error = QueryError::io("sensors", "cannot read hwmon", &source);
        assert_eq!(error.kind, QueryErrorKind::PermissionDenied);
        assert!(error.is_warning());
    }

    #[test]
    fn missing_tool_is_a_warning() {
        let error = QueryError::not_found("storage", "smartctl is not installed");
        assert!(error.is_warning());
        assert_eq!(error.kind, QueryErrorKind::NotFound);
    }

    #[test]
    fn timeout_is_an_error() {
        let error = QueryError::timeout("storage", std::time::Duration::from_secs(3));
        assert!(error.is_error());
        assert_eq!(error.kind, QueryErrorKind::Timeout);
    }

    #[test]
    fn errors_round_trip_through_json() {
        let error = QueryError::backend("vulkan", "loader missing").with_detail("libvulkan.so");
        let json = serde_json::to_string(&error).expect("serialize");
        let decoded: QueryError = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.module, "vulkan");
        assert_eq!(decoded.level, ErrorLevel::Warning);
        assert_eq!(decoded.detail.as_deref(), Some("libvulkan.so"));
    }
}
