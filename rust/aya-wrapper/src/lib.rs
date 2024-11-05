pub mod maps;
pub mod programs;


use std::path::Path;

use aya::EbpfError;

use maps::{MapConverterMut, MapConverterOwned, MapConverterRef, MapMut, MapOwned, MapRef};
use programs::{ProgramConverterMut, ProgramConverterRef, ProgramMut, ProgramRef};

pub use aya;

#[cfg(feature = "mocks")]
pub use mocks::MockEbpf;

#[cfg(feature = "mocks")]
pub use mockall;

pub trait Ebpf: Sized {
    type MapConverterRef<'a>: MapConverterRef where Self: 'a;
    type MapConverterMut<'a>: MapConverterMut where Self: 'a;
    type MapConverterOwned: MapConverterOwned;
    type ProgramConverterRef<'a>: ProgramConverterRef where Self: 'a;
    type ProgramConverterMut<'a>: ProgramConverterMut where Self: 'a;

    fn load_file<P: AsRef<Path> + 'static>(path: P) -> Result<Self, EbpfError>;
    fn load(data: &[u8]) -> Result<Self, EbpfError>;
    fn map(&self, name: &str) -> Option<MapRef<Self::MapConverterRef<'_>>>;
    fn map_mut(&mut self, name: &str) -> Option<MapMut<Self::MapConverterMut<'_>>>;
    fn take_map(&mut self, name: &str) -> Option<MapOwned<Self::MapConverterOwned>>;
    fn maps(&self) -> impl Iterator<Item = (String, MapRef<Self::MapConverterRef<'_>>)>;
    fn maps_mut(&mut self) -> impl Iterator<Item = (String, MapMut<Self::MapConverterMut<'_>>)>;
    fn program(&self, name: &str) -> Option<ProgramRef<Self::ProgramConverterRef<'_>>>;
    fn program_mut(&mut self, name: &str) -> Option<ProgramMut<Self::ProgramConverterMut<'_>>>;
    fn programs(&self) -> impl Iterator<Item = (String, ProgramRef<Self::ProgramConverterRef<'_>>)>;
    fn programs_mut(&mut self) -> impl Iterator<Item = (String, ProgramMut<Self::ProgramConverterMut<'_>>)>;
}


impl Ebpf for aya::Ebpf {
    type MapConverterRef<'a> = &'a aya::maps::Map;
    type MapConverterMut<'a> = &'a mut aya::maps::Map;
    type MapConverterOwned = aya::maps::Map;
    type ProgramConverterRef<'a> = &'a aya::programs::Program;
    type ProgramConverterMut<'a> = &'a mut aya::programs::Program;
    
    fn load_file<P: AsRef<Path> + 'static>(path: P) -> Result<Self, EbpfError> {
        Self::load_file(path)
    }
    
    fn load(data: &[u8]) -> Result<Self, EbpfError> {
        Self::load(data)
    }
    
    fn map(&self, name: &str) -> Option<MapRef<Self::MapConverterRef<'_>>> {
        self.map(name).map(MapRef::new)
    }
    
    fn map_mut(&mut self, name: &str) -> Option<MapMut<Self::MapConverterMut<'_>>> {
        self.map_mut(name).map(MapMut::new)
    }
    
    fn take_map(&mut self, name: &str) -> Option<MapOwned<Self::MapConverterOwned>> {
        self.take_map(name).map(MapOwned::new)
    }
    
    fn maps(&self) -> impl Iterator<Item = (String, MapRef<Self::MapConverterRef<'_>>)> {
        self.maps().map(|(k, v)| (k.to_owned(), MapRef::new(v)))
    }
    
    fn maps_mut(&mut self) -> impl Iterator<Item = (String, MapMut<Self::MapConverterMut<'_>>)> {
        self.maps_mut().map(|(k, v)| (k.to_owned(), MapMut::new(v)))
    }
    
    fn program(&self, name: &str) -> Option<ProgramRef<&aya::programs::Program>> {
        self.program(name).map(ProgramRef::new)
    }
    fn program_mut(&mut self, name: &str) -> Option<ProgramMut<&mut aya::programs::Program>> {
        self.program_mut(name).map(ProgramMut::new)
    }
    fn programs(&self) -> impl Iterator<Item = (String, ProgramRef<&aya::programs::Program>)> {
        self.programs().map(|(k, v)| (k.to_owned(), ProgramRef::new(v)))
    }
    fn programs_mut(&mut self) -> impl Iterator<Item = (String, ProgramMut<&mut aya::programs::Program>)> {
        self.programs_mut().map(|(k, v)| (k.to_owned(), ProgramMut::new(v)))
    }
}

#[cfg(feature = "mocks")]
mod mocks {
    use std::path::Path;

    use aya::EbpfError;

    use crate::{maps::{MapMut, MapOwned, MapRef, MockMapConverter}, programs::{MockProgramConverter, ProgramMut, ProgramRef}, Ebpf};

    mockall::mock! {
        pub Ebpf {}
        
        impl Ebpf for Ebpf {
            type MapConverterRef<'a> = MockMapConverter;
            type MapConverterMut<'a> = MockMapConverter;
            type MapConverterOwned = MockMapConverter;
            type ProgramConverterRef<'a> = MockProgramConverter;
            type ProgramConverterMut<'a> = MockProgramConverter;
            
            fn load_file<P: AsRef<Path> + 'static>(path: P) -> Result<Self, EbpfError>;
            fn load(data: &[u8]) -> Result<Self, EbpfError>;
            fn map(&self, name: &str) -> Option<MapRef<MockMapConverter>>;
            fn map_mut(&mut self, name: &str) -> Option<MapMut<MockMapConverter>>;
            fn take_map(&mut self, name: &str) -> Option<MapOwned<MockMapConverter>>;
            fn maps(&self) -> impl Iterator<Item = (String, MapRef<MockMapConverter>)>;
            fn maps_mut(&mut self) -> impl Iterator<Item = (String, MapMut<MockMapConverter>)>;
            fn program(&self, name: &str) -> Option<ProgramRef<MockProgramConverter>>;
            fn program_mut(&mut self, name: &str) -> Option<ProgramMut<MockProgramConverter>>;
            fn programs(&self) -> impl Iterator<Item = (String, ProgramRef<MockProgramConverter>)>;
            fn programs_mut(&mut self) -> impl Iterator<Item = (String, ProgramMut<MockProgramConverter>)>;
        }
    }
}