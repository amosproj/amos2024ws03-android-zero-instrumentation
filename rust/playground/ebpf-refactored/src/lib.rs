// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

#![cfg_attr(not(test), no_std)]
#![allow(internal_features)]
#![feature(core_intrinsics)]

pub mod bounds_check;
pub mod cache;
pub mod event_local;
pub mod events;
pub mod filter;
pub mod iterator_ext;
pub mod maps;
pub mod path;
pub mod pipeline;
pub mod scratch;
pub mod syscalls;
pub mod task_ext;

extern crate aya_log_ebpf;
