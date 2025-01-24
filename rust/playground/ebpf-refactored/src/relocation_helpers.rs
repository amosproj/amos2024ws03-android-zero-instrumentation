// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT
#![allow(clippy::len_without_is_empty)]

#[cfg(test)]
fn bpf_probe_read_kernel<T>(ptr: *const T) -> Result<T, i64> {
    unsafe { Ok(core::ptr::read(ptr)) }
}

#[cfg(not(test))]
use aya_ebpf::helpers::bpf_probe_read_kernel;

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    #[repr(C, packed)]
    pub struct qstr {
        pub hash: u32,
        pub len: u32,
        pub name: *const u8,
    }

    include!(concat!(env!("OUT_DIR"), "/relocation_helpers.rs"));
}

use ffi::*;

macro_rules! gen_accessor {
    (plain: $parent:ident => $name:ident, $type:ty) => {
        gen_accessor_plain!($parent => $name, $type);
    };
    (wrapper: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_wrapper!($parent => $name, [< $type >]);
        }
    };
    (no_read: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_no_read!($parent => $name, $type);
        }
    };
    (no_read_wrapped: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_no_read_wrapped!($parent => $name, [< $type >]);
        }
    };
    (no_read_container: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_no_read_container!($parent => $name, [< $type >]);
        }
    };
}

#[cfg(test)]
macro_rules! gen_accessor_raw_test {
    ($parent_type:ident => $field_name:ident, $field_type:ty) => {
        paste::paste! {
            #[test]
            fn [< $parent_type _ $field_name _raw >]() {
                let mut parent = unsafe { core::mem::zeroed::<$parent_type>() };
                let expected = &raw mut parent.$field_name;

                let actual = unsafe { [< $parent_type _ $field_name >](&raw mut  parent) };
                assert_eq!(actual, expected);
            }
        }
    };
    (plain: $parent:ident => $name:ident, $type:ty) => {
        gen_accessor_raw_test!($parent => $name, $type);
    };
    (wrapper: $parent:ident => $name:ident, $type:ty) => {
        gen_accessor_raw_test!($parent => $name, $type);
    };
    (no_read: $parent:ident => $name:ident, $type:ty) => {
        gen_accessor_raw_test!($parent => $name, $type);
    };
    (no_read_wrapped: $parent:ident => $name:ident, $type:ty) => {
        gen_accessor_raw_test!($parent => $name, $type);
    };
    (no_read_container: $parent:ident => $name:ident, $type:ty) => {};
}

#[cfg(test)]
macro_rules! gen_accessor_test {
    (plain: $parent:ident => $name:ident, $type:ty) => {
        gen_accessor_plain_test!($parent => $name, $type);
    };
    (wrapper: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_wrapper_test!($parent => $name, [< $type >]);
        }
    };
    (no_read: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_no_read_test!($parent => $name, $type);
        }
    };
    (no_read_wrapped: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_no_read_wrapped_test!($parent => $name, $type);
        }
    };
    (no_read_container: $parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            gen_accessor_no_read_container_test!($parent => $name, [< $type >]);
        }
    };
}

macro_rules! gen_accessors {
    ($parent:ident => { $($variant:ident $name:ident: $type:ty),* $(,)? }) => {
        paste::paste! {
            #[doc = "Represents `*mut " $parent "` with CO:RE relocations."]
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct [< $parent:camel >] {
                inner: *mut $parent,
            }

            impl [< $parent:camel >] {
                $(
                    gen_accessor!($variant: $parent => $name, $type);
                )*
            }

            #[cfg(test)]
            mod [< test_ $parent >] {
                use super::*;

                $(
                    gen_accessor_test!($variant: $parent => $name, $type);
                    gen_accessor_raw_test!($variant: $parent => $name, $type);
                )*
            }
        }
    };
}

macro_rules! gen_accessor_plain {
    ($parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            #[doc = "Reads the value of the field `" $parent "." $name "` with CO:RE relocations."]
            #[inline(always)]
            pub fn $name(&self) -> Result<$type, i64> {
                unsafe { bpf_probe_read_kernel([< $parent _ $name >](self.inner) as *const _) }
            }
        }
    };
}

#[cfg(test)]
macro_rules! gen_accessor_plain_test {
    ($parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            #[test]
            fn $name() {
                let mut raw_parent = unsafe { core::mem::zeroed::<$parent>() };
                unsafe { core::ptr::write_bytes(&raw mut raw_parent.$name, 1, 1) };
                let parent = [< $parent:camel >] { inner: &raw mut raw_parent };
                let actual = parent.$name().unwrap();
                let actual_as_slice = unsafe { core::slice::from_raw_parts(&raw const actual as *const u8, core::mem::size_of::<$type>()) };
                let expected_as_slice = unsafe { core::slice::from_raw_parts(&raw const raw_parent.$name as *const u8, core::mem::size_of::<$type>()) };
                assert_eq!(actual_as_slice, expected_as_slice);
            }
        }
    };
}

macro_rules! gen_accessor_wrapper {
    ($parent:ident => $name:ident, $type:ident) => {
        paste::paste! {
            #[doc = "Reads the value of the field `" $parent "." $name "` with CO:RE relocations."]
            #[inline(always)]
            pub fn $name(&self) -> Result<$type, i64> {
                Ok($type { inner: unsafe { bpf_probe_read_kernel([< $parent _ $name >](self.inner) as *const _) }? })
            }
        }
    };
}

