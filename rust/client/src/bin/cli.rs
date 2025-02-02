// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use clap::Subcommand;
use client::Client;
use client::ClientError;
use shared::config::BlockingConfig;
use shared::config::Configuration;
use shared::config::FileDescriptorChangeConfig;
use shared::config::Filter;
use shared::config::GarbageCollectConfig;
use shared::config::JniReferencesConfig;
use shared::config::MissingBehavior;
use shared::config::SignalConfig;
use shared::config::UInt32Filter;
use shared::config::UprobeConfig;
use shared::config::WriteConfig;
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
//   client
//       .set_configuration(Configuration {
//           blocking_config: Some(BlockingConfig {threshold: None, filter: Some(Filter { pid_filter: Some(UInt32Filter { missing_behavior: MissingBehavior::NotMatch.into(), r#match: vec![pid], ..Default::default() }), ..Default::default() })}),
//           file_descriptor_change_config: Some(FileDescriptorChangeConfig {filter: Some(Filter { pid_filter: Some(UInt32Filter { missing_behavior: MissingBehavior::NotMatch.into(), r#match: vec![pid], ..Default::default() }), ..Default::default() })}),
//           garbage_collect_config: Some(GarbageCollectConfig { filter: Some(Filter { pid_filter: Some(UInt32Filter { missing_behavior: MissingBehavior::NotMatch.into(), r#match: vec![pid], ..Default::default() }), ..Default::default() })}),
//           jni_references_config: Some(JniReferencesConfig {filter: Some(Filter { pid_filter: Some(UInt32Filter { missing_behavior: MissingBehavior::NotMatch.into(), r#match: vec![pid], ..Default::default() }), ..Default::default() })}),
//           signal_config: Some(SignalConfig {filter: Some(Filter { pid_filter: Some(UInt32Filter { missing_behavior: MissingBehavior::NotMatch.into(), r#match: vec![pid], ..Default::default() }), ..Default::default() })}),
//           uprobe_configs: vec![],
//           write_config: Some(WriteConfig {filter: Some(Filter { pid_filter: Some(UInt32Filter { missing_behavior: MissingBehavior::NotMatch.into(), r#match: vec![pid], ..Default::default() }), ..Default::default() })})
//       })
//       .await?;

    let mut stream = client.init_stream().await?;

    while let Some(next) = stream.next().await {
        println!("{next:?}");
    }

    Ok(())
}

async fn set_config(client: &mut Client) -> Result<()> {
    Ok(())
   //client
   //    .set_configuration(Configuration {
   //        uprobes: vec![],
   //        vfs_write: Some(VfsWriteConfig {
   //            entries: HashMap::new(),
   //        }),
   //        sys_sendmsg: Some(SysSendmsgConfig {
   //            entries: HashMap::new(),
   //        }),
   //        jni_references: None,
   //        sys_sigquit: Some(SysSigquitConfig { pids: vec![] }),
   //        gc: None,
   //        sys_fd_tracking: Some(SysFdTrackingConfig { pids: vec![] }),
   //    })
   //    .await?;
   //println!("Success");
   //Ok(())
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
