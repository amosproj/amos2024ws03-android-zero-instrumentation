// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::maps::HashMap;

#[repr(C)]
pub struct Args<T> {
    pub args: [u64; 6],
    pub extra: T,
}

pub struct EventLocal<T: 'static>(&'static HashMap<u64, Args<T>>);

impl<T> EventLocal<T> {}