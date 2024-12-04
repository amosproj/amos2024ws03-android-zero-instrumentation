// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use object::{Object, ObjectSymbol, ReadCache};
use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Command;

// TODO: custom error type for file

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

pub async fn some_entry_method(pid: i32) -> Result<u64, SymbolError> {
    let oat_file_paths = get_oat_files(pid)?;
    if oat_file_paths.len() == 0 {
        // TODO: generate oat-file and wait till it is available

        // till implemented:
        return Err(SymbolError::Other {
            text: format!("Could not find oat file for process with pid: {}", pid),
        });
    }
    Ok(parse_oat_files_for_function(
        &oat_file_paths,
        "java.lang.String uniffi.shared.UprobeConfig.component1()",
        "ziofa",
    )
    .await?)
}

pub fn get_oat_files(pid: i32) -> Result<Vec<PathBuf>, ProcError> {
    // get from : /proc/pid/maps -> oat directory (directory with all the odex files)

    let process = Process::new(pid)?;
    let maps = process.maps()?;
    let all_files = maps
        .iter()
        .filter_map(|mm_map| match mm_map.clone().pathname {
            MMapPath::Path(path) => Some(path),
            _ => None,
        })
        .filter(|path: &PathBuf| path.to_str().unwrap().contains(".odex"))
        .collect();

    Ok(all_files)
}

async fn parse_oat_files_for_function(
    oat_file_paths: &Vec<PathBuf>,
    symbol_name: &str,
    function_name: &str,
) -> Result<u64, SymbolError> {
    for path in oat_file_paths {
        // for testing the code only
        if !path.to_str().unwrap().contains(function_name) {
            continue;
        }

        let file_length = parse_oat_file(path, symbol_name).await?;
        return Ok(file_length);
    }
    Err(SymbolError::Other {
        text: "no ziofa oat file".to_string(),
    })
}

async fn parse_oat_file(path: &PathBuf, symbol_name: &str) -> Result<u64, SymbolError> {
    let section_address = tokio::task::spawn_blocking({
        let path = path.clone();
        move || get_symbol_address_from_oat(&path, "oatdata")
    })
    .await??;
    
    let _oatdump_status = Command::new("oatdump")
        .args(vec![
            "--output=/data/local/tmp/dump.json",
            "--dump-method-and-offset-as-json",
            format!("--oat-file={}", path.to_str().unwrap().to_string()).as_str(),
        ])
        .spawn()
        .expect("oatdump failed to spawn")
        .wait()
        .await
        .expect("oatdump failed to run");
    // TODO: Check for status [robin]
    //       do we even need the status? -> if yes for what? [beni]
    
    let json_file = File::open("/data/local/tmp/dump.json").unwrap();
    let json_reader = BufReader::new(json_file);
    let json = serde_json::Deserializer::from_reader(json_reader).into_iter::<Symbol>();

    for res in json {
        if let Ok(symbol) = res {
            if symbol.method == symbol_name {
                let symbol_address = u64::from_str_radix(symbol.offset.strip_prefix("0x").unwrap(), 16).unwrap();
                if symbol_address == 0 {
                    return Err(SymbolError::SymbolIsNotCompiled {
                        symbol: symbol_name.to_string(),
                    });
                }
                return Ok(symbol_address + section_address);
            }
        }
    }
    // Problem: sync code in async fn
    // TODO: Error handling

    Err(SymbolError::SymbolDoesNotExist {
        symbol: symbol_name.to_string(),
    })
}

fn get_symbol_address_from_oat(path: &PathBuf, symbol_name: &str) -> Result<u64, std::io::Error> {
    let file = File::open(path)?;
    let file_chache = ReadCache::new(file);
    let obj = object::File::parse(&file_chache).unwrap();

    let section = obj
        .dynamic_symbols()
        .find(|s| s.name() == Ok(symbol_name))
        .unwrap();

    Ok(section.address())
}
