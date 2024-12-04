// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::constants::OATDUMP_PATH;
use crate::symbols_helpers::{self, get_odex_files_for_pid};
use procfs::ProcError;
use serde::{Deserialize, Serialize};
use serde_json::de::IoRead;
use serde_json::StreamDeserializer;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use symbols_helpers::{generate_json_oatdump, get_section_address};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
struct JsonSymbol {
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
