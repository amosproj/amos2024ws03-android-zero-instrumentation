
use aya::programs::ProgramError;

pub mod xdp;

#[cfg(feature = "mocks")]
pub use xdp::{MockProgramFd, MockXdp, MockXdpLinkId};

#[cfg(feature = "mocks")]
pub use mocks::MockProgramConverter;

pub use xdp::{XdpMut, XdpRef, Xdp, XdpOwned};


pub trait ProgramConverterRef {
    type XdpRef: XdpRef + TryFrom<Self, Error = ProgramError> where Self: Sized;
}

pub trait ProgramConverterMut {
    type XdpMut: XdpMut + TryFrom<Self, Error = ProgramError> where Self: Sized;
}

pub struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}


pub struct ProgramRef<P: ProgramConverterRef>(P);
pub struct ProgramMut<P: ProgramConverterMut>(P);

impl<P: ProgramConverterRef> ProgramRef<P> {
    pub fn new(p: P) -> Self {
        Self(p)
    }
}
impl<P: ProgramConverterMut> ProgramMut<P> {
    pub fn new(p: P) -> Self {
        Self(p)
    }
}

// aya impls

impl<'a> ProgramConverterRef for &'a aya::programs::Program {
    type XdpRef = Wrapper<&'a aya::programs::Xdp>;
}
impl<'a> ProgramConverterMut for &'a mut aya::programs::Program {
    type XdpMut = Wrapper<&'a mut aya::programs::Xdp>;
}

#[cfg(feature = "mocks")]
mod mocks {
    use super::{MockXdp, ProgramConverterMut, ProgramConverterRef};

    mockall::mock! {
        pub ProgramConverter {}
        
        impl ProgramConverterRef for ProgramConverter {
            type XdpRef = MockXdp;
        }
        
        impl ProgramConverterMut for ProgramConverter {
            type XdpMut = MockXdp;
        }
    }
}