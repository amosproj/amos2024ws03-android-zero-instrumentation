// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::io;
use std::future::Future;

use shared::config::Configuration;

mod memory;
mod normal;

pub use normal::NormalConfigurationStorage;

#[allow(unused_imports)]
pub use memory::MemoryConfigurationStorage;

pub trait ConfigurationStorage: Send + Sync + 'static {
    fn load(&self, path: &str) -> impl Future<Output = io::Result<Configuration>> + Send ;

    fn save(&self, config: &Configuration, path: &str) -> impl Future<Output = io::Result<()>> + Send ;
}
