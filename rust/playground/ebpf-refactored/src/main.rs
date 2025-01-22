// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

mod relocation_helpers;
use relocation_helpers::magic_number;

#[cfg(all(not(target_arch = "bpf"), not(test)))]
#[no_mangle]
pub extern "C" fn main(argc: isize, _argv: *const *const u8) -> isize {
    magic_number(argc as i32) as isize
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}