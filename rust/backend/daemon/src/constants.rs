// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

pub(crate) const DEV_DEFAULT_CONFIG_PATH: &str = "./ziofa.json";

pub fn sock_addr() -> SocketAddr {
    // "learn rust" they said, "it's a great language" they said
    "[::1]:50051".parse().expect("is valid address")
}
