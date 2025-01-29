// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use aya_ebpf::maps::PerCpuArray;

#[repr(transparent)]
pub struct ScratchSpace<T: 'static>(&'static PerCpuArray<(bool, T)>);

impl<T: 'static> ScratchSpace<T> {
    /// # Safety
    ///
    /// You must ensure that the map is only accessed throught this class
    /// for it to be safe.
    #[inline(always)]
    pub const unsafe fn new(map: &'static PerCpuArray<(bool, T)>) -> Self {
        Self(map)
    }

    #[inline(always)]
    pub fn cast<U: 'static>(&self) -> &ScratchSpace<U> {
        const {
            assert!(size_of::<T>() >= size_of::<U>());
        };
        unsafe { &*(self as *const _ as *const _) }
    }

    #[inline(always)]
    pub fn get(&self) -> Option<ScratchValue<T>> {
        let ptr = self.0.get_ptr_mut(0)?;

        unsafe {
            let borrowed = &mut (*ptr).0;
            let value = &mut *(&raw mut (*ptr).1 as *mut MaybeUninit<T>);

            if *borrowed {
                return None;
            }
            *borrowed = true;

            Some(ScratchValue { borrowed, value })
        }
    }
}

pub struct ScratchValue<T: 'static> {
    borrowed: &'static mut bool,
    value: &'static mut MaybeUninit<T>,
}

impl<T> Deref for ScratchValue<T> {
    type Target = MaybeUninit<T>;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for ScratchValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T> Drop for ScratchValue<T> {
    fn drop(&mut self) {
        *self.borrowed = false;
    }
}

pub trait TryIntoMem<T> {
    fn convert_into_mem<'a>(&self, mem: &'a mut MaybeUninit<T>) -> Result<&'a mut T, i64>;
}
