// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{ffi::OsStr, path::{Path, PathBuf}};

use async_walkdir::{DirEntry, Error, WalkDir};
use tokio_stream::{Stream, StreamExt};

#[derive(Debug)]
pub enum SymbolFilePath {
    Odex(PathBuf),
    Oat(PathBuf),
    Art(PathBuf),
    So(PathBuf)
}

impl TryFrom<PathBuf> for SymbolFilePath {
    type Error = PathBuf;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        match value.extension().and_then(OsStr::to_str) {
            Some("odex") => Ok(SymbolFilePath::Odex(value)),
            Some("oat") => Ok(SymbolFilePath::Oat(value)),
            Some("art") => Ok(SymbolFilePath::Art(value)),
            Some("so") => Ok(SymbolFilePath::So(value)),
            _ => Err(value)
        }
    }
}

impl SymbolFilePath {
    pub fn path(&self) -> &Path {
        match self {
            SymbolFilePath::Odex(path_buf) => path_buf,
            SymbolFilePath::Oat(path_buf) => path_buf,
            SymbolFilePath::Art(path_buf) => path_buf,
            SymbolFilePath::So(path_buf) => path_buf,
        }
    }
}

fn path_filter_map(entry: Result<DirEntry, Error>) -> Option<SymbolFilePath> {
    entry.ok()?.path().try_into().ok()
}

pub fn all_symbol_files() -> impl Stream<Item = SymbolFilePath> {
    let system = WalkDir::new("/system");
    let system_ext = WalkDir::new("/system_ext");
    let data = WalkDir::new("/data");
    let product = WalkDir::new("/product");
    let vendor = WalkDir::new("/vendor");

    system.merge(system_ext).merge(data).merge(product).merge(vendor).filter_map(path_filter_map)
}