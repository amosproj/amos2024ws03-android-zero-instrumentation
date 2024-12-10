// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

mod configuration;
mod constants;
pub mod counter;
mod ebpf_utils;
mod helpers;
mod procfs_utils;
mod server;
mod features;
mod collector;
mod vfs_write_feature;
mod sys_sendmsg_feature;
mod jni_reference_feature;
mod symbols;

pub async fn run_server() {
    helpers::bump_rlimit();

    server::serve_forever().await;
} 