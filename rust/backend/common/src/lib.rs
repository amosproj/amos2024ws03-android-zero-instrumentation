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
pub struct VfsWriteCall {
    pid: u32,
    tid: u32,
    begin_time_stamp: u64,
    fd: i32,
    bytes_written: usize,
}

impl VfsWriteCall {
    pub fn new(pid: u32, tid: u32, begin_time_stamp: u64, fd: i32, bytes_written: usize) -> Self {
        Self { pid, tid, begin_time_stamp, fd, bytes_written}
    }
}

#[inline(always)]
pub fn generate_id(pid: u32, tgid: u32) -> u64{
    let pid_u64 = pid as u64;
    let tgid_u64 = tgid as u64;

    (pid_u64 << 32) | tgid_u64
}


