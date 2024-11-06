// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

mod client;


pub use client::{Client, ClientError, Result};

#[cfg(feature = "uniffi")]
pub mod bindings;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();