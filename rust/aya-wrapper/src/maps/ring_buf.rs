use std::{borrow::{Borrow, BorrowMut}, fmt::Debug, ops::Deref, os::fd::{AsRawFd, RawFd}};

use aya::maps::{ring_buf::RingBufItem, MapData, MapError};

use super::{MapConverterMut, MapConverterOwned, MapConverterRef, MapMut, MapOwned, MapRef};



// Trait defs

pub trait RingBufRef: AsRawFd {
    type Item<'a>: Deref<Target = [u8]> + Debug where Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
pub trait RingBufMut: RingBufRef {}
pub trait RingBufOwned: RingBufMut {}

// Wrapper struct
#[derive(Debug)]
pub struct RingBuf<I>(I);

impl<I> RingBuf<I> {
    pub fn new(inner: I) -> Self {
        Self(inner)
    }
}

impl<I: RingBufRef> AsRawFd for RingBuf<I> {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
impl<I: RingBufRef> RingBufRef for RingBuf<I> {
    type Item<'a> = I::Item<'a> where I: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.0.next()
    }
}
impl<I: RingBufMut> RingBufMut for RingBuf<I> {}
impl<I: RingBufOwned> RingBufOwned for RingBuf<I> {}


impl<M: MapConverterRef> TryFrom<MapRef<M>> for RingBuf<M::RingBufRef> {
    type Error = MapError;
    fn try_from(value: MapRef<M>) -> Result<Self, Self::Error> {
        value.0.try_into().map(Self::new)
    }
}
impl<M: MapConverterMut> TryFrom<MapMut<M>> for RingBuf<M::RingBufMut> {
    type Error = MapError;
    fn try_from(value: MapMut<M>) -> Result<Self, Self::Error> {
        value.0.try_into().map(Self::new)
    }
}
impl<M: MapConverterOwned> TryFrom<MapOwned<M>> for RingBuf<M::RingBufOwned> {
    type Error = MapError;
    fn try_from(value: MapOwned<M>) -> Result<Self, Self::Error> {
        value.0.try_into().map(Self::new)
    }
}




// Aya Impl

impl<T: Borrow<MapData>> RingBufRef for aya::maps::RingBuf<T> {
    type Item<'a> = RingBufItem<'a> where T: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.next()
    }
}

impl<T: BorrowMut<MapData>> RingBufMut for aya::maps::RingBuf<T> {}
impl RingBufOwned for aya::maps::RingBuf<MapData> {}


#[cfg(feature = "mocks")]
pub use mocks::{MockRingBuf, MockRingBufItem, __mock_MockRingBuf_TryFrom_5153577446266629640::__try_from::Context as TryFromContext};

#[cfg(feature = "mocks")]
mod mocks {
    use std::{ops::Deref, os::fd::{AsRawFd, RawFd}};
    use aya::maps::MapError;

    use crate::maps::MockMapConverter;
    use super::{RingBufMut, RingBufOwned, RingBufRef};

    mockall::mock! {
        #[derive(Debug)]
        pub RingBufItem {}
        impl Deref for RingBufItem {
            type Target = [u8];
            fn deref(&self) -> &[u8];
        }
    }

    mockall::mock! {
        #[derive(Debug)]
        pub RingBuf {}
        impl RingBufRef for RingBuf {
            type Item<'a> = MockRingBufItem;
            fn next(&mut self) -> Option<MockRingBufItem>;
        }
        impl RingBufMut for RingBuf {}
        impl RingBufOwned for RingBuf {}
        impl AsRawFd for RingBuf {
            fn as_raw_fd(&self) -> RawFd;
        }
        
        impl TryFrom<MockMapConverter> for RingBuf {
            type Error = MapError;
            fn try_from(value: MockMapConverter) -> Result<Self, MapError>;
        }
    }
}