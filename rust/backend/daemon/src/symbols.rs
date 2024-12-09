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
    #[error("The desired odex file isn't available. Did you call get_odex_files()?")]
    OdexFileNotAvailable { odex_path: PathBuf },
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
    /// maps pid to odex files
    odex_files: HashMap<u32, HashSet<PathBuf>>,
    /// maps odex file path and symbol name to offset
    symbols: HashMap<PathBuf, HashMap<String, u64>>,
}

impl SymbolHandler {
    pub fn new() -> Self {
        SymbolHandler {
            odex_files: HashMap::new(),
            symbols: HashMap::new(),
        }
    }

    /// load the paths to all odex files
    // TODO: blocking?
    fn load_odex_paths(&mut self, pid: u32) -> Result<(), ProcError> {
        // if pid was already crawled, nothing it to do
        // TODO: Check for old/potentially outdated entries and reload them
        if self.odex_files.contains_key(&pid) {
            return Ok(());
        }
        let odex_files = self.odex_files.entry(pid).or_default();

        let process = Process::new(i32::try_from(pid).unwrap())?;
        let maps = process.maps()?;

        // for each .odex file: insert a new hashmap into this pids entry of self.symbols
        for odex in maps
            .iter()
            .filter_map(|mm_map| match mm_map.clone().pathname {
                MMapPath::Path(path) => Some(path),
                _ => None,
            })
            .filter(|path: &PathBuf| path.to_str().unwrap().ends_with(".odex"))
        {
            odex_files.insert(odex.clone());
            self.symbols.insert(odex, HashMap::new());
        }

        Ok(())
        // TODO: Remove old/long unused paths from cache
    }

    pub fn get_odex_paths(&mut self, pid: u32) -> Result<&HashSet<PathBuf>, SymbolError> {
        self.load_odex_paths(pid)?;

        self.odex_files
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })
    }

    async fn load_symbols(&mut self, odex_path: &PathBuf) -> Result<(), SymbolError> {
        // if the .odex file is not cached, throw error
        if !self.symbols.contains_key(odex_path) {
            return Err(SymbolError::OdexFileNotAvailable {
                odex_path: odex_path.to_path_buf(),
            });
        }

        // if the symbol map already contains entries, nothing needs to be done
        if !self.symbols.get(odex_path).unwrap().is_empty() {
            return Ok(());
        }

        self.generate_json_oatdump(odex_path)
            .await
            .expect("fn 'load_symbols' Failed to generate oatdump");

        let oatdata_section_offset = self.get_oatsection_address(odex_path).await?;

        let symbols = self.symbols.get_mut(odex_path).unwrap();

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

            symbols.insert(symbol.method, offset);
        }

        Ok(())
    }

    pub async fn get_symbols(
        &mut self,
        odex_path: &PathBuf,
    ) -> Result<&HashMap<String, u64>, SymbolError> {
        self.load_symbols(odex_path).await?;

        Ok(self.symbols.get(odex_path).unwrap())
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
