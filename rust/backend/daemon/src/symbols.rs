// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::constants::OATDUMP_PATH;
use object::{Object, ObjectSymbol, ReadCache};
use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;
use thiserror::Error;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum SymbolError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ProcError(#[from] ProcError),
    #[error("Odex paths are not loaded for specified pid")]
    OdexPathsNotLoaded { pid: i32 },
    #[error("Symbols are not loaded for specified pid and odex path")]
    SymbolsNotLoaded { pid: i32, odex_path: PathBuf },
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

impl From<SymbolError> for tonic::Status {
    fn from(err: SymbolError) -> Self {
        Self::from_error(Box::new(err))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonSymbol {
    method: String,
    offset: String,
}

pub struct SymbolHandler {
    /// maps pid, odex file path and symbol name to offset
    symbols: HashMap<i32, HashMap<PathBuf, HashMap<String, u64>>>,
}

impl SymbolHandler {
    pub fn new() -> Self {
        SymbolHandler {
            symbols: HashMap::new(),
        }
    }

    /// loads the paths to all odex files
    // TODO: blocking?
    fn load_odex_paths(&mut self, pid: i32) -> Result<(), ProcError> {
        let process = Process::new(pid)?;
        let maps = process.maps()?;

        // TODO: Check for old/potentially outdated entries and reload them
        if self.symbols.contains_key(&pid) {
            return Ok(());
        }

        for _maps in maps.iter()
            .filter_map(|mm_map| match mm_map.clone().pathname {
                MMapPath::Path(path) => Some(path),
                _ => None,
            })
            .filter(|path: &PathBuf| path.to_str().unwrap().contains(".odex"))
        {
            self.symbols.insert(pid, HashMap::new());
        }

        Ok(())
        // TODO: Remove old/long unused paths from cache
    }

    pub fn get_odex_paths(&mut self, pid: i32) -> Result<HashSet<&PathBuf>, SymbolError> {
        if !self.symbols.contains_key(&pid) {
            self.load_odex_paths(pid)?;
        }

        Ok(self.symbols
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })?
            .keys()
            .collect()
        )
    }

    async fn load_symbols(&mut self, pid: i32, odex_path: &PathBuf) -> Result<(), SymbolError> {
        // TODO: Check cache before re-generating oatdump...

        self.generate_json_oatdump(odex_path)
            .await
            .expect("fn 'load_symbols' Failed to generate oatdump");

        let oatdata_section_offset = self.get_oatsection_address(odex_path).await?;

        let json_file = tokio::fs::File::open(OATDUMP_PATH).await?;
        let json_reader = tokio::io::BufReader::new(json_file);
        let mut lines = json_reader.lines();

        // check if pid already has a hash map, insert one otherwise
        if !self.symbols.contains_key(&pid) {
            self.symbols.insert(pid, HashMap::new());
        }

        // get map from odex file path to symbol map
        let odex_to_symbol_map = self.symbols.get_mut(&pid).unwrap();

        // if the wished odex file path already contains a symbol map, nothing needs to be done
        // TODO: Check for old/potentially outdated entries and reload them
        if !odex_to_symbol_map.contains_key(odex_path) {
            odex_to_symbol_map.insert(odex_path.to_path_buf(), HashMap::new());
        } else {
            return Ok(());
        }
        let symbol_to_offset = odex_to_symbol_map.get_mut(odex_path).unwrap();

        // store all symbols with their offsets in the map
        while let Some(line) = lines.next_line().await? {
            let symbol: JsonSymbol = serde_json::from_str(&line)?;
            // the actual symbol offset is build from section offset and relative offset
            let offset = oatdata_section_offset +
                u64::from_str_radix(symbol.offset.strip_prefix("0x").unwrap(), 16).unwrap();
            symbol_to_offset.insert(symbol.method, offset);
        }

        Ok(())
    }

    pub async fn get_symbols(
        &mut self,
        pid: i32,
        odex_path: &PathBuf,
    ) -> Result<&HashMap<String, u64>, SymbolError> {
        // TODO: check if was already generated, if not exec. load symbol...
        if !self.symbols.contains_key(&pid) {
            self.load_odex_paths(pid)?;
        }

        if !self.symbols.get(&pid).unwrap().contains_key(odex_path) {
            self.load_symbols(pid, odex_path).await?;
        }

        self.symbols
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })?
            .get(odex_path)
            .ok_or(SymbolError::SymbolsNotLoaded {
                pid,
                odex_path: odex_path.to_path_buf(),
            })
    }

    async fn generate_json_oatdump(&self, path: &PathBuf) -> Result<(), SymbolError> {
        Command::new("oatdump")
            .args(vec![
                format!("--output={}", OATDUMP_PATH).as_str(),
                "--dump-method-and-offset-as-json",
                format!("--oat-file={}", path.to_str().unwrap()).as_str(),
            ])
            .spawn()?
            .wait()
            .await?;
        Ok(())
    }

    async fn get_oatsection_address(&self, oat_path: &PathBuf) -> Result<u64, Error> {
        tokio::task::spawn_blocking({
            let path = oat_path.clone();
            move || {
                let file = File::open(path)?;
                let file_cache = ReadCache::new(file);
                let obj = object::File::parse(&file_cache).unwrap();

                let section = obj
                    .dynamic_symbols()
                    .find(|s| s.name() == Ok("oatdata"))
                    .unwrap();

                Ok(section.address())
            }
        })
            .await?
    }
}