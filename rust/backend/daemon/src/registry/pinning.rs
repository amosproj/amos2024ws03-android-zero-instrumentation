// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{io, path::Path};

use aya::{
    maps::{Array, HashMap, Map, MapData, MapError, RingBuf},
    pin::PinError,
    programs::{KProbe, Program, ProgramError, RawTracePoint, TracePoint, UProbe},
    Ebpf, EbpfError, Pod,
};

use super::{OwnedArray, OwnedHashMap, OwnedRingBuf};

pub trait EbpfLoad {
    fn load(&mut self) -> Result<(), ProgramError>;
}

pub trait EbpfPin {
    fn pin(&mut self, path: &str) -> Result<(), PinError>;
}

pub trait TryMapFromPin {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError>
    where
        Self: Sized;
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

impl EbpfLoad for RawTracePoint {
    fn load(&mut self) -> Result<(), ProgramError> {
        RawTracePoint::load(self)
    }
}

impl EbpfPin for RawTracePoint {
    fn pin(&mut self, path: &str) -> Result<(), PinError> {
        RawTracePoint::pin(self, path)
    }
}

impl EbpfPin for Map {
    fn pin(&mut self, path: &str) -> Result<(), PinError> {
        Map::pin(self, path)
    }
}

pub trait LoadAndPin {
    fn load_and_pin<E>(&mut self, name: &str, base: &str) -> Result<(), EbpfError>
    where
        E: EbpfLoad + EbpfPin,
        for<'a> &'a mut E: TryFrom<&'a mut Program, Error = ProgramError>;
}

impl LoadAndPin for Ebpf {
    fn load_and_pin<E>(&mut self, name: &str, base: &str) -> Result<(), EbpfError>
    where
        E: EbpfLoad + EbpfPin,
        for<'a> &'a mut E: TryFrom<&'a mut Program, Error = ProgramError>,
    {
        let program =
            self.program_mut(name)
                .ok_or(EbpfError::ProgramError(ProgramError::InvalidName {
                    name: name.to_string(),
                }))?;
        let inner: &mut E = program.try_into()?;
        inner.load()?;
        let full_path = format!("{base}/{name}");
        inner
            .pin(&full_path)
            .map_err(|e| ProgramError::IOError(io::Error::other(e)))?;

        Ok(())
    }
}

pub trait PinMap {
    fn pin_map(&mut self, name: &str, base: &str) -> Result<(), EbpfError>;
}

impl PinMap for Ebpf {
    fn pin_map(&mut self, name: &str, base: &str) -> Result<(), EbpfError> {
        let map = self
            .map_mut(name)
            .ok_or(EbpfError::MapError(MapError::InvalidName {
                name: name.to_string(),
            }))?;
        let full_path = format!("{base}/{name}");
        map.pin(&full_path)
            .map_err(|e| MapError::IoError(io::Error::other(e)))?;

        Ok(())
    }
}

impl TryMapFromPin for OwnedRingBuf {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError>
    where
        Self: Sized,
    {
        RingBuf::try_from(Map::RingBuf(MapData::from_pin(path)?))
    }
}

impl<K: Pod, V: Pod> TryMapFromPin for OwnedHashMap<K, V> {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError>
    where
        Self: Sized,
    {
        HashMap::<_, K, V>::try_from(Map::HashMap(MapData::from_pin(path)?))
    }
}

impl<V: Pod> TryMapFromPin for OwnedArray<V> {
    fn try_from_pin<P: AsRef<Path>>(path: P) -> Result<Self, MapError>
    where
        Self: Sized,
    {
        Array::<_, V>::try_from(Map::Array(MapData::from_pin(path)?))
    }
}
