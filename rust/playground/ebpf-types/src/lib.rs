#![no_std]

// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use bytemuck::{AnyBitPattern, CheckedBitPattern, Zeroable};

#[derive(Debug, Clone, Copy, Default, AnyBitPattern)]
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
}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
pub struct ProcessContext {
    pub cmdline: [u8; 256],
    pub exe_path: [u8; 4096],
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            cmdline: [0; 256],
            exe_path: [0; 4096],
        }
    }
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TaskContext {}

#[cfg(feature = "user")]
unsafe impl aya::Pod for ProcessContext {}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct EventContext {
    pub task: TaskContext,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Event<T> {
    pub context: EventContext,
    pub kind: EventKind,
    pub data: T
}

#[derive(Clone, Copy)]
pub struct EventBits<T: CheckedBitPattern> {
    context: <EventContext as CheckedBitPattern>::Bits,
    kind: <EventKind as CheckedBitPattern>::Bits,
    data: <T as CheckedBitPattern>::Bits,
}
unsafe impl<T: CheckedBitPattern + 'static> Zeroable for EventBits<T> {}
unsafe impl<T: CheckedBitPattern + 'static> AnyBitPattern for EventBits<T> {}

unsafe impl<T: CheckedBitPattern + 'static> CheckedBitPattern for Event<T> {
    type Bits = EventBits<T>;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        EventContext::is_valid_bit_pattern(&bits.context) 
            && EventKind::is_valid_bit_pattern(&bits.kind)
            && T::is_valid_bit_pattern(&bits.data)
    }
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(u8)]
pub enum EventKind {
    Write,
    SendMsg,
    Jni,
    SigQuit,
    GarbaceCollect,
    FileOp,
}


#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(C)]
pub struct Write {
    pub bytes_written: u64,
    pub file_path: [u8; 4096],
    pub source: WriteSource
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(u8)]
pub enum WriteSource {
    /// Corresponds to `write` syscall
    Write,
    /// Corresponds to `pwrite64` syscall
    WriteV,
    /// Corresponds to `pwritev` syscall
    Write64,
    /// Corresponds to `pwritev2` syscall
    WriteV2,
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