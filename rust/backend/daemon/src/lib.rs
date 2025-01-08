// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

mod constants;
mod ebpf_utils;
mod helpers;
mod procfs_utils;
mod server;
mod features;
mod collector;
mod symbols;
mod registry;
mod filesystem;

pub async fn run_server() {
    helpers::bump_rlimit();

    server::serve_forever().await;
} 