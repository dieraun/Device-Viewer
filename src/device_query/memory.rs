//! Physical memory and swap.

use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};
use super::run_blocking;

/// One memory module / DIMM.
///
/// Filling this requires SMBIOS access; it stays empty on platforms where the
/// information is not exposed to user space. The field is kept so snapshots
/// from future backends remain compatible.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryModule {
    pub slot: Option<String>,
    pub size_bytes: Option<u64>,
    pub speed_mt_s: Option<u32>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub memory_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub modules: Vec<MemoryModule>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<MemoryInfo> {
    run_blocking("memory", cancel, || {
        let system = System::new_all().map_err(|error| {
            QueryError::backend("memory", "the operating system refused memory enumeration")
                .with_detail(error.to_string())
        })?;

        let total = system.total_memory();
        let used = system.used_memory();
        let swap_total = system.total_swap();
        let swap_used = system.used_swap();

        Ok(MemoryInfo {
            total_bytes: total,
            used_bytes: used,
            free_bytes: total.saturating_sub(used),
            available_bytes: total.saturating_sub(used),
            swap_total_bytes: swap_total,
            swap_used_bytes: swap_used,
            modules: Vec::new(),
            warnings: Vec::new(),
        })
    })
    .await
}
