// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

pub use ebpf_refactored::*;

#[cfg(all(not(target_arch = "bpf"), not(test)))]
#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    0
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
