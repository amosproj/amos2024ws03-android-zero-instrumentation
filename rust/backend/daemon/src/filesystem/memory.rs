// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::io;

use shared::config::Configuration;

use super::Filesystem;

// TODO: members + implementation
pub struct MemoryFilesystem;

impl Filesystem for MemoryFilesystem {
    fn load(&self, _path: &str) -> io::Result<Configuration> {
        todo!()
    }

    fn save(&self, _config: &Configuration, _path: &str) -> io::Result<()> {
        todo!()
    }
}