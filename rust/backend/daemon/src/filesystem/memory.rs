// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, io};

use shared::config::Configuration;
use tokio::sync::RwLock;

use super::ConfigurationStorage;

pub struct MemoryConfigurationStorage {
    storage: RwLock<HashMap<String, Configuration>>,
}

impl MemoryConfigurationStorage {
    #[allow(dead_code)]
    pub fn new() -> Self {
        MemoryConfigurationStorage {
            storage: RwLock::new(HashMap::new()),
        }
    }
}

impl ConfigurationStorage for MemoryConfigurationStorage {
    async fn load(&self, path: &str) -> io::Result<Configuration> {
        tokio::task::block_in_place(|| {
            let storage = self.storage.blocking_read();
            storage
                .get(path)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Configuration not found"))
        })
    }

    async fn save(&self, config: &Configuration, path: &str) -> io::Result<()> {
        tokio::task::block_in_place(|| {
            let mut storage = self.storage.blocking_write();
            storage.insert(path.to_string(), config.clone());
            Ok(())
        })
    }
}
