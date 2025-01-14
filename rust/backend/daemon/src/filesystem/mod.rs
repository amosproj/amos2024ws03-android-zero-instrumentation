// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::io;

use shared::config::Configuration;


mod normal;
mod memory;

pub use normal::NormalFilesystem;

// TODO: pub use memory::MemoryFilesystem;

/*
 * TODOs:
 * - This should probably not be named Filesystem, because the functionality is much more narrow
 *   than that. Maybe something like ConfigurationStore or ConfigurationStorage?
 * - The trait should definetly be async, because otherwise we always have to use spawn_blocking.
 *   See the tokio documentation for why: https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html
 *   You can use tokio::fs for file system operations.
 */
pub trait Filesystem: Send + Sync + 'static {
    fn load(&self, path: &str) -> io::Result<Configuration>;

    fn save(&self, config: &Configuration, path: &str) -> io::Result<()>;
}

