
use aya::maps::{MapData, MapError};


pub mod ring_buf;
pub use ring_buf::{RingBuf, RingBufRef, RingBufMut, RingBufOwned};

#[cfg(feature = "mocks")]
pub use ring_buf::{MockRingBuf, MockRingBufItem};

#[cfg(feature = "mocks")]
pub use mocks::MockMapConverter;



pub trait MapConverterRef {
    type RingBufRef: RingBufRef + TryFrom<Self, Error = MapError> where Self: Sized;
}

pub trait MapConverterMut {
    type RingBufMut: RingBufMut + TryFrom<Self, Error = MapError> where Self: Sized;
}

pub trait MapConverterOwned {
    type RingBufOwned: RingBufOwned + TryFrom<Self, Error = MapError> where Self: Sized;
}


pub struct MapRef<M: MapConverterRef>(M);
pub struct MapMut<M: MapConverterMut>(M);
pub struct MapOwned<M: MapConverterOwned>(M);

impl<M: MapConverterRef> MapRef<M> {
    pub fn new(p: M) -> Self {
        Self(p)
    }
}
impl<M: MapConverterMut> MapMut<M> {
    pub fn new(p: M) -> Self {
        Self(p)
    }
}
impl<M: MapConverterOwned> MapOwned<M> {
    pub fn new(p: M) -> Self {
        Self(p)
    }
}




// aya impls

impl<'a> MapConverterRef for &'a aya::maps::Map {
    type RingBufRef = aya::maps::RingBuf<&'a MapData>;
}
impl<'a> MapConverterMut for &'a mut aya::maps::Map {
    type RingBufMut = aya::maps::RingBuf<&'a mut MapData>;
}
impl MapConverterOwned for aya::maps::Map {
    type RingBufOwned = aya::maps::RingBuf<MapData>;
}


#[cfg(feature = "mocks")]
mod mocks {
    use super::{MapConverterMut, MapConverterOwned, MapConverterRef, MockRingBuf};

    mockall::mock! {
        pub MapConverter {}
        
        impl MapConverterRef for MapConverter {
            type RingBufRef = MockRingBuf;
        }
        
        impl MapConverterMut for MapConverter {
            type RingBufMut = MockRingBuf;
        }

        impl MapConverterOwned for MapConverter {
            type RingBufOwned = MockRingBuf;
        }
    }
}