use std::{ops::{Deref, DerefMut}, os::fd::{AsRawFd, RawFd}, sync::Arc};

use crossbeam::atomic::AtomicCell;
use thiserror::Error;

use super::{OwnedRingBuf, typed_ringbuf::TypedRingBuffer};

struct SingleOwner<T>(AtomicCell<Option<Box<T>>>);

pub struct RegistryItem<T>(Arc<SingleOwner<T>>);



impl<T> Clone for RegistryItem<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> From<T> for SingleOwner<T> {
    fn from(value: T) -> Self {
        Self(AtomicCell::new(Some(Box::new(value))))
    }
}

impl<T> From<T> for RegistryItem<T> {
    fn from(value: T) -> Self {
        Self(Arc::new(value.into()))
    }
}


pub struct RegistryGuard<T> {
    inner: Option<Box<T>>,
    _registry_entry: RegistryItem<T>,
}
impl<T> From<OwnedRingBuf> for RegistryItem<TypedRingBuffer<T>> {
    fn from(value: OwnedRingBuf) -> Self {
        TypedRingBuffer::from(value).into()
    }
}
#[derive(Error, Debug)]
pub enum TakeError {
    #[error("the item is already taken")]
    AlreadyTaken,
}

impl<T> RegistryItem<T> {
    pub fn try_take(&self) -> Result<RegistryGuard<T>, TakeError> {
        if let Some(value) = self.0.0.take() {
            Ok(RegistryGuard { inner: Some(value), _registry_entry: self.clone() })
        } else {
            Err(TakeError::AlreadyTaken)
        }
    }
    pub fn take(&self) -> RegistryGuard<T> {
        self.try_take().expect("not taken")
    }
}

impl<T> Deref for RegistryGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("not dropped")
    }
}

impl<T> DerefMut for RegistryGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("not dropped")
    }
}

impl<T> Drop for RegistryGuard<T> {
    fn drop(&mut self) {
        self._registry_entry.0.0.store(self.inner.take());
    }
}

impl<T: AsRawFd> AsRawFd for RegistryGuard<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_ref().expect("not dropped").as_raw_fd()
    }
}
