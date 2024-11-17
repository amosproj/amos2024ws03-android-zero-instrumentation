// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use shared::config::{Configuration, EbpfEntry};
use std::{
    fs::File,
    io,
    io::{BufReader, BufWriter},
};
use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Validation: wrong entry count")]
    WrongEntryCount {
        old_config_count: u32,
        new_config_count: u32,
    },

    #[error("Validation: and entry was in the old config but is not in the new one")]
    EntryDropped { entry_names: Vec<String> },

    #[error(transparent)]
    LoadFailed(#[from] io::Error),

    #[error(transparent)]
    SaveFailed(#[from] io::Error),
}

impl From<ConfigError> for Status {
    fn from(value: ConfigError) -> Self {
        Self::from_error(Box::new(value))
    }
}

pub fn load_from_file(path: &str) -> Result<Configuration, io::Error> {
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

pub fn validate(new_config: &Configuration, config_path: &str) -> Result<(), ConfigError> {
    //TODO: Implement this function

    // has to have the same amount of entries
    // entries have to match (no entries are allowed to be dropped, or added)
    // all entries have to be correct

    let mut old_config = match load_from_file(config_path) {
        Ok(config) => config,
        Err(e) => return Err(ConfigError::LoadFailed(e)),
    };
    
    if old_config.entries.len() != new_config.entries.len() {
        return Err(
            ConfigError::WrongEntryCount { 
                old_config_count: old_config.entries.len() as u32, 
                new_config_count: new_config.entries.len() as u32
            }
        )
    }
    
    let entries_not_contained = old_config.entries.iter().filter(
        |entry| !new_config.entries.contains(entry)
    ).collect::<Vec<&EbpfEntry>>();
    if !entries_not_contained {
        return Err(
            ConfigError::EntryDropped {
                entry_names: new_config.entries.map(|entry| {entry.ebpf_name}).collect()
            }
        )
    }
    
    Ok(())
}

fn equal_entries(old: &EbpfEntry, new: &EbpfEntry) -> bool {
    // TODO: extend method to check if all but the attach values are equal
    old.ebpf_name == new.ebpf_name
}
