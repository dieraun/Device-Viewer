//! CPU identity, topology and live load.

use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};
use super::run_blocking;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuCore {
    pub id: usize,
    /// Instantaneous utilization in percent.
    pub usage_percent: f32,
    /// Current clock in MHz.
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    pub vendor: String,
    pub brand: String,
    pub architecture: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub global_usage_percent: f32,
    pub cores: Vec<CpuCore>,
    pub load_average: Option<[f64; 3]>,
    pub warnings: Vec<String>,
}

/// Collects CPU data. The sysinfo refresh performs blocking syscalls.
pub async fn query(cancel: &CancellationToken) -> QueryResult<CpuInfo> {
    run_blocking("cpu", cancel, || {
        let mut system = System::new_all().map_err(|error| {
            QueryError::backend("cpu", "the operating system refused CPU enumeration")
                .with_detail(error.to_string())
        })?;
        // sysinfo needs two samples to compute a meaningful load percentage.
        std::thread::sleep(std::time::Duration::from_millis(120));
        system.refresh_cpu_all();

        let processors = system.cpus();
        if processors.is_empty() {
            return Err(QueryError::error(
                "cpu",
                super::error::QueryErrorKind::Backend,
                "the operating system reported no logical processors",
            ));
        }

        let first = &processors[0];
        let brand = if first.brand().trim().is_empty() {
            first.name().to_string()
        } else {
            first.brand().to_string()
        };

        let logical_cores = processors.len();
        let physical_cores = System::physical_core_count()
            .unwrap_or(logical_cores)
            .max(1);

        let cores = processors
            .iter()
            .enumerate()
            .map(|(index, cpu)| CpuCore {
                id: index,
                usage_percent: cpu.usage(),
                frequency_mhz: cpu.frequency(),
            })
            .collect();

        let load = System::load_average().ok();
        Ok(CpuInfo {
            vendor: first.vendor_id().to_string(),
            brand,
            architecture: System::cpu_arch(),
            physical_cores,
            logical_cores,
            global_usage_percent: system.global_cpu_usage(),
            cores,
            load_average: load.map(|load| [load.one, load.five, load.fifteen]),
            warnings: Vec::new(),
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cpu_query_returns_a_model() {
        let cancel = CancellationToken::new();
        let info = query(&cancel).await.expect("cpu query");
        assert!(!info.brand.is_empty());
        assert_eq!(info.logical_cores, info.cores.len());
    }
}
