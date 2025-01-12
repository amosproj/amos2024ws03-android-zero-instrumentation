#![no_std]

// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use bytemuck::{checked::CheckedCastError, AnyBitPattern, CheckedBitPattern};


pub trait TryFromRaw: Sized {
    fn try_from_raw(raw: &[u8]) -> Result<Self, CheckedCastError>;
}

impl TryFromRaw for VfsWriteCall {
    fn try_from_raw(raw: &[u8]) -> Result<Self, CheckedCastError> {
        Ok(*bytemuck::try_from_bytes(raw)?)
    }
}

impl TryFromRaw for SysSendmsgCall {
    fn try_from_raw(raw: &[u8]) -> Result<Self, CheckedCastError> {
        Ok(*bytemuck::try_from_bytes(raw)?)
    }
}

impl TryFromRaw for JNICall {
    fn try_from_raw(raw: &[u8]) -> Result<Self, CheckedCastError> {
        Ok(*bytemuck::checked::try_from_bytes(raw)?)
    }
}

impl TryFromRaw for SysSigquitCall {
    fn try_from_raw(raw: &[u8]) -> Result<Self, CheckedCastError> {
        Ok(*bytemuck::try_from_bytes(raw)?)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AnyBitPattern)]
pub struct VfsWriteCall {
    pub pid: u32,
    pub tid: u32,
    pub begin_time_stamp: u64,
    pub fp: u64,
    pub bytes_written: usize,
}

impl VfsWriteCall {
    pub fn new(pid: u32, tid: u32, begin_time_stamp: u64, fp: u64, bytes_written: usize) -> Self {
        Self {
            pid,
            tid,
            begin_time_stamp,
            fp,
            bytes_written,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AnyBitPattern)]
pub struct SysSendmsgCall {
    pub pid: u32,
    pub tid: u32,
    pub begin_time_stamp: u64,
    pub fd: u64,
    pub duration_nano_sec: u64, // in nanoseconds
}

impl SysSendmsgCall {
    pub fn new(pid: u32, tid: u32, begin_time_stamp: u64, fd: u64, duration_nano_sec: u64) -> Self {
        Self {
            pid,
            tid,
            begin_time_stamp,
            fd,
            duration_nano_sec,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, CheckedBitPattern)]
pub enum JNIMethodName {
    AddLocalRef,
    DeleteLocalRef,
    AddGlobalRef,
    DeleteGlobalRef,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, CheckedBitPattern)]
pub struct JNICall {
    pub pid: u32,
    pub tid: u32,
    pub begin_time_stamp: u64,
    pub method_name: JNIMethodName,
}

// ---------------------------------------
// SysSigquit: detect SIGQUIT signals

#[repr(C)]
#[derive(Debug, Copy, Clone, AnyBitPattern)]
pub struct SysSigquitCall {
    pub pid: u32,
    pub tid: u32,
    pub time_stamp: u64,
    pub target_pid: u64, // the pid, that gets the SIGQUIT signal
}

impl SysSigquitCall {
    pub fn new(pid: u32, tid: u32, time_stamp: u64, target_pid: u64) -> Self {
        Self {
            pid,
            tid,
            time_stamp,
            target_pid,
        }
    }
}


// -----------------------------------------
// Detect blocking main-thread

#[repr(u8)]
#[derive(Debug, Copy, Clone, CheckedBitPattern)]
pub enum MainBlockingCause {
    Futex,
    Poll,
    Select,
    VfsWrite,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, CheckedBitPattern)]
pub struct BlockingMainCall {
    pub pid: u32,
    pub tid: u32,
    pub begin_time_stamp: u64,
    pub duration_nano_sec: u64, // in nanoseconds
    pub blocking_cause: MainBlockingCause,
}

impl BlockingMainCall {
    pub fn new(pid: u32, tid: u32, begin_time_stamp: u64, duration_nano_sec: u64, blocking_cause: MainBlockingCause) -> Self {
        Self {
            pid,
            tid,
            begin_time_stamp,
            duration_nano_sec,
            blocking_cause,
        }
    }
}



// ----------------------------------
// generate a unique id from pid and tid
#[inline(always)]
pub fn generate_id(pid: u32, tgid: u32) -> u64 {
    let pid_u64 = pid as u64;
    let tgid_u64 = tgid as u64;

    (pid_u64 << 32) | tgid_u64
}
