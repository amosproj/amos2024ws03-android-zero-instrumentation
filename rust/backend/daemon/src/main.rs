// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use tracing_subscriber::EnvFilter;
mod collector;
mod constants;
mod ebpf_utils;
mod features;
mod filesystem;
mod helpers;
mod procfs_utils;
mod registry;
mod server;
mod symbols;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    // apparently needed...
    helpers::bump_rlimit();

    server::serve_forever_socket().await;
}
