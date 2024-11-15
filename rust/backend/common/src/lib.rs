#![no_std]

// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

#[derive(Debug, Copy, Clone)]
pub enum KProbeTypes {
    Poll,
    VfsWrite,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KProbeData {
    pub pid: u32,
    pub tid: u32,
    pub probe_type: KProbeTypes,
    pub ret: bool,
}
