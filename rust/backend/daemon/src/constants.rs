// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{net::SocketAddr, time};

pub(crate) const DEV_DEFAULT_FILE_PATH: &str = "./ziofa.json";

pub fn sock_addr() -> SocketAddr {
    "[::1]:50051".parse().expect("is valid address")
}

pub const ZIOFA_EBPF_PATH: &str = "/sys/fs/bpf/ziofa";

pub const _DEFAULT_TIMEFRAME: time::Duration = time::Duration::from_secs(1);
pub const TIMESERIES_LENGTH: usize = 40;
pub const INDEX_PATH: &str = "/data/local/tmp/index";

// Update via downloading the submodules in rust/garbage-collection
// and running `cargo run --bin parser --features cli`
#[cfg(target_arch = "x86_64")]
pub const GC_HEAP_META_JSON: &str = r#"{"target_footprint":{"offset":456,"size":8},"num_bytes_allocated":{"offset":544,"size":8},"gc_cause":{"offset":592,"size":4},"duration_ns":{"offset":600,"size":8},"freed_objects":{"offset":656,"size":8},"freed_bytes":{"offset":664,"size":8},"freed_los_objects":{"offset":672,"size":8},"freed_los_bytes":{"offset":680,"size":8},"gcs_completed":{"offset":1040,"size":4},"pause_times_begin":{"offset":696,"size":8},"pause_times_end":{"offset":704,"size":8}}"#;

#[cfg(target_arch = "aarch64")]
pub const GC_HEAP_META_JSON: &str = r#"{"target_footprint":{"offset":456,"size":8},"num_bytes_allocated":{"offset":544,"size":8},"gc_cause":{"offset":592,"size":4},"duration_ns":{"offset":600,"size":8},"freed_objects":{"offset":656,"size":8},"freed_bytes":{"offset":664,"size":8},"freed_los_objects":{"offset":672,"size":8},"freed_los_bytes":{"offset":680,"size":8},"gcs_completed":{"offset":1040,"size":4},"pause_times_begin":{"offset":696,"size":8},"pause_times_end":{"offset":704,"size":8}}"#;
