// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use procfs::process::Process;
use procfs::ProcError;
use std::path::{Path, PathBuf};

// TODO: custom error type for file

fn parse_oat_files(pid: i32) -> Result<(), ProcError> {
    let oat_file_path = oat_file_exists(pid)?;
    parse_oat_file(&oat_file_path)?;
    Ok(())
}

fn parse_oat_file(oat_path: &Path) -> Result<(), ProcError> {
    todo!(" implement ")
}

fn oat_file_exists(pid: i32) -> Result<PathBuf, ProcError> {
    // get from : /proc/pid/maps -> oat directory (directory with all the odex files)

    let process = Process::new(pid)?;
    let maps = process.maps()?;

    // TODO: Implement this
    let path_from_map = "";

    let mut path_buf = PathBuf::new();
    path_buf.push(path_from_map);

    Ok(path_buf)
}
