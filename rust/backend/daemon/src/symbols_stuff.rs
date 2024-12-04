// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::constants::OATDUMP_PATH;
use crate::symbols_stuff_helpers;
use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use serde::{Deserialize, Serialize};
use serde_json::de::IoRead;
use serde_json::StreamDeserializer;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use symbols_stuff_helpers::{generate_json_oatdump, get_section_address};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
struct Symbol {
    method: String,
    offset: String,
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
}

impl From<SymbolError> for tonic::Status {
    fn from(err: SymbolError) -> Self {
        Self::from_error(Box::new(err))
    }
}

pub async fn get_symbol_offset_for_function_of_process(
    pid: i32,
    process_name: &str,
    symbol_name: &str,
) -> Result<u64, SymbolError> {
    let odex_file_paths = get_odex_files_for_pid(pid)?;
    parse_odex_files_for_process(&odex_file_paths, symbol_name, process_name).await
}

pub fn get_odex_files_for_pid(pid: i32) -> Result<Vec<PathBuf>, SymbolError> {
    // get from : /proc/pid/maps -> oat directory (directory with all the odex files)

    let process = Process::new(pid)?;
    let maps = process.maps()?;
    let all_files: Vec<PathBuf> = maps
        .iter()
        .filter_map(|mm_map| match mm_map.clone().pathname {
            MMapPath::Path(path) => Some(path),
            _ => None,
        })
        .filter(|path: &PathBuf| path.to_str().unwrap().contains(".odex"))
        .collect();
    match all_files.len() != 0 {
        true => Ok(all_files),
        false => Err(SymbolError::Other {
            text: format!("Could not find oat file for process with pid: {}", pid),
        }),
    }
}

async fn parse_odex_files_for_process(
    odex_file_paths: &Vec<PathBuf>,
    symbol_name: &str,
    process_name: &str,
) -> Result<u64, SymbolError> {
    for odex_file_path in odex_file_paths {
        // TODO: is this really the way... i doubt it
        if !odex_file_path.to_str().unwrap().contains(process_name) {
            continue;
        }

        return Ok(parse_odex_file(odex_file_path, symbol_name).await?);
    }
    Err(SymbolError::Other {
        text: format!("no oat file found for function-name: {}", process_name),
    })
}

async fn parse_odex_file(odex_file_path: &PathBuf, symbol_name: &str) -> Result<u64, SymbolError> {
    let section_address = get_section_address(odex_file_path).await?;
    generate_json_oatdump(odex_file_path).await?;
    get_symbol_address_from_json(symbol_name, section_address)
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

fn get_oatdump_contents(
) -> Result<StreamDeserializer<'static, IoRead<BufReader<File>>, Symbol>, SymbolError> {
    let json_file = File::open(OATDUMP_PATH)?;
    let json_reader = BufReader::new(json_file);
    Ok(serde_json::Deserializer::from_reader(json_reader).into_iter::<Symbol>())
}

fn get_symbol_address(section_address: u64, symbol: Symbol) -> Result<u64, SymbolError> {
    let symbol_address =  u64::from_str_radix(symbol.offset.strip_prefix("0x").unwrap(), 16).unwrap() ;
    if symbol_address == 0{
        return Err(SymbolError::SymbolIsNotCompiled {
            symbol: symbol.method.to_string(),
        });
    }
    
    Ok(symbol_address + section_address)
}
