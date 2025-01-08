// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{fs::File, io::{self, BufReader, BufWriter}};

use shared::config::Configuration;

use super::Filesystem;


pub struct NormalFilesystem;

impl Filesystem for NormalFilesystem {
    fn load(&self, path: &str) -> io::Result<Configuration> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    fn save(&self, config: &Configuration, path: &str) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, config)?;
        Ok(())
    }
}