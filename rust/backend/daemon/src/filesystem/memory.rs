// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::io;

use std::collections::HashMap;
use tokio::sync::RwLock;

use shared::config::Configuration;

use super::Filesystem;

pub struct MemoryFilesystem {
    storage: RwLock<HashMap<String, Configuration>>,
}

impl MemoryFilesystem {
    pub fn new() -> Self {
        MemoryFilesystem {
            storage: RwLock::new(HashMap::new()),
        }
    }
}

impl Filesystem for MemoryFilesystem {
    fn load(&self, path: &str) -> io::Result<Configuration> {
        tokio::task::block_in_place(|| {
            let storage = self.storage.blocking_read();
            storage.get(path).cloned().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Configuration not found")
            })
        })
    }

    fn save(&self, config: &Configuration, path: &str) -> io::Result<()> {
        tokio::task::block_in_place(|| {
            let mut storage = self.storage.blocking_write();
            storage.insert(path.to_string(), config.clone());
            Ok(())
        })
    }
}
