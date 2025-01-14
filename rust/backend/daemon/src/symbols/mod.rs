// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
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
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

pub mod walking;
pub mod symbolizer;
pub mod actors;
pub mod index;

#[derive(Debug, Error)]
pub enum SymbolError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ProcError(#[from] ProcError),
    #[error("Odex paths are not loaded for specified pid")]
    SymbolPathsNotLoaded { pid: u32 },
    #[error("Symbols are not loaded for specified pid and odex path")]
    SymbolsNotLoaded { pid: u32, path: PathBuf },
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("The desired odex file isn't available. Did you call get_odex_files()?")]
    SymbolFileNotAvailable { path: PathBuf },
    #[error(transparent)]
    ReadError(#[from] object::read::Error),
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
    files: HashMap<u32, HashSet<PathBuf>>,
    /// maps odex file path and symbol name to offset
    symbols: HashMap<PathBuf, HashMap<String, u64>>,
}

impl SymbolHandler {
    pub fn new() -> Self {
        SymbolHandler {
            files: HashMap::new(),
            symbols: HashMap::new(),
        }
    }

    /// load the paths to all odex files
    // TODO: blocking?
    fn load_map_paths(&mut self, pid: u32, extension: &str) -> Result<(), ProcError> {
        // if pid was already crawled, nothing it to do
        // TODO: Check for old/potentially outdated entries and reload them
        if self.files.contains_key(&pid) {
            return Ok(());
        }
        let odex_files = self.files.entry(pid).or_default();

        let process = Process::new(i32::try_from(pid).unwrap())?;
        let maps = process.maps()?;

        // for each .odex file: insert a new hashmap into this pids entry of self.symbols
        for odex in maps
            .iter()
            .filter_map(|mm_map| match mm_map.clone().pathname {
                MMapPath::Path(path) => Some(path),
                _ => None,
            })
            .filter(|path: &PathBuf| path.to_str().unwrap().ends_with(extension))
        {
            odex_files.insert(odex.clone());
            self.symbols.insert(odex, HashMap::new());
        }

        Ok(())
        // TODO: Remove old/long unused paths from cache
    }

    pub fn get_paths(&mut self, pid: u32, extension: &str) -> Result<&HashSet<PathBuf>, SymbolError> {
        self.load_map_paths(pid, extension)?;

        self.files
            .get(&pid)
            .ok_or(SymbolError::SymbolPathsNotLoaded { pid })
    }

    async fn load_so_symbols(&mut self, so_path: &PathBuf) -> Result<(), SymbolError> {
        if !self.symbols.contains_key(so_path) {
            return Err(SymbolError::SymbolFileNotAvailable {
                path: so_path.to_path_buf(),
            });
        }

        // if the symbol map already contains entries, nothing needs to be done
        if !self.symbols.get(so_path).unwrap().is_empty() {
            return Ok(());
        }

        let map = tokio::task::spawn_blocking({
            let path = so_path.to_path_buf();
            move || {
                let mut map = HashMap::new();

                let file = File::open(path)?;
                let file_cache = ReadCache::new(file);
                let obj = object::File::parse(&file_cache).unwrap();

                for symbol in obj.dynamic_symbols() {
                    let name = symbol.name()?.to_string();
                    let address = symbol.address();

                    if address == 0 {
                        continue;
                    }

                    map.insert(name, address);
                }

                Ok::<HashMap<String, u64>, SymbolError>(map)
            }
        })
        .await??;

        self.symbols.insert(so_path.clone(), map);

        Ok(())
    }

    async fn load_oat_symbols(&mut self, odex_path: &PathBuf) -> Result<(), SymbolError> {
        // if the .odex file is not cached, throw error
        if !self.symbols.contains_key(odex_path) {
            return Err(SymbolError::SymbolFileNotAvailable {
                path: odex_path.to_path_buf(),
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
        path: &PathBuf,
    ) -> Result<&HashMap<String, u64>, SymbolError> {
        if path.to_str().unwrap().ends_with(".odex") {
            self.load_oat_symbols(path).await?;
        } else if path.to_str().unwrap().ends_with(".so") {
            self.load_so_symbols(path).await?;
        }

        self.symbols
            .get(path)
            .ok_or(SymbolError::SymbolFileNotAvailable {
                path: path.to_path_buf(),
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

    async fn get_oatsection_address(&self, oat_path: &Path) -> Result<u64, io::Error> {
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
