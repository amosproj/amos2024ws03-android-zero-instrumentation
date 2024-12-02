// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
// TODO: custom error type for file

pub fn some_entry_method(pid: i32) -> Result<u64, ProcError> {
    let oat_file_paths = get_oat_files(pid)?;
    if oat_file_paths.len() == 0 {
        // TODO: generate oat-file and wait till it is available

        // till implemented:
        return Err(ProcError::Other(format!(
            "Could not find oat file for process with pid: {}",
            pid
        )));
    }
    Ok(parse_oat_files(&oat_file_paths)?)
}

fn parse_oat_files(oat_file_paths: &Vec<PathBuf>) -> Result<u64, ProcError> {
    for path in oat_file_paths {
        // for testing the code only
        if !path.to_str().unwrap().contains("ziofa") {
            continue;
        }

        let file_length = parse_oat_file(path)?;
        return Ok(file_length as u64);
    }
    Err(ProcError::Other("no ziofa oat file".to_string()))
}

fn parse_oat_file(path: &PathBuf) -> Result<usize, ProcError> {
    let mut file = File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content.len())
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
