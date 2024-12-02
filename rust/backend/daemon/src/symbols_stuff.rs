// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use procfs::process::{MMapPath, Process};
use procfs::ProcError;
use std::path::PathBuf;

// TODO: custom error type for file

fn parse_oat_files(pid: i32) -> Result<(), ProcError> {
    let oat_file_path = oat_file_exists(pid)?;
    parse_oat_file(&oat_file_path)?;
    Ok(())
}


fn parse_oat_file(_oat_path: &Vec<PathBuf>) -> Result<(), ProcError> {
    todo!(" implement ")
}

pub fn oat_file_exists(pid: i32) -> Result<Vec<PathBuf>, ProcError> {
    // get from : /proc/pid/maps -> oat directory (directory with all the odex files)

    let process = Process::new(pid)?;
    let maps = process.maps()?;
    let all_files: Vec<PathBuf> = maps.iter().filter_map(
        |mm_map| {
            match mm_map.clone().pathname {
                MMapPath::Path(path) => Some(path),
                _ => None
            }
        }
    // ).filter(
    //     |path: &PathBuf| {
    //         path.ends_with(".odex")
    //     }
    ).collect();


    Ok(all_files)
}
