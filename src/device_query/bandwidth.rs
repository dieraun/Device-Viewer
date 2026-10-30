//! Lightweight memory bandwidth benchmark.
//!
//! This is intentionally not a network speed test: it measures the memory
//! subsystem with a sequential write, read and copy pass over a fixed buffer.
//! The workload is small enough to run during snapshot collection and observes
//! the collection cancellation token between passes.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::{QueryError, QueryResult};

const BUFFER_BYTES: usize = 32 * 1024 * 1024;
const PASSES: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BandwidthSample {
    pub operation: String,
    pub bytes: u64,
    pub elapsed_ms: f64,
    pub megabytes_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BandwidthInfo {
    pub buffer_bytes: u64,
    pub read_megabytes_per_second: Option<f64>,
    pub write_megabytes_per_second: Option<f64>,
    pub copy_megabytes_per_second: Option<f64>,
    pub samples: Vec<BandwidthSample>,
    pub warnings: Vec<String>,
}

pub async fn query(cancel: &CancellationToken) -> QueryResult<BandwidthInfo> {
    let token = cancel.clone();
    super::run_blocking("bandwidth", cancel, move || run_benchmark(&token)).await
}

fn run_benchmark(cancel: &CancellationToken) -> QueryResult<BandwidthInfo> {
    if cancel.is_cancelled() {
        return Err(QueryError::cancelled("bandwidth"));
    }

    let mut source = vec![0u8; BUFFER_BYTES];
    let mut destination = vec![0u8; BUFFER_BYTES];
    let mut info = BandwidthInfo {
        buffer_bytes: BUFFER_BYTES as u64,
        ..BandwidthInfo::default()
    };

    for pass in 0..PASSES {
        if cancel.is_cancelled() {
            return Err(QueryError::cancelled("bandwidth"));
        }

        let started = Instant::now();
        source.fill((pass as u8).wrapping_mul(37).wrapping_add(11));
        let write_elapsed = started.elapsed();
        info.samples.push(sample("write", write_elapsed));
        info.write_megabytes_per_second = Some(info.write_megabytes_per_second.map_or_else(
            || mbps(BUFFER_BYTES, write_elapsed),
            |best| best.max(mbps(BUFFER_BYTES, write_elapsed)),
        ));

        if cancel.is_cancelled() {
            return Err(QueryError::cancelled("bandwidth"));
        }

        let started = Instant::now();
        let mut checksum = 0u64;
        for byte in &source {
            checksum = checksum.wrapping_add(u64::from(*byte));
        }
        std::hint::black_box(checksum);
        let read_elapsed = started.elapsed();
        info.samples.push(sample("read", read_elapsed));
        info.read_megabytes_per_second = Some(info.read_megabytes_per_second.map_or_else(
            || mbps(BUFFER_BYTES, read_elapsed),
            |best| best.max(mbps(BUFFER_BYTES, read_elapsed)),
        ));

        if cancel.is_cancelled() {
            return Err(QueryError::cancelled("bandwidth"));
        }

        let started = Instant::now();
        destination.copy_from_slice(&source);
        let copy_elapsed = started.elapsed();
        std::hint::black_box(&destination);
        info.samples.push(sample("copy", copy_elapsed));
        info.copy_megabytes_per_second = Some(info.copy_megabytes_per_second.map_or_else(
            || mbps(BUFFER_BYTES, copy_elapsed),
            |best| best.max(mbps(BUFFER_BYTES, copy_elapsed)),
        ));
    }

    Ok(info)
}

fn sample(operation: &str, elapsed: std::time::Duration) -> BandwidthSample {
    BandwidthSample {
        operation: operation.to_string(),
        bytes: BUFFER_BYTES as u64,
        elapsed_ms: elapsed.as_secs_f64() * 1000.0,
        megabytes_per_second: mbps(BUFFER_BYTES, elapsed),
    }
}

fn mbps(bytes: usize, elapsed: std::time::Duration) -> f64 {
    let seconds = elapsed.as_secs_f64();
    if seconds <= f64::EPSILON {
        return 0.0;
    }
    bytes as f64 / seconds / 1_000_000.0
}
