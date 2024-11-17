// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use shared::config::{Configuration, EbpfEntry};
use std::iter::Map;
use std::{
    fs::File,
    io,
    io::{BufReader, BufWriter},
};
use aya::programs::Program;
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

    #[error("Ebpf functions mentioned don't exist")]
    EbpfFunctionNoneExistent { names: Vec<String> },

    #[error(transparent)]
    LoadFailed(#[from] io::Error),

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

pub fn validate(new_config: &Configuration, config_path: &str, ebpf_progs: Vec<(&str, &Program)>) -> Result<(), ConfigError> {

    // has to have the same amount of entries
    // entries have to match (no entries are allowed to be dropped, or added)
    // all entries have to be correct

    let old_config = match load_from_file(config_path) {
        Ok(config) => config,
        Err(_) => return Ok(()),
    };

    if old_config.entries.len() != new_config.entries.len() {
        return Err(
            ConfigError::WrongEntryCount {
                old_config_count: old_config.entries.len() as u32,
                new_config_count: new_config.entries.len() as u32,
            }
        );
    }

    let entries_not_contained = old_config.entries.iter().filter(
        |entry| !config_contains(new_config, entry)
    ).collect::<Vec<&EbpfEntry>>();

    if !entries_not_contained.len() == 0 {
        return Err(
            ConfigError::EntryDropped {
                entry_names: Map::collect(
                    entries_not_contained.iter().map(
                        |ebpf_entry: &&EbpfEntry| { ebpf_entry.ebpf_name.clone() }
                    )
                )
            });
    }

    let ebpf_prog_names: Vec<&str> = ebpf_progs.iter().map(|&(name, _)| { name }).collect();

    let ebpf_functions_none_existent: Vec<&EbpfEntry> = new_config.entries.iter().filter(
        |ebpf_entry: &&EbpfEntry| { !ebpf_prog_names.contains(&&*ebpf_entry.ebpf_name) }
    ).collect();
    if ebpf_functions_none_existent.len() != 0 {
        return Err(
            ConfigError::EbpfFunctionNoneExistent {
                names: ebpf_functions_none_existent.iter().map(|ebpf_entry: &&EbpfEntry| { ebpf_entry.ebpf_name.clone() }).collect() 
            }
        );
    }

    Ok(())
}

fn config_contains(config: &Configuration, entry: &EbpfEntry) -> bool {
    let mut cont = true;
    for conf_entry in config.entries.iter() {
        if !elem_is_equal(conf_entry, entry) {
            cont = false;
        }
    }
    cont
}


fn elem_is_equal(elem_1: &EbpfEntry, elem_2: &EbpfEntry) -> bool {
    // TODO: think about a way to do this right :)
    elem_1.ebpf_name == elem_2.ebpf_name
}