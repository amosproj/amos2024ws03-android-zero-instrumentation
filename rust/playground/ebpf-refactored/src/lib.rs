// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

#![cfg_attr(not(test), no_std)]

pub mod bounds_check;
pub mod iterator_ext;
pub mod path;

#[cfg(target_arch = "bpf")]
pub mod programs;

#[cfg(target_arch = "bpf")]
pub mod task_info;

#[cfg(target_arch = "bpf")]
pub mod process_info;

#[cfg(target_arch = "bpf")]
pub mod pipeline;

#[cfg(target_arch = "bpf")]
pub mod write;

#[cfg(target_arch = "bpf")]
pub mod blocking;

#[cfg(target_arch = "bpf")]
pub mod signal;

#[cfg(target_arch = "bpf")]
pub mod syscalls;

#[cfg(target_arch = "bpf")]
pub mod fdtracking;
