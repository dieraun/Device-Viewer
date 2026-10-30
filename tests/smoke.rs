//! Integration-test skeleton for Device Viewer.
//!
//! Hardware-specific tests belong in modules that can run without requiring a
//! particular GPU or driver. Keep the base test target dependency-free.

#[test]
fn package_metadata_is_stable() {
    assert_eq!(env!("CARGO_PKG_NAME"), "device-viewer");
    assert_eq!(env!("CARGO_PKG_VERSION"), "1.0.0");
}
