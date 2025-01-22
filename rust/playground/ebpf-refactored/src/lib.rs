#![cfg_attr(not(test), no_std)]

pub mod relocation_helpers;

#[cfg(target_arch = "bpf")]
pub mod programs;

#[cfg(target_arch = "bpf")]
pub mod task_info;
