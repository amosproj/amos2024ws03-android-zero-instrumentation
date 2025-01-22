#![cfg_attr(not(test), no_std)]

// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

pub mod relocation_helpers;

#[cfg(target_arch = "bpf")]
pub mod programs;

#[cfg(target_arch = "bpf")]
pub mod task_info;
