// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use procfs::process::{MMapPath, Process};
use tokio::process::Command;
use std::fs::File;
use object::{Object, ObjectSymbol, ReadCache};
use std::io::Error;
use crate::constants::OATDUMP_PATH;
use crate::symbols::SymbolError;

pub async fn generate_json_oatdump(path: &PathBuf) -> Result<(), SymbolError> {
    let _oatdump_status = Command::new("oatdump")
        .args(vec![
            format!("--output={}", OATDUMP_PATH).as_str(),
            "--dump-method-and-offset-as-json",
            format!("--oat-file={}", path.to_str().unwrap()).as_str(),
        ])
        .spawn()?
        .wait()
        .await?;
    // TODO: Check for status [robin]
    //       do we even need the status -> if yes for what? [benedikt]
    Ok(())
}

pub async fn get_section_address(oat_path: &PathBuf) -> Result<u64, Error> {
    tokio::task::spawn_blocking({
        let path = oat_path.clone();
        move || get_symbol_address_from_oat(&path, "oatdata")
    })
    .await?
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
    match !all_files.is_empty() {
        true => Ok(all_files),
        false => Err(SymbolError::Other {
            text: format!("Could not find oat file for process with pid: {}", pid),
        }),
    }
}

fn get_symbol_address_from_oat(path: &PathBuf, symbol_name: &str) -> Result<u64, Error> {
    let file = File::open(path)?;
    let file_cache = ReadCache::new(file);
    let obj = object::File::parse(&file_cache).unwrap();

    let section = obj
        .dynamic_symbols()
        .find(|s| s.name() == Ok(symbol_name))
        .unwrap();

    Ok(section.address())
}