// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

pub(crate) const DEV_DEFAULT_FILE_PATH: &str = "./ziofa.json";

pub fn sock_addr() -> SocketAddr {
    "[::1]:50051".parse().expect("is valid address")
}

pub const OATDUMP_PATH: &str = "/data/local/tmp/dump.json";
pub const ZIOFA_EBPF_PATH: &str = "/sys/fs/bpf/ziofa";
pub const INDEX_PATH: &str = "/data/local/tmp/index";