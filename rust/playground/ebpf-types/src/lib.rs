#![no_std]

// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use bytemuck::{AnyBitPattern, CheckedBitPattern};

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct TaskContext {
    /// PID in userspace
    pub pid: u32,
    /// TID in userspace
    pub tid: u32,
    /// Parent PID in userspace
    pub ppid: u32,
    /// comm
    pub comm: [u8; 16],
    /// cmdline
    pub cmdline: [u8; 256],
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            pid: 0,
            tid: 0,
            ppid: 0,
            comm: [0; 16],
            cmdline: [0; 256],
        }
    }
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TaskContext {}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct EventContext {
    pub task: TaskContext,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(C, align(8))]
pub struct Event {
    pub context: EventContext,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(C)]
pub enum EventKind {
    VfsWrite(VfsWrite),
    SendMsg(SendMsg),
    Jni(Jni),
    SigQuit(SigQuit),
    GarbaceCollect(GarbageCollect),
    FileOp(FileDescriptorOp),    
}


#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct VfsWrite {
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct SendMsg {
    pub file_descriptor: u64,
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(u8)]
pub enum Jni {
    AddLocalRef,
    DeleteLocalRef,
    AddGlobalRef,
    DeleteGlobalRef,
}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct SigQuit {
    pub target_pid: u32,
}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct GarbageCollect {
    pub _unused: [u8; 0]
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(u8)]
pub enum FileDescriptorOp {
    Create,
    Destroy,
}