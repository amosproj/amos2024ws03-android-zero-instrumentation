use core::mem::MaybeUninit;

use aya_ebpf::{
    bindings::{BPF_EXIST, BPF_NOEXIST},
    maps::LruHashMap,
};

use crate::scratch::{ScratchSpace, TryIntoMem};

#[repr(transparent)]
pub struct Cache<K: 'static, V: 'static>(&'static LruHashMap<K, V>);

impl<K: 'static, V: 'static> Cache<K, V> {
    /// # Safety
    ///
    /// The assumption of this map is that
    /// 1. The data is written atomically, this is ensured
    ///    by only giving out a readable reference to the data
    ///    add updating through ebpf helpers.
    /// 2. All readers really only care about the newest data
    ///    so if the data is overwritten while holding a reference
    ///    to it must be fine.
    ///
    /// One problem that can always happen with maps is that one
    /// reads a value for a key and a reference is returned. Before
    /// copying the value at the address, the value could have changed
    /// by an update to the map. So we can never be sure that we
    /// actually get the value for a key.
    #[inline(always)]
    pub const unsafe fn new(map: &'static LruHashMap<K, V>) -> Self {
        Self(map)
    }

    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<&'static V> {
        // # SAFETY
        //
        // This map only allows writing initialized values, so retrieving a value
        // from the map should always give a reference to an initialized value.
        // The ebpf verifier ensures that references always point to valid memory
        // so the pointer will also never be dangling.
        //
        // The worst case is that while holding the reference, the value is
        // overwritten, but still in this case, the value is defined
        // from the perspective of Rust, it might just be another value.
        //
        // There is no real way to prevent this other than always copying the
        // value out of hashmap, but still then, in the worst case the value
        // might be overwritten in between getting the pointer and reading
        // the data.
        unsafe { self.0.get(key) }
    }

    #[inline(always)]
    pub fn insert(&self, key: &K, value: &V) -> Result<(), i64> {
        self.0.insert(key, value, BPF_NOEXIST as u64)
    }

    #[inline(always)]
    pub fn overwrite(&self, key: &K, value: &V) -> Result<(), i64> {
        self.0.insert(key, value, BPF_EXIST as u64)
    }

    #[inline(always)]
    pub fn update(&self, key: &K, value: &V) -> Result<(), i64> {
        self.0.insert(key, value, 0)
    }

    #[inline(always)]
    pub fn delete(&self, key: &K) -> Result<(), i64> {
        self.0.remove(key)
    }

    #[inline(always)]
    pub fn get_or_insert<T, F>(
        &self,
        key: &K,
        scratch: &ScratchSpace<T>,
        init: F,
    ) -> Result<&'static V, i64>
    where
        F: FnOnce(&mut MaybeUninit<V>) -> Result<&V, i64>,
    {
        if let Some(value) = self.get(key) {
            return Ok(value);
        }

        let mut scratch = scratch.cast().get().ok_or(-1)?;
        let value = init(&mut scratch)?;
        self.insert(key, value)?;

        self.get(key).ok_or(-1)
    }
}

pub trait TryWithCache<K, V>: TryIntoMem<V> {
    fn get_key(&self) -> Result<K, i64>;

    #[inline(always)]
    fn with_cache(
        &self,
        cache: &Cache<K, V>,
        scratch: &ScratchSpace<V>,
    ) -> Result<&'static V, i64> {
        cache.get_or_insert(
            &self.get_key()?,
            scratch,
            #[inline(always)]
            |mem| Ok(&*self.convert_into_mem(mem)?),
        )
    }
}
