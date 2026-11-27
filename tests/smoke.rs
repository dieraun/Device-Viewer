//! Integration-test skeleton for Device Viewer.
//!
//! Hardware-specific tests belong in modules that can run without requiring a
//! particular GPU or driver. Keep the base test target dependency-free.

#[test]
fn package_metadata_is_stable() {
    assert_eq!(env!("CARGO_PKG_NAME"), "device-viewer");
    let version: &str = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty(), "version must not be empty");
    let segments: Vec<&str> = version.split('.').collect();
    assert!(
        segments.len() >= 2,
        "version must have at least two dot-separated segments: {version}"
    );
    for segment in &segments {
        assert!(
            !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_digit()),
            "version segment must be numeric: {segment} in {version}"
        );
    }
}