#[cfg(test)]
macro_rules! gen_accessor_wrapper_test {
    ($parent:ident => $name:ident, $type:ident) => {
        paste::paste! {
            #[test]
            fn $name() {
                let mut raw_parent = unsafe { core::mem::zeroed::<$parent>() };
                unsafe { core::ptr::write_bytes(&raw mut raw_parent.$name, 1, 1) };
                let parent = [< $parent:camel >] { inner: &raw mut raw_parent };
                let actual = parent.$name().unwrap().inner;
                let actual_as_slice = unsafe { core::slice::from_raw_parts(&raw const actual as *const u8, core::mem::size_of::<$type>()) };
                let expected_as_slice = unsafe { core::slice::from_raw_parts(&raw const raw_parent.$name as *const u8, core::mem::size_of::<$type>()) };
                assert_eq!(actual_as_slice, expected_as_slice);
            }
        }
    };
}

macro_rules! gen_accessor_no_read_wrapped {
    ($parent:ident => $name:ident, $type:ident) => {
        paste::paste! {
            #[doc = "Reads the value of the field `" $parent "." $name "` with CO:RE relocations."]
            #[inline(always)]
            pub fn $name(&self) -> $type {
                unsafe { $type { inner: [< $parent _ $name >](self.inner) } }
            }
        }
    };
}

macro_rules! gen_accessor_no_read {
    ($parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            #[doc = "Reads the value of the field `" $parent "." $name "` with CO:RE relocations."]
            #[inline(always)]
            pub fn $name(&self) -> $type {
                unsafe { [< $parent _ $name >](self.inner) }
            }
        }
    };
}

macro_rules! gen_accessor_no_read_container {
    ($parent:ident => $name:ident, $type:ident) => {
        paste::paste! {
            #[inline(always)]
            pub fn container(&self) -> $type {
                unsafe { $type { inner: [< $parent _ container >](self.inner) } }
            }
        }
    };
}

#[cfg(test)]
macro_rules! gen_accessor_no_read_test {
    ($parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            #[test]
            fn $name() {
                let mut raw_parent = unsafe { core::mem::zeroed::<$parent>() };
                unsafe { core::ptr::write_bytes(&raw mut raw_parent.$name, 1, 1) };
                let parent = [< $parent:camel >] { inner: &raw mut raw_parent };
                let actual = parent.$name();
                let actual_as_slice = unsafe { core::slice::from_raw_parts(actual as *const u8, core::mem::size_of::<$type>()) };
                let expected_as_slice = unsafe { core::slice::from_raw_parts(&raw const raw_parent.$name as *const u8, core::mem::size_of::<$type>()) };
                assert_eq!(actual_as_slice, expected_as_slice);
            }
        }
    };
}

#[cfg(test)]
macro_rules! gen_accessor_no_read_wrapped_test {
    ($parent:ident => $name:ident, $type:ty) => {
        paste::paste! {
            #[test]
            fn $name() {
                let mut raw_parent = unsafe { core::mem::zeroed::<$parent>() };
                unsafe { core::ptr::write_bytes(&raw mut raw_parent.$name, 1, 1) };
                let parent = [< $parent:camel >] { inner: &raw mut raw_parent };
                let actual = parent.$name().inner;
                let actual_as_slice = unsafe { core::slice::from_raw_parts(actual as *const u8, core::mem::size_of::<$type>()) };
                let expected_as_slice = unsafe { core::slice::from_raw_parts(&raw const raw_parent.$name as *const u8, core::mem::size_of::<$type>()) };
                assert_eq!(actual_as_slice, expected_as_slice);
            }
        }
    };
}

#[cfg(test)]
macro_rules! gen_accessor_no_read_container_test {
    ($parent:ident => $name:ident, $type:ident) => {
        paste::paste! {
            #[test]
            fn $name() {
                let mut raw_parent = unsafe { core::mem::zeroed() };
                let _ = [< $type:camel >] { inner: &raw mut raw_parent };
                let child = [< $parent:camel >] { inner: &raw mut raw_parent.$name };
                let actual = child.container().inner;
                assert_eq!(actual, &raw mut raw_parent);
            }
        }
    };
}

impl TaskStruct {
    /// # SAFETY
    ///
    /// Must point to a valid `task_struct` struct.
    pub unsafe fn new(inner: *mut aya_ebpf::bindings::task_struct) -> Self {
        Self {
            inner: inner as *mut _,
        }
    }
}

gen_accessors!(task_struct => {
    wrapper mm: MmStruct,
    plain pid: u32,
    plain tgid: u32,
    plain start_time: u64,
    wrapper real_parent: TaskStruct,
    wrapper group_leader: TaskStruct,
    no_read comm: *mut [i8; 16],
});

gen_accessors!(mm_struct => {
    plain arg_start: u64,
    plain arg_end: u64,
    wrapper exe_file: File,
});
gen_accessors!(file => {
    no_read_wrapped f_path: Path,
});
gen_accessors!(path => {
    wrapper dentry: Dentry,
    wrapper mnt: Vfsmount,
});
gen_accessors!(dentry => {
    no_read_wrapped d_name: Qstr,
    wrapper d_parent: Dentry,
});
gen_accessors!(vfsmount => {
    no_read_container mnt: Mount,
    wrapper mnt_root: Dentry,
});

gen_accessors!(qstr => {
    plain len: u32,
    plain name: *const u8,
});
gen_accessors!(mount => {
    wrapper mnt_parent: Mount,
    wrapper mnt_mountpoint: Dentry,
    no_read_wrapped mnt: Vfsmount,
});
