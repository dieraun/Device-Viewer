//! Khronos OpenVX runtime availability.
//!
//! The `openvx` crate intentionally exposes only opaque handles, so this module
//! reports whether a context can be created instead of inventing device details.

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenVxInfo {
    pub available: bool,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<OpenVxInfo> {
    super::run_blocking("openvx", cancel, platform_query).await
}

#[cfg(feature = "openvx")]
fn platform_query() -> QueryResult<OpenVxInfo> {
    use openvx::AsRaw;

    let context = openvx::types::VxContext::create();
    let available = !context.as_raw().is_null();
    let mut warnings = Vec::new();
    if !available {
        warnings.push("vxCreateContext returned a null context".to_string());
    }
    drop(context);

    Ok(OpenVxInfo {
        available,
        warnings,
    })
}

#[cfg(not(feature = "openvx"))]
fn platform_query() -> QueryResult<OpenVxInfo> {
    Err(QueryError::unsupported(
        "openvx",
        "binary was built without the `openvx` feature",
    ))
}
