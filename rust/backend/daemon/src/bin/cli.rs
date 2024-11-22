// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use shared::{
    config::{Configuration, EbpfEntry},
    ziofa::ziofa_client::ZiofaClient,
};
use tonic::transport::Channel;

#[derive(Parser, Debug)]
struct Args {
    /// Address the client binds to
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    addr: String,

    /// Verbose output (i. e. print processes)
    #[arg(short, long)]
    verbose: bool,
}

async fn test_check_server(client: &mut ZiofaClient<Channel>) {
    println!("TEST check_server");
    match client.check_server(()).await {
        Ok(_) => println!("SUCCESS"),
        Err(e) => println!("ERROR: {:?}", e),
    };
    println!();
}

async fn test_get_configuration(
    client: &mut ZiofaClient<Channel>,
    verbose: bool,
) -> Vec<EbpfEntry> {
    println!("TEST get_configuration");
    let config = match client.get_configuration(()).await {
        Ok(t) => {
            let res = t.into_inner().entries;
            println!("SUCCESS");
            if verbose {
                for (i, e) in res.iter().enumerate() {
                    println!("Entry {}: {:?}", i, e);
                }
            }
            res
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
            Vec::new()
        }
    };
    println!();
    config
}

async fn test_set_configuration(client: &mut ZiofaClient<Channel>, config: Vec<EbpfEntry>) {
    println!("TEST set_configuration");
    match client
        .set_configuration(Configuration { entries: config })
        .await
    {
        Ok(t) => {
            let res = t.into_inner().response_type;
            println!("SUCCESS: {}", res);
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }
    println!();
}

async fn test_list_processes(client: &mut ZiofaClient<Channel>, verbose: bool) {
    println!("TEST list_processes");
    match client.list_processes(()).await {
        Ok(t) => {
            let res = t.into_inner().processes;
            println!("SUCCESS");
            if verbose {
                for (i, p) in res.iter().enumerate() {
                    println!("Process {}: {:?}", i, p);
                }
            }
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }
    println!();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Trying to connect to server: \"{}\"", args.addr);
    let mut client = ZiofaClient::connect(args.addr).await.unwrap();

    test_check_server(&mut client).await;
    let config = test_get_configuration(&mut client, args.verbose).await;
    test_set_configuration(&mut client, config).await;
    test_list_processes(&mut client, args.verbose).await;

    if !args.verbose {
        println!("Note: To view verbose output, pass the \"-v\" flag.");
    }
}
