use aya_ebpf::bindings::task_struct;

#[cfg(test)]
fn bpf_probe_read_kernel<T>(ptr: *const T) -> Result<T, i64> {
    unsafe { Ok(core::ptr::read(ptr)) }
}

#[cfg(test)]
fn bpf_probe_read_kernel_buf<T>(ptr: *const T, buf: &mut [u8]) -> Result<(), i64> {
    unsafe { core::ptr::copy_nonoverlapping(ptr, buf.as_mut_ptr() as *mut _, buf.len()) };
    Ok(())
}

#[cfg(test)]
fn bpf_probe_read_user_buf<T>(ptr: *const T, buf: &mut [u8]) -> Result<(), i64> {
    unsafe { core::ptr::copy_nonoverlapping(ptr, buf.as_mut_ptr() as *mut _, buf.len()) };
    Ok(())
}

#[cfg(not(test))]
pub use aya_ebpf::helpers::{
    bpf_probe_read_kernel, bpf_probe_read_kernel_buf, bpf_probe_read_user_buf,
};

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/relocation_helpers.rs"));
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct mm_struct {
    _unused: [u8; 0],
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_mm(task: *mut task_struct) -> Result<*mut mm_struct, i64> {
    bpf_probe_read_kernel(ffi::task_struct_mm(task as *mut _) as *const _)
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_pid(task: *mut task_struct) -> Result<i32, i64> {
    bpf_probe_read_kernel(ffi::task_struct_pid(task as *mut _))
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_tgid(task: *mut task_struct) -> Result<i32, i64> {
    bpf_probe_read_kernel(ffi::task_struct_tgid(task as *mut _))
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_start_time(task: *mut task_struct) -> Result<u64, i64> {
    bpf_probe_read_kernel(ffi::task_struct_start_time(task as *mut _))
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_comm(task: *mut task_struct, buf: &mut [u8; 16]) -> Result<(), i64> {
    bpf_probe_read_kernel_buf(ffi::task_struct_comm(task as *mut _) as *const u8, buf)
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_real_parent(task: *mut task_struct) -> Result<*mut task_struct, i64> {
    let task = bpf_probe_read_kernel(ffi::task_struct_real_parent(task as *mut _))?;
    Ok(task as *mut _)
}

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
#[inline(always)]
pub unsafe fn task_struct_group_leader(task: *mut task_struct) -> Result<*mut task_struct, i64> {
    let task = bpf_probe_read_kernel(ffi::task_struct_group_leader(task as *mut _))?;
    Ok(task as *mut _)
}

/// # SAFETY
///
/// Must point to a valid `mm_struct` struct.
#[inline(always)]
pub unsafe fn mm_struct_arg_start(mm: *mut mm_struct) -> Result<u64, i64> {
    bpf_probe_read_kernel(ffi::mm_struct_arg_start(mm as *mut _))
}

/// # SAFETY
///
/// Must point to a valid `mm_struct` struct.
#[inline(always)]
pub unsafe fn mm_struct_arg_end(mm: *mut mm_struct) -> Result<u64, i64> {
    bpf_probe_read_kernel(ffi::mm_struct_arg_end(mm as *mut _))
}

/// # SAFETY
///
/// `arg_start` and `arg_end` must point to valid memory.
#[inline(always)]
pub unsafe fn proc_cmdline(arg_start: u64, arg_end: u64, buf: &mut [u8; 256]) -> Result<(), i64> {
    let len = arg_end.saturating_sub(arg_start) as usize;
    let len = len & 255;
    let dst = &mut buf[..len];

    bpf_probe_read_user_buf(arg_start as *const u8, dst)
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use core::mem;

    use super::*;

    macro_rules! do_test {
        ($type:ident, $field:ident, $in:expr, $out:expr) => {
            paste::paste! {
                let mut value = unsafe { mem::zeroed::<ffi::$type>() };
                value.$field = $in;
                let $field = unsafe { [< $type _ $field >](&raw mut value as *mut $type).unwrap()  };
                assert_eq!($field, $out);
            }
        };
    }

    macro_rules! gen_tests {
        ($type:ident => $($field:ident: $in:expr, $out:expr),+) => {
            paste::paste! {
                $(
                    #[test]
                    fn [< test_ $type _ $field >]() {
                        do_test!($type, $field, $in, $out);
                    }
                )+
            }
        };
    }

    gen_tests! {task_struct =>
        mm: 1 as *mut ffi::mm_struct, 1 as *mut mm_struct,
        pid: 1, 1,
        tgid: 1, 1,
        start_time: 1, 1,
        real_parent: 1 as *mut ffi::task_struct, 1 as *mut task_struct,
        group_leader: 1 as *mut ffi::task_struct, 1 as *mut task_struct
    }

    #[test]
    fn test_task_struct_comm() {
        let expected = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut value = unsafe { mem::zeroed::<ffi::task_struct>() };
        value.comm = unsafe { *(&raw const expected as *const [i8; 16]) };
        let mut buf = [0; 16];
        unsafe { task_struct_comm(&raw mut value as *mut task_struct, &mut buf).unwrap() };
        assert_eq!(buf, expected);
    }

    gen_tests! {mm_struct =>
        arg_start: 1, 1,
        arg_end: 1, 1
    }

    #[test]
    fn test_proc_cmdline() {
        let (expected, len) = {
            let mut buf = [0; 256];
            let package_name = b"com.androidx.car";
            buf[..package_name.len()].copy_from_slice(package_name);
            (buf, package_name.len() as u64)
        };

        let arg_start = (&raw const expected[0]) as u64;
        let arg_end = arg_start + len;

        let mut buf = [0; 256];
        unsafe { proc_cmdline(arg_start, arg_end, &mut buf).unwrap() };

        assert_eq!(buf, expected);
    }
}
