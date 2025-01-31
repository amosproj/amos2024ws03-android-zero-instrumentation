// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use clap::Subcommand;
use client::Client;
use client::ClientError;
use shared::config::SysFdTrackingConfig;
use shared::config::{Configuration, SysSendmsgConfig, VfsWriteConfig, SysSigquitConfig};
use std::collections::HashMap;
use tokio_stream::StreamExt;

pub type Result<T> = core::result::Result<T, ClientError>;

#[derive(Debug, Parser)]
struct Args {
    /// Address the client binds to
    #[arg(long, default_value = "http://[::1]:50051")]
    addr: String,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Set a config and subscribe to the event stream
    Sendmsg {
        /// Pid for which to track sendmsg calls
        #[arg(short, long)]
        pid: u32,
    },

    /// Get the current config
    GetConfig,

    /// Set the default config
    SetConfig,

    /// List all running processes
    Processes {
        /// Only output number of processes
        #[arg(short, long)]
        silent: bool,
    },

    /// Get the paths of all .odex files
    Odex {
        /// Pid for which to get the .odex files
        #[arg(short, long)]
        pid: u32,

        /// Only output number of odex files
        #[arg(short, long)]
        silent: bool,
    },

    /// Get the paths of all .so files
    So {
        /// Pid for which to get the .odex files
        #[arg(short, long)]
        pid: u32,

        /// Only output number of odex files
        #[arg(short, long)]
        silent: bool,
    },

    /// Get all symbols with their offsets
    Symbols {
        /// Path to the .odex file which should be crawled
        #[arg(short, long)]
        file: String,

        /// Only output number of symbols
        #[arg(short, long)]
        silent: bool,
    },
    
    /// Create an Index for all Symbols on the System
    IndexSymbols,

    /// Search via query for symbols
    SearchSymbols {
        /// The query string
        query: String,
        
        /// The limit of symbols sent by the server
        limit: u64
    },

    /// Finds the symbol given symbol name and library path
    GetSymbolOffset {
        /// The name of the symbol
        symbol_name: String,
    
        /// The path of the library containing the symbol
        library_path: String
    }
}

async fn sendmsg(client: &mut Client, pid: u32) -> Result<()> {
    client
        .set_configuration(Configuration {
            uprobes: vec![],
            vfs_write: Some(VfsWriteConfig {
                entries: HashMap::from([(pid, 0)]),
            }),
            sys_sendmsg: Some(SysSendmsgConfig {
                entries: HashMap::from([(pid, 0)]),
            }),
            jni_references: None,
            sys_sigquit: Some(SysSigquitConfig { pids: vec![pid] }),
            gc: None,
            sys_fd_tracking: Some(SysFdTrackingConfig { pids: vec![pid] }),
        })
        .await?;

    let mut stream = client.init_stream().await?;

    while let Some(next) = stream.next().await {
        println!("{next:?}");
    }

    Ok(())
}

async fn set_config(client: &mut Client) -> Result<()> {
    client
        .set_configuration(Configuration {
            uprobes: vec![],
            vfs_write: Some(VfsWriteConfig {
                entries: HashMap::new(),
            }),
            sys_sendmsg: Some(SysSendmsgConfig {
                entries: HashMap::new(),
            }),
            jni_references: None,
            sys_sigquit: Some(SysSigquitConfig { pids: vec![] }),
            gc: None,
            sys_fd_tracking: Some(SysFdTrackingConfig { pids: vec![] }),
        })
        .await?;
    println!("Success");
    Ok(())
}

async fn list_processes(client: &mut Client, silent: bool) -> Result<()> {
    let processes = client.list_processes().await?;
    if silent {
        println!("Number of processes: {}", processes.len());
        return Ok(());
    }

    for (i, p) in processes.iter().enumerate() {
        println!("Process {}: {:?}", i, p);
    }

    Ok(())
}

async fn get_odex_files(client: &mut Client, pid: u32, silent: bool) -> Result<()> {
    let mut stream = client.get_odex_files(pid).await?;
    let mut count: u32 = 0;

    while let Some(Ok(next)) = stream.next().await {
        if !silent {
            println!("{}", next.name);
        } else {
            count += 1;
        }
    }

    if silent {
        println!("Number of .odex files: {}", count);
    }

    Ok(())
}

async fn get_so_files(client: &mut Client, pid: u32, silent: bool) -> Result<()> {
    let mut stream = client.get_so_files(pid).await?;
    let mut count: u32 = 0;

    while let Some(Ok(next)) = stream.next().await {
        if !silent {
            println!("{}", next.name);
        } else {
            count += 1;
        }
    }

    if silent {
        println!("Number of .so files: {}", count);
    }

    Ok(())
}

async fn get_symbols(client: &mut Client, file: String, silent: bool) -> Result<()> {
    let mut stream = client.get_symbols(file).await?;
    let mut count: u32 = 0;

    while let Some(Ok(next)) = stream.next().await {
        if !silent {
            println!("method: {} | offset: {}", next.method, next.offset);
        } else {
            count += 1;
        }
    }

    if silent {
        println!("Number of symbols: {}", count);
    }

    Ok(())
}

async fn index_symbols(client: &mut Client) -> Result<()> {
    println!("Indexing Symbols, this can take a while...");
    client.index_symbols().await?;
    println!("SUCCESS");
    Ok(())
}

async fn search_symbols(client: &mut Client, query: String, limit: u64) -> Result<()> {
    let symbols = client.search_symbols(query, limit).await?;
    
    let mut count = 0;
    for symbol in symbols {
        println!("method: {} | offset: {} | library_path: {}", symbol.method, symbol.offset, symbol.path);
        count += 1;
    }
    println!("Total number of symbols: {count}");
    
    Ok(())
}

async fn get_symbol_offset(client: &mut Client, symbol_name: String, library_path: String) -> Result<()> {
    let offset = client.get_symbol_offset(symbol_name, library_path).await?;
    
    if let Some(offset) = offset {
        println!("Found offset: {offset}");
    } else {
        println!("Did not find symbol");
    }
    
    Ok(())
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    println!("Trying to connect to {}", args.addr);
    let mut client = Client::connect(args.addr.to_owned()).await?;

    match args.cmd {
        Commands::Sendmsg { pid } => {
            sendmsg(&mut client, pid).await?;
        }
        Commands::GetConfig => {
            println!("{:?}", client.get_configuration().await?);
        }
        Commands::SetConfig => {
            set_config(&mut client).await?;
        }
        Commands::Processes { silent } => {
            list_processes(&mut client, silent).await?;
        }
        Commands::Odex { pid, silent } => {
            get_odex_files(&mut client, pid, silent).await?;
        }
        Commands::Symbols { file, silent } => {
            get_symbols(&mut client, file, silent).await?;
        }
        Commands::So { pid, silent } => {
            get_so_files(&mut client, pid, silent).await?;
        }
        Commands::IndexSymbols => {
            index_symbols(&mut client).await?;
        }
        Commands::SearchSymbols { query, limit } => {
            search_symbols(&mut client, query, limit).await?;
        }
        Commands::GetSymbolOffset { symbol_name, library_path } => {
            get_symbol_offset(&mut client, symbol_name, library_path).await?;
        }
    }

    Ok(())
}
