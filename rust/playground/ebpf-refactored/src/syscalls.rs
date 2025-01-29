// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

#[cfg(bpf_target_arch = "aarch64")]
pub use syscall_numbers::aarch64::*;
#[cfg(bpf_target_arch = "x86_64")]
pub use syscall_numbers::x86_64::*;
