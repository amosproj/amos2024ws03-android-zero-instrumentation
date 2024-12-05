// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::constants::OATDUMP_PATH;
use object::{Object, ObjectSymbol, ReadCache};
use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Error;
use std::path::{Path, PathBuf};
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
    OdexPathsNotLoaded { pid: u32 },
    #[error("Symbols are not loaded for specified pid and odex path")]
    SymbolsNotLoaded { pid: u32, odex_path: PathBuf },
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("The desired odex file isn't available")]
    OdexFileNotAvailable { pid: u32, odex_path: PathBuf },
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
    symbols: HashMap<u32, HashMap<PathBuf, HashMap<String, u64>>>,
}

impl SymbolHandler {
    pub fn new() -> Self {
        SymbolHandler {
            symbols: HashMap::new(),
        }
    }

    /// load the paths to all odex files
    // TODO: blocking?
    fn load_odex_paths(&mut self, pid: u32) -> Result<(), ProcError> {
        // if pid was already crawled, nothing it to do
        // TODO: Check for old/potentially outdated entries and reload them
        if let Entry::Vacant(e) = self.symbols.entry(pid) {
            e.insert(HashMap::new());
        } else {
            return Ok(());
        }

        let process = Process::new(i32::try_from(pid).unwrap())?;
        let maps = process.maps()?;

        // for each .odex file: insert a new hashmap into this pids entry of self.symbols
        for map in maps
            .iter()
            .filter_map(|mm_map| match mm_map.clone().pathname {
                MMapPath::Path(path) => Some(path),
                _ => None,
            })
            .filter(|path: &PathBuf| path.to_str().unwrap().ends_with(".odex"))
        {
            self.symbols
                .get_mut(&pid)
                .unwrap()
                .insert(map, HashMap::new());
        }

        Ok(())
        // TODO: Remove old/long unused paths from cache
    }

    pub fn get_odex_paths(&mut self, pid: u32) -> Result<HashSet<&PathBuf>, SymbolError> {
        self.load_odex_paths(pid)?;

        Ok(self
            .symbols
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })?
            .keys()
            .collect())
    }

    async fn load_symbols(&mut self, pid: u32, odex_path: &PathBuf) -> Result<(), SymbolError> {
        // make sure the needed data structures are there
        self.load_odex_paths(pid)?;

        // the following is in a code block as the immutable references of self.symbols and
        // odex_to_symbol_map need to be dropped before getting mutable references to them below
        {
            // if the .odex file is not cached, throw error
            let odex_to_symbol_map = self.symbols.get(&pid).unwrap();
            if !odex_to_symbol_map.contains_key(odex_path) {
                return Err(SymbolError::OdexFileNotAvailable {
                    pid,
                    odex_path: odex_path.to_path_buf(),
                });
            }

            // if the map already contains entries, nothing needs to be done
            let symbol_to_offset = odex_to_symbol_map.get(odex_path).unwrap();
            if !symbol_to_offset.is_empty() {
                return Ok(());
            }
        }

        self.generate_json_oatdump(odex_path)
            .await
            .expect("fn 'load_symbols' Failed to generate oatdump");

        let oatdata_section_offset = self.get_oatsection_address(odex_path).await?;

        // get map from odex file path to symbol map
        let odex_to_symbol_map = self.symbols.get_mut(&pid).unwrap();
        let symbol_to_offset = odex_to_symbol_map.get_mut(odex_path).unwrap();

        let json_file = tokio::fs::File::open(OATDUMP_PATH).await?;
        let json_reader = tokio::io::BufReader::new(json_file);
        let mut lines = json_reader.lines();

        // store all symbols with their offsets in the map
        while let Some(line) = lines.next_line().await? {
            let symbol: JsonSymbol = serde_json::from_str(&line)?;
            let relative_offset =
                u64::from_str_radix(symbol.offset.strip_prefix("0x").unwrap(), 16).unwrap();

            // skip uncompiled symbols
            if relative_offset == 0 {
                continue;
            }

            // the actual symbol offset is build from section offset and relative offset
            let offset = relative_offset + oatdata_section_offset;

            symbol_to_offset.insert(symbol.method, offset);
        }

        Ok(())
    }

    pub async fn get_symbols(
        &mut self,
        pid: u32,
        odex_path: &PathBuf,
    ) -> Result<&HashMap<String, u64>, SymbolError> {
        self.load_symbols(pid, odex_path).await?;

        self.symbols
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })?
            .get(odex_path)
            .ok_or(SymbolError::SymbolsNotLoaded {
                pid,
                odex_path: odex_path.to_path_buf(),
            })
    }

    async fn generate_json_oatdump(&self, path: &Path) -> Result<(), SymbolError> {
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

    async fn get_oatsection_address(&self, oat_path: &Path) -> Result<u64, Error> {
        tokio::task::spawn_blocking({
            let path = oat_path.to_path_buf();
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
