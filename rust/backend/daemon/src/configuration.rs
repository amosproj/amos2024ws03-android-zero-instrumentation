// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{
    fs::File,
    io,
    io::{BufReader, BufWriter},
};

use shared::config::Configuration;

pub fn load_from_file(path: &str) -> io::Result<Configuration> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
pub fn save_to_file(config: &Configuration, path: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, config)?;
    Ok(())
}

pub fn validate(_config: &Configuration) -> Result<(), io::Error> {
    //TODO: Implement this function
    Ok(())
}
