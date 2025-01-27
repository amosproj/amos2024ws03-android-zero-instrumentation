#![no_std]

// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use bytemuck::{AnyBitPattern, CheckedBitPattern, PodInOption, Zeroable, ZeroableInOption};

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
pub struct Event<T: ?Sized> {
    pub kind: EventKind,
    pub context: EventContext,
    pub data: T,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EventBits<T: CheckedBitPattern> {
    kind: <EventKind as CheckedBitPattern>::Bits,
    context: <EventContext as CheckedBitPattern>::Bits,
    data: <T as CheckedBitPattern>::Bits,
}
unsafe impl<T: CheckedBitPattern + 'static> Zeroable for EventBits<T> {}
unsafe impl<T: CheckedBitPattern + 'static> AnyBitPattern for EventBits<T> {}

unsafe impl<T: CheckedBitPattern + 'static> CheckedBitPattern for Event<T> {
    type Bits = EventBits<T>;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        EventKind::is_valid_bit_pattern(&bits.kind)
            && EventContext::is_valid_bit_pattern(&bits.context)
            && T::is_valid_bit_pattern(&bits.data)
    }
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(C)]
pub struct Write {
    pub bytes_written: u64,
    pub file_path: [u8; 4096],
    pub source: WriteSource,
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
pub struct Blocking {
    pub syscall_id: u64,
    pub duration: u64,
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
pub struct Signal {
    pub target_pid: i32,
    pub signal: u32,
}

#[derive(Debug, Clone, Copy, AnyBitPattern)]
#[repr(C)]
pub struct GarbageCollect {
    pub _unused: [u8; 0],
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(C)]
pub struct FileDescriptorChange {
    pub open_fds: u64,
    pub operation: FileDescriptorOp,
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(u8)]
pub enum FileDescriptorOp {
    Open,
    Close,
}

#[derive(Debug, Clone, Copy, CheckedBitPattern)]
#[repr(u8)]
pub enum EventKind {
    Write,
    Blocking,
    Signal,
    GarbageCollect,
    FileDescriptorChange,
    Jni,
    MAX,
}

#[derive(Debug, Clone, Copy, Default, AnyBitPattern)]
#[repr(C)]
pub struct FilterConfig {
    pub pid_filter: Option<Filter>,
    pub comm_filter: Option<Filter>,
    pub exe_path_filter: Option<Filter>,
    pub cmdline_filter: Option<Filter>,
}

#[derive(Debug, Clone, Copy, Default, CheckedBitPattern)]
#[repr(C)]
pub struct Filter {
    pub missing_behavior: MissingBehavior,
}

/// # Safety
///
/// MissingBehavior starts at 1 so it is invalid to have a value of 0
/// E.g. None = 0, Some(Match) = 1, Some(NotMatch) = 2
unsafe impl PodInOption for Filter {}
unsafe impl ZeroableInOption for Filter {}

#[derive(Debug, Clone, Copy, CheckedBitPattern, Default)]
#[repr(u8)]
pub enum MissingBehavior {
    #[default]
    Match = 1,
    NotMatch,
}

/// Each bit corresponds to an EventKind, e.g. 1 << EventKind::Write
pub struct Equality {
    /// 1 corresponds to Eq, 0 corresponds to NotEq
    pub eq_for_event_kind: u64,
    /// 1 corresponds to the key being used for the event kind, 0 corresponds to not being used
    pub used_for_event_kind: u64,
}
