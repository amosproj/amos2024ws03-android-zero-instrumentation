// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

mod helpers;
mod server;
mod configuration;
mod constants;
mod ebpf_utils;

use tokio;


#[tokio::main]
async fn main() {
    env_logger::init();

    // apparently needed...
    helpers::bump_rlimit();

    server::serve_forever().await;
}

