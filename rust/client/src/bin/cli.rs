// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use client::{Client, ClientError};
use shared::config::{Configuration, SysSendmsgConfig};
use tokio::{join, select, signal::ctrl_c, sync::oneshot};
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Parser)]
struct Cli {
    //#[clap(short, long, help = "interface where packets should be counted")]
    //iface: String,
    #[clap(short, long, help = "pid for which to track sendmsg calls")]
    pid: u32
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
    let Cli { pid } = Cli::parse();

    let mut client = Client::connect("http://[::1]:50051".to_owned()).await?;

    client
        .set_configuration(Configuration {
            uprobes: vec![],
            vfs_write: None,
            sys_sendmsg: Some(SysSendmsgConfig { pids: vec![pid] })
        })
        .await?;

    let mut stream = client.init_stream().await?;

    while let Some(next) = stream.next().await {
        println!("{next:?}");
    }

    Ok(())
}
