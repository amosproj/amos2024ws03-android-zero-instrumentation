// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

mod configuration;
mod constants;
mod ebpf_utils;
mod helpers;
mod server;

#[tokio::main]
async fn main() {
    env_logger::init();

    // apparently needed...
    helpers::bump_rlimit();

    server::serve_forever().await;
}
