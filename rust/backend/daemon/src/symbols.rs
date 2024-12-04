// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::constants::OATDUMP_PATH;
use crate::symbols_helpers::{self, get_odex_files_for_pid};
use object::Symbol;
use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use serde::{Deserialize, Serialize};
use serde_json::de::IoRead;
use serde_json::StreamDeserializer;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use symbols_helpers::{generate_json_oatdump, get_section_address};
use thiserror::Error;
use tokio::io::AsyncBufReadExt;

#[derive(Serialize, Deserialize, Debug)]
struct JsonSymbol {
    method: String,
    offset: String,
}

pub struct SymbolHandler {
    /// maps pid to odex paths supplied by /proc/pid/maps
    odex_paths: HashMap<i32, HashSet<PathBuf>>,
    /// maps pid, odex file path and symbol name to offset
    symbols: HashMap<i32, HashMap<PathBuf, HashMap<String, u64>>>,
}

impl SymbolHandler {
    pub fn new() -> Self {
        SymbolHandler {
            odex_paths: HashMap::new(),
            symbols: HashMap::new(),
        }
    }

    /// loads the paths to all odex files
    // TODO: blocking?
    pub fn load_odex_paths(&mut self, pid: i32) -> Result<(), ProcError> {
        let process = Process::new(pid)?;
        let maps = process.maps()?;

        // TODO: Check for old/potentially outdated entries and reload them
        if self.odex_paths.contains_key(&pid) {
            return Ok(());
        }

        self.odex_paths.insert(
            pid,
            maps.iter()
                .filter_map(|mm_map| match mm_map.clone().pathname {
                    MMapPath::Path(path) => Some(path),
                    _ => None,
                })
                .filter(|path: &PathBuf| path.to_str().unwrap().contains(".odex"))
                .collect(),
        );

        Ok(())
        // TODO: Remove old/long unused paths from cache
    }

    pub fn get_odex_paths(&self, pid: i32) -> Result<&HashSet<PathBuf>, SymbolError> {
        self.odex_paths
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })
    }

    pub async fn load_symbols(&mut self, pid: i32, odex_path: &PathBuf) -> Result<(), SymbolError> {
        generate_json_oatdump(odex_path).await;

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
        for line in lines.next_line().await {
            if line.is_none() {
                break;
            }
            let symbol: JsonSymbol = serde_json::from_str(&line.unwrap())?;
            let offset =
                u64::from_str_radix(symbol.offset.strip_prefix("0x").unwrap(), 16).unwrap();
            symbol_to_offset.insert(symbol.method, offset);
        }

        Ok(())
    }

    pub fn get_symbols(
        &self,
        pid: i32,
        odex_path: &PathBuf,
    ) -> Result<&HashMap<String, u64>, SymbolError> {
        self.symbols
            .get(&pid)
            .ok_or(SymbolError::OdexPathsNotLoaded { pid })?
            .get(odex_path)
            .ok_or(SymbolError::SymbolsNotLoaded {
                pid,
                odex_path: odex_path.to_path_buf(),
            })
    }
}

#[derive(Debug, Error)]
pub enum SymbolError {
    #[error("Symbol doesn't exist")]
    SymbolDoesNotExist { symbol: String },
    #[error("Symbol is not compiled")]
    SymbolIsNotCompiled { symbol: String },
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Other")]
    Other { text: String },
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

pub async fn get_symbol_offset_for_function_of_process(
    pid: i32,
    package_name: &str,
    symbol_name: &str,
) -> Result<u64, SymbolError> {
    let odex_file_paths = get_odex_files_for_pid(pid)?;
    parse_odex_files_for_process(&odex_file_paths, symbol_name, package_name).await
}

async fn parse_odex_files_for_process(
    odex_file_paths: &Vec<PathBuf>,
    symbol_name: &str,
    package_name: &str,
) -> Result<u64, SymbolError> {
    for odex_file_path in odex_file_paths {
        // TODO: is this really the way... i doubt it
        if !odex_file_path.to_str().unwrap().contains(package_name) {
            continue;
        }

        return Ok(parse_odex_file(odex_file_path, symbol_name).await?);
    }
    Err(SymbolError::Other {
        text: format!("no oat file found for package-name: {}", package_name),
    })
}

pub async fn get_symbols_of_pid(pid: i32, package_name: &str) -> Result<Vec<String>, SymbolError> {
    let odex_file_paths = get_odex_files_for_pid(pid)?;
    for odex_file_path in odex_file_paths {
        // TODO: is this really the way... i doubt it
        if !odex_file_path.to_str().unwrap().contains(package_name) {
            continue;
        }
        generate_json_oatdump(&odex_file_path).await?;
        let outdump_contents = get_oatdump_contents()?;
        return Ok(get_symbols_from_json(outdump_contents));
    }

    Err(SymbolError::Other {
        text: format!("no oat file found for package-name: {}", package_name),
    })
}

async fn parse_odex_file(odex_file_path: &PathBuf, symbol_name: &str) -> Result<u64, SymbolError> {
    let section_address = get_section_address(odex_file_path).await?;
    generate_json_oatdump(odex_file_path).await?;
    get_symbol_address_from_json(symbol_name, section_address)
}

fn get_symbols_from_json(
    outdump_contents: StreamDeserializer<'_, IoRead<BufReader<File>>, JsonSymbol>,
) -> Vec<String> {
    outdump_contents
        .filter_map(|c| match c {
            Ok(symbol) => Some(symbol.method),
            Err(_) => None,
        })
        .collect()
}

fn get_symbol_address_from_json(
    symbol_name: &str,
    section_address: u64,
) -> Result<u64, SymbolError> {
    for res in get_oatdump_contents()? {
        if let Ok(symbol) = res {
            if symbol.method == symbol_name {
                return get_symbol_address(section_address, symbol);
            }
        }
    }
    Err(SymbolError::SymbolDoesNotExist {
        symbol: symbol_name.to_string(),
    })
}

pub async fn stream_symbols(
) -> Result<StreamDeserializer<'static, IoRead<BufReader<File>>, JsonSymbol>, SymbolError> {
    let json_file = tokio::fs::File::open(OATDUMP_PATH).await?;
    let json_reader = tokio::io::BufReader::new(json_file);
    let lines = json_reader.lines();
    lines.Ok(serde_json::Deserializer::from_reader(json_reader).into_iter::<JsonSymbol>())
}

fn get_oatdump_contents(
) -> Result<StreamDeserializer<'static, IoRead<BufReader<File>>, JsonSymbol>, SymbolError> {
    let json_file = File::open(OATDUMP_PATH)?;
    let json_reader = BufReader::new(json_file);
    Ok(serde_json::Deserializer::from_reader(json_reader).into_iter::<JsonSymbol>())
}

fn get_symbol_address(section_address: u64, symbol: JsonSymbol) -> Result<u64, SymbolError> {
    let symbol_address =
        u64::from_str_radix(symbol.offset.strip_prefix("0x").unwrap(), 16).unwrap();
    if symbol_address == 0 {
        return Err(SymbolError::SymbolIsNotCompiled {
            symbol: symbol.method.to_string(),
        });
    }

    Ok(symbol_address + section_address)
}
