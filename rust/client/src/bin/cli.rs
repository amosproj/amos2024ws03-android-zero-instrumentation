// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use client::{Client, ClientError};
use tokio::{join, select, signal::ctrl_c, sync::oneshot};
use tokio_stream::StreamExt;
use shared::config::{Configuration, EbpfEntry};

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[clap(short, long, help = "interface where packets should be counted")]
    iface: String,
}

pub async fn counter_cli(mut client: Client, iface: String) -> anyhow::Result<()> {
    if let Err(e) = client.load().await {
        println!("{e:?}");
    }

    if let Err(e) = client.attach(iface).await {
        println!("{e:?}");
    }

    if let Err(e) = client.start_collecting().await {
        println!("{e:?}");
    }

    let mut stream = client.clone().server_count().await?;

    let (tx, rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        while let Some(count) = stream.next().await {
            println!("{}", count?)
        }
        tx.send(()).unwrap();
        Ok::<(), ClientError>(())
    });

    let shutdown = tokio::spawn(async move {
        select! {
            _ = ctrl_c() => {}
            _ = rx => {}
        }

        if let Err(e) = client.stop_collecting().await {
            println!("{e:?}");
        }

        if let Err(e) = client.unload().await {
            println!("{e:?}");
        }
    });

    let _ = join!(handle, shutdown);

    Ok(())
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let Cli {  .. } = Cli::parse();

    let mut client = Client::connect("http://[::1]:50051".to_owned()).await?;

    client.set_configuration(Configuration {
        entries: vec![
            EbpfEntry {
                attach: true,
                uprobe_info: None,
                hook: "vfs_write".to_owned(),
                fn_id: 0,
                ebpf_name: "vfs_write".to_owned(),
                description: "".to_owned(),
                hr_name: "vfs_write".to_owned(),
            },
            EbpfEntry {
                attach: true,
                uprobe_info: None,
                hook: "vfs_write".to_owned(),
                fn_id: 1,
                ebpf_name: "vfs_write_ret".to_owned(),
                description: "".to_owned(),
                hr_name: "vfs_write_ret".to_owned(),
            }
        ]
    }).await?;

    let mut stream = client.init_stream().await?;

    while let Some(next) = stream.next().await {
        println!("{next:?}");
    }

    Ok(())
}
