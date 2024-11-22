// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

mod configuration;
mod constants;
pub mod counter;
mod ebpf_utils;
mod utils;
mod procfs_utils;
mod server;
mod map_collectors;

pub async fn run_server() {
    utils::bump_rlimit();

    server::serve_forever().await;
} 