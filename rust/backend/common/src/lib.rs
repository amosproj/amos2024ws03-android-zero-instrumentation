#![no_std]

// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

pub const TIME_LIMIT_NS: u64 = 1_000_000;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VfsWriteCall {
    pub pid: u32,
    pub tid: u32,
    pub begin_time_stamp: u64,
    pub fp: u64,
    pub bytes_written: usize,
}

impl VfsWriteCall {
    pub fn new(pid: u32, tid: u32, begin_time_stamp: u64, fp: u64, bytes_written: usize) -> Self {
        Self { pid, tid, begin_time_stamp, fp, bytes_written}
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SysSendmsgCall {
    pub pid: u32,
    pub tid: u32,
    pub begin_time_stamp: u64,
    pub fd: u64,
    pub duration_micro_sec: u64, // in microseconds
}

impl SysSendmsgCall {
    pub fn new(pid: u32, tid: u32, begin_time_stamp: u64, fd: u64, duration_micro_sec: u64) -> Self {
        Self { pid, tid, begin_time_stamp, fd, duration_micro_sec }
    }
}

#[inline(always)]
pub fn generate_id(pid: u32, tgid: u32) -> u64{
    let pid_u64 = pid as u64;
    let tgid_u64 = tgid as u64;

    (pid_u64 << 32) | tgid_u64
}