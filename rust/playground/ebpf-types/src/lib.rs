#![no_std]

use bytemuck::AnyBitPattern;

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
