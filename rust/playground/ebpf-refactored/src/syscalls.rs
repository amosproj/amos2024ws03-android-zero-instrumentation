#[cfg(bpf_target_arch = "aarch64")]
pub use syscall_numbers::aarch64::*;
#[cfg(bpf_target_arch = "x86_64")]
pub use syscall_numbers::x86_64::*;
