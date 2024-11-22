// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use tracing_subscriber::EnvFilter;

mod configuration;
mod constants;
pub mod counter;
mod ebpf_utils;
mod utils;
mod server;
mod procfs_utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    // apparently needed...
    utils::bump_rlimit();

    server::serve_forever().await;
}
