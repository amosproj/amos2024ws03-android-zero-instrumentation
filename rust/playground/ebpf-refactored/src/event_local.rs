// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::ops::Deref;

use aya_ebpf::{bindings::BPF_NOEXIST, maps::HashMap};
use ebpf_types::{EventData, EventKind};

#[repr(C)]
pub struct EventLocal<T: EventLocalData> {
    atomic_held: u64,
    pub data: T::Data,
}

pub struct EventLocalStorage<T: 'static + EventLocalData>(&'static HashMap<u64, EventLocal<T>>);

pub trait EventLocalData: EventData + 'static {
    type Data;
}

pub struct PlaceHolder;

impl EventData for PlaceHolder {
    const EVENT_KIND: EventKind = EventKind::MAX;
}

impl EventLocalData for PlaceHolder {
    type Data = [u8; 8192];
}

impl<T: EventLocalData> EventLocalStorage<T> {
    /// # Safety
    ///
    /// You must ensure that the map is only accessed throught this class
    /// for it to be safe.
    pub const unsafe fn new(map: &'static HashMap<u64, EventLocal<T>>) -> Self {
        Self(map)
    }

    #[inline(always)]
    pub fn cast<U: 'static + EventLocalData>(&self) -> &EventLocalStorage<U> {
        const {
            assert!(size_of::<T::Data>() >= size_of::<U::Data>());
        };
        unsafe { &*(self as *const _ as *const _) }
    }

    fn get_key<D: EventLocalData>(key: u32) -> u64 {
        ((D::EVENT_KIND as u64) << 32) | key as u64
    }

    pub fn set(&self, key: u32, data: &mut EventLocal<T>) -> Result<(), i64> {
        data.atomic_held = 0;

        let full_key = Self::get_key::<T>(key);
        if self.0.insert(&full_key, data, BPF_NOEXIST as u64).is_ok() {
            return Ok(());
        }

        let _ = self.take(key)?;

        self.0.insert(&full_key, data, BPF_NOEXIST as u64)
    }

    #[inline(always)]
    pub fn take(&self, key: u32) -> Result<EventLocalValue<T>, i64> {
        let key = Self::get_key::<T>(key);
        let ptr = self.0.get_ptr_mut(&key).ok_or(-1)?;
        let is_held =
            unsafe { core::intrinsics::atomic_xchg_seqcst(&raw mut (*ptr).atomic_held, 1) };
        if is_held != 0 {
            return Err(-1);
        }

        unsafe {
            Ok(EventLocalValue {
                key,
                map: self.0,
                value: &*ptr,
            })
        }
    }
}

pub struct EventLocalValue<T: EventLocalData + 'static> {
    key: u64,
    map: &'static HashMap<u64, EventLocal<T>>,
    value: &'static EventLocal<T>,
}

impl<T: EventLocalData + 'static> Deref for EventLocalValue<T> {
    type Target = EventLocal<T>;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: EventLocalData + 'static> Drop for EventLocalValue<T> {
    fn drop(&mut self) {
        let _ = self.map.remove(&self.key);
    }
}
