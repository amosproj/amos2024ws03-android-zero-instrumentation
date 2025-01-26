// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::io;

use shared::config::Configuration;


mod normal;
mod memory;

pub use normal::NormalConfigurationStorage;

pub use memory::MemoryConfigurationStorage;

pub trait ConfigurationStorage: Send + Sync + 'static {
    async fn load(&self, path: &str) -> io::Result<Configuration>;

    async fn save(&self, config: &Configuration, path: &str) -> io::Result<()>;
}

