mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/relocation_helpers.rs"));
}

#[inline(always)]
pub fn magic_number(x: i32) -> i32 {
    unsafe { ffi::magic_number(x) }
}
