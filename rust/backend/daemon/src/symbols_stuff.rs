// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use object::{Object, ObjectSection, ObjectSymbol, ReadCache};
use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use serde::{Serialize, Deserialize};

// TODO: custom error type for file

#[derive(Serialize, Deserialize, Debug)]
struct Symbol {
    method: String,
    offset: String,
}

pub async fn some_entry_method(pid: i32) -> Result<u64, ProcError> {
    let oat_file_paths = get_oat_files(pid)?;
    if oat_file_paths.len() == 0 {
        // TODO: generate oat-file and wait till it is available

        // till implemented:
        return Err(ProcError::Other(format!(
            "Could not find oat file for process with pid: {}",
            pid
        )));
    }
    Ok(parse_oat_files(&oat_file_paths).await?)
}

async fn parse_oat_files(oat_file_paths: &Vec<PathBuf>) -> Result<u64, ProcError> {
    for path in oat_file_paths {
        // for testing the code only
        if !path.to_str().unwrap().contains("ziofa") {
            continue;
        }

        let file_length = parse_oat_file(path).await?;
        return Ok(file_length);
    }
    Err(ProcError::Other("no ziofa oat file".to_string()))
}

fn get_symbol_address_from_oat<P: AsRef<Path>>(path: P, symbol_name: &str) -> Result<u64, std::io::Error> {
    let file = File::open(path)?;
    let file_chache = ReadCache::new(file);
    let obj = object::File::parse(&file_chache).unwrap();

    let section = obj
        .dynamic_symbols()
        .find(|s| s.name() == Ok(symbol_name))
        .unwrap();

    Ok(section.address())
}

async fn parse_oat_file(path: &PathBuf) -> Result<u64, std::io::Error> {
    let oatdump_status = Command::new("oatdump")
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

    let json_file = File::open("/data/local/tmp/dump.json")?;
    let json_reader = BufReader::new(json_file);
    let json = serde_json::Deserializer::from_reader(json_reader).into_iter::<Symbol>();

    // Problem: sync code in async fn
    let symbol_address= 0;

    let section_address = tokio::task::spawn_blocking({
        let path = path.clone();
        || get_symbol_address_from_oat(path, "oatdata")
    })
    .await??;

    Ok(symbol_address + section_address)
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
