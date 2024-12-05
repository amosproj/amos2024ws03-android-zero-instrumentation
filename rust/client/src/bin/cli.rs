// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use clap::Subcommand;
use client::Client;
use client::ClientError;
use shared::config::{Configuration, SysSendmsgConfig, VfsWriteConfig};
use std::collections::HashMap;
use tokio_stream::StreamExt;

pub type Result<T> = core::result::Result<T, ClientError>;

#[derive(Debug, Parser)]
struct Args {
    /// Address the client binds to
    #[arg(long, default_value = "http://127.0.0.1:50051")]
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

    /// Send and receive an empty request/response to/from the server
    Check,

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

    /// Get all symbols with their offsets
    Symbols {
        /// Pid for which to get the symbols
        #[arg(short, long)]
        pid: u32,

        /// Path to the .odex file which should be crawled
        #[arg(short, long)]
        odex_file: String,

        /// Only output number of symbols
        #[arg(short, long)]
        silent: bool,
    },
}

async fn sendmsg(client: &mut Client, pid: u32) -> Result<()> {
    client
        .set_configuration(Configuration {
            uprobes: vec![],
            vfs_write: None,
            sys_sendmsg: Some(SysSendmsgConfig {
                entries: HashMap::from([(pid, 0)]),
            }),
        })
        .await?;

    let mut stream = client.init_stream().await?;

    while let Some(next) = stream.next().await {
        println!("{next:?}");
    }

    Ok(())
}

async fn set_config(client: &mut Client) -> Result<()> {
    println!(
        "response_type: {}",
        client
            .set_configuration(Configuration {
                uprobes: vec![],
                vfs_write: Some(VfsWriteConfig {
                    entries: std::collections::HashMap::new(),
                }),
                sys_sendmsg: Some(SysSendmsgConfig {
                    entries: std::collections::HashMap::new(),
                }),
            })
            .await?
    );

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

async fn get_symbols(client: &mut Client, pid: u32, odex_file: String, silent: bool) -> Result<()> {
    let mut stream = client.get_symbols(pid, odex_file).await?;
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

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let mut client = Client::connect(args.addr.to_owned()).await?;

    match args.cmd {
        Commands::Check => {
            client.check_server().await?;
            println!("Success");
        }
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
        },
        Commands::Symbols { pid, odex_file, silent } => {
            get_symbols(&mut client, pid, odex_file, silent).await?;
        }
    }

    Ok(())
}
