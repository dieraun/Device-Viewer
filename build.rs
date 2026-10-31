//! Build metadata for Device Viewer.
//!
//! This script has no third-party build dependencies. It exposes the build
//! target, profile, package version, source revision and a reproducible
//! timestamp as compile-time environment variables:
//!
//! `DV_BUILD_TARGET`, `DV_BUILD_PROFILE`, `DV_BUILD_VERSION`,
//! `DV_BUILD_COMMIT`, `DV_BUILD_TIMESTAMP`, `DV_RUSTC_VERSION`.
//!
//! The values are optional metadata; the application does not require them at
//! runtime. `SOURCE_DATE_EPOCH` is honoured for reproducible builds.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=DV_BUILD_COMMIT");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rerun-if-env-changed=RUSTC");

    let target = env_value("TARGET").unwrap_or_else(|| "unknown".to_string());
    let profile = env_value("PROFILE").unwrap_or_else(|| "unknown".to_string());
    let version = env_value("CARGO_PKG_VERSION").unwrap_or_else(|| "0.0.0".to_string());

    let commit = env_value("DV_BUILD_COMMIT")
        .or_else(|| env_value("GITHUB_SHA"))
        .or_else(git_revision)
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = env_value("SOURCE_DATE_EPOCH")
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(now_unix_seconds)
        .unwrap_or(0);

    emit("DV_BUILD_TARGET", &target);
    emit("DV_BUILD_PROFILE", &profile);
    emit("DV_BUILD_VERSION", &version);
    emit("DV_BUILD_COMMIT", &commit);
    emit("DV_BUILD_TIMESTAMP", &timestamp.to_string());
    emit(
        "DV_RUSTC_VERSION",
        &rustc_version().unwrap_or_else(|| "unknown".to_string()),
    );
}

fn rustc_version() -> Option<String> {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8(output.stdout).ok()?;
    version.lines().next().map(|line| sanitize(line.trim()))
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| sanitize(&value))
        .filter(|value| !value.is_empty())
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let revision = String::from_utf8(output.stdout).ok()?;
    let revision = sanitize(revision.trim());
    if revision.is_empty() {
        None
    } else {
        Some(revision)
    }
}

fn now_unix_seconds() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn emit(name: &str, value: &str) {
    println!("cargo:rustc-env={name}={}", sanitize(value));
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_graphic() || *character == ' ')
        .take(128)
        .collect()
}
