use std::{io, path::Path};

use aya::{maps::{HashMap, Map, MapData, MapError, RingBuf}, pin::PinError, programs::{KProbe, Program, ProgramError, TracePoint, UProbe}, Ebpf, EbpfError, Pod};

use super::{OwnedHashMap, OwnedRingBuf};

pub trait EbpfLoad {
    fn load(&mut self) -> Result<(), ProgramError>;
}

pub trait EbpfPin {
    fn pin(&mut self, path: &str) -> Result<(), PinError>;
}

pub trait TryMapFromPin {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError> where Self: Sized;
}


impl EbpfLoad for KProbe {
    fn load(&mut self) -> Result<(), ProgramError> {
        KProbe::load(self)
    }
}

impl EbpfPin for KProbe {
    fn pin(&mut self, path: &str) -> Result<(), PinError> {
        KProbe::pin(self, path)
    }
}

impl EbpfLoad for UProbe {
    fn load(&mut self) -> Result<(), ProgramError> {
        UProbe::load(self)
    }
}

impl EbpfPin for UProbe {
    fn pin(&mut self, path: &str) -> Result<(), PinError> {
        UProbe::pin(self, path)
    }
}

impl EbpfLoad for TracePoint {
    fn load(&mut self) -> Result<(), ProgramError> {
        TracePoint::load(self)
    }
}

impl EbpfPin for TracePoint {
    fn pin(&mut self, path: &str) -> Result<(), PinError> {
        TracePoint::pin(self, path)
    }
}

pub trait LoadAndPin {
    fn load_and_pin<E>(&mut self, name: &str, base: &str) -> Result<(), EbpfError> 
        where E: EbpfLoad + EbpfPin,
         for<'a> &'a mut E: TryFrom<&'a mut Program, Error = ProgramError>;
}

impl LoadAndPin for Ebpf {
    fn load_and_pin<E>(&mut self, name: &str, base: &str) -> Result<(), EbpfError> where E: EbpfLoad + EbpfPin,
         for<'a> &'a mut E: TryFrom<&'a mut Program, Error = ProgramError>
     {
        let program = self.program_mut(name).ok_or(EbpfError::ProgramError(ProgramError::InvalidName { name: name.to_string() }))?;
        let inner: &mut E = program.try_into()?;
        inner.load()?;
        let full_path = format!("{base}/{name}");
        inner.pin(&full_path).map_err(|e| ProgramError::IOError(io::Error::other(e)))?;
        
        Ok(())
    }
}

impl TryMapFromPin for OwnedRingBuf {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError> where Self: Sized {
        RingBuf::try_from(Map::RingBuf(MapData::from_pin(path)?))
    }
}

impl<K: Pod, V: Pod> TryMapFromPin for OwnedHashMap<K, V> {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError> where Self: Sized {
        HashMap::<_, K, V>::try_from(Map::HashMap(MapData::from_pin(path)?))
    }
}
