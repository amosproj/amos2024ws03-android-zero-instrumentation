use std::borrow::{Borrow, BorrowMut};

use aya::programs::{ProgramError, ProgramFd, XdpFlags};

use super::{ProgramConverterRef, ProgramConverterMut, ProgramMut, ProgramRef, Wrapper};

// Trait Defs

pub trait XdpRef {
    type FdRef<'a> where Self: 'a; 
    fn fd(&self) -> Result<Self::FdRef<'_>, ProgramError>;
}

pub trait XdpMut: XdpRef {
    type LinkId;
    fn load(&mut self) -> Result<(), ProgramError>;
    fn attach(&mut self, interface: &str, flags: XdpFlags) -> Result<Self::LinkId, ProgramError>;
}
pub trait XdpOwned: XdpMut {}

// Wrapper struct

pub struct Xdp<I>(I);

impl<I> Xdp<I> {
    pub fn new(inner: I) -> Self {
        Self(inner)
    }
}

impl<I: XdpRef> XdpRef for Xdp<I> {
    type FdRef<'a> = I::FdRef<'a> where Self: 'a ;
    fn fd(&self) -> Result<Self::FdRef<'_>, ProgramError> {
        self.0.fd()
    }
}

impl<I: XdpMut> XdpMut for Xdp<I> {
    type LinkId = I::LinkId;
    fn load(&mut self) -> Result<(), ProgramError> {
        self.0.load()
    }
    fn attach(&mut self, interface: &str, flags: XdpFlags) -> Result<Self::LinkId, ProgramError> {
        self.0.attach(interface, flags)
    }
}

impl<I: XdpOwned> XdpOwned for Xdp<I> {}

impl<P: ProgramConverterRef> TryFrom<ProgramRef<P>> for Xdp<P::XdpRef> {
    type Error = ProgramError;
    fn try_from(value: ProgramRef<P>) -> Result<Self, Self::Error> {
        value.0.try_into().map(Self::new)
    }
}
impl<P: ProgramConverterMut> TryFrom<ProgramMut<P>> for Xdp<P::XdpMut> {
    type Error = ProgramError;
    fn try_from(value: ProgramMut<P>) -> Result<Self, Self::Error> {
        value.0.try_into().map(Self::new)
    }
}


// Aya Impl

impl XdpRef for aya::programs::Xdp {
    type FdRef<'a> = &'a ProgramFd;
    fn fd(&self) -> Result<&ProgramFd, ProgramError> {
        self.fd()
    }
}
impl XdpMut for aya::programs::Xdp {
    type LinkId = aya::programs::xdp::XdpLinkId;
    fn load(&mut self) -> Result<(), ProgramError> {
        self.load()
    }
    fn attach(&mut self, interface: &str, flags: XdpFlags) -> Result<Self::LinkId, ProgramError> {
        self.attach(interface, flags)
    }
}
impl XdpOwned for aya::programs::Xdp {}

impl<T: Borrow<aya::programs::Xdp>> XdpRef for Wrapper<T> {
    type FdRef<'a> = &'a ProgramFd where T: 'a;
    fn fd(&self) -> Result<&ProgramFd, ProgramError> {
        self.0.borrow().fd()
    }
}
impl<T: BorrowMut<aya::programs::Xdp>> XdpMut for Wrapper<T> {
    type LinkId = aya::programs::xdp::XdpLinkId;
    fn load(&mut self) -> Result<(), ProgramError> {
        self.0.borrow_mut().load()
    }
    fn attach(&mut self, interface: &str, flags: XdpFlags) -> Result<Self::LinkId, ProgramError> {
        self.0.borrow_mut().attach(interface, flags)
    }
}
impl XdpOwned for Wrapper<aya::programs::Xdp> {}

impl<'a> TryFrom<&'a aya::programs::Program> for Wrapper<&'a aya::programs::Xdp> {
    type Error = ProgramError;
    fn try_from(value: &'a aya::programs::Program) -> Result<Self, Self::Error> {
        value.try_into().map(Self::new)
    }
}
impl<'a> TryFrom<&'a mut aya::programs::Program> for Wrapper<&'a mut aya::programs::Xdp> {
    type Error = ProgramError;
    fn try_from(value: &'a mut aya::programs::Program) -> Result<Self, Self::Error> {
        value.try_into().map(Self::new)
    }
}

#[cfg(feature = "mocks")]
pub use mocks::{MockProgramFd, MockXdpLinkId, MockXdp, __mock_MockXdp_TryFrom_698325346282305646::__try_from::Context as TryFromContext };

#[cfg(feature = "mocks")]
mod mocks {
    use aya::programs::{ProgramError, XdpFlags};
    
    use crate::programs::MockProgramConverter;

    use super::{XdpMut, XdpOwned, XdpRef};

    mockall::mock! {
        pub ProgramFd {}
    }

    mockall::mock! {
        pub XdpLinkId {}
    }

    mockall::mock! {
        pub Xdp {}
        
        impl XdpRef for Xdp {
            type FdRef<'a> = MockProgramFd; 
            fn fd(&self) -> Result<MockProgramFd, ProgramError>;
        }
        
        impl XdpMut for Xdp {
            type LinkId = MockXdpLinkId;
            fn load(&mut self) -> Result<(), ProgramError>;
            fn attach(&mut self, interface: &str, flags: XdpFlags) -> Result<MockXdpLinkId, ProgramError>;
        }
        
        impl XdpOwned for Xdp {}
        

        impl TryFrom<MockProgramConverter> for Xdp {
            type Error = ProgramError;
            fn try_from(value: MockProgramConverter) -> Result<Self, ProgramError>;
        }
    }
}

