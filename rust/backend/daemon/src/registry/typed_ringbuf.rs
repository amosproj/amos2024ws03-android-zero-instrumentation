use std::{marker::PhantomData, os::fd::{AsRawFd, RawFd}};

use backend_common::TryFromRaw;

use super::OwnedRingBuf;

pub struct TypedRingBuffer<T> {
    inner: OwnedRingBuf,
    _phantom: PhantomData<T>,
}

impl<T> AsRawFd for TypedRingBuffer<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl<T: TryFromRaw> TypedRingBuffer<T> 
{
    pub fn next(&mut self) -> Option<T> {
        if let Some(data) = self.inner.next() {
            Some(T::try_from_raw(&*data).expect("wrong data type for ring buffer"))
        } else {
            None
        }
    }
}


impl<T> TypedRingBuffer<T> {
    pub fn new(inner: OwnedRingBuf) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<OwnedRingBuf> for TypedRingBuffer<T> {
    fn from(value: OwnedRingBuf) -> Self {
        Self::new(value)
    }
}