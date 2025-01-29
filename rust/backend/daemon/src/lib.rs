// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

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

pub async fn run_server() {
    helpers::bump_rlimit();

    server::serve_forever_socket().await;
}
