// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
use shared::ziofa::PidMessage;
use shared::{
    config::{Configuration, SysSendmsgConfig, VfsWriteConfig},
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

    /// for checking the oat_file_exists method
    #[arg(long)]
    pid: i32,
}

async fn test_oat_file_exist(client: &mut ZiofaClient<Channel>, pid: i32) {
    println!("TEST checking oat_file_exists");
    match client.test_oat_file_exists(PidMessage { pid }).await {
        Ok(list_of_oatfiles) => {
            println!("SUCCESS");
            for file in list_of_oatfiles.into_inner().paths {
                println!("{:?}", file);
            }
        }
        Err(e) => println!("ERROR: {:?}", e),
    };
    println!();
}

async fn test_some_entry_method(client: &mut ZiofaClient<Channel>, pid: i32) {
    println!("TEST checking some_entry_method");
    match client.test_some_entry_method(PidMessage { pid }).await {
        Ok(byte_size_of_odex_file) => {
            println!("SUCCESS");
            println!(
                "Size of odex file in bytes: {}",
                byte_size_of_odex_file.into_inner().content_length
            )
        }
        Err(e) => println!("ERROR: {:?}", e),
    };
    println!();
}

async fn test_check_server(client: &mut ZiofaClient<Channel>) {
    println!("TEST check_server");
    match client.check_server(()).await {
        Ok(_) => println!("SUCCESS"),
        Err(e) => println!("ERROR: {:?}", e),
    };
    println!();
}

async fn test_get_configuration(client: &mut ZiofaClient<Channel>, verbose: bool) -> Configuration {
    println!("TEST get_configuration");
    let config = match client.get_configuration(()).await {
        Ok(t) => {
            let res = t.into_inner();
            println!("SUCCESS");

            if verbose {
                println!("{:?}", res);
            }

            res
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
            Configuration {
                uprobes: vec![],
                vfs_write: Some(VfsWriteConfig { entries: std::collections::HashMap::new() }),
                sys_sendmsg: Some(SysSendmsgConfig { entries: std::collections::HashMap::new() }),
            }
        }
    };
    println!();
    config
}

async fn test_set_configuration(client: &mut ZiofaClient<Channel>, config: Configuration) {
    println!("TEST set_configuration");
    match client.set_configuration(config).await {
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
    test_oat_file_exist(&mut client, args.pid).await;
    test_some_entry_method(&mut client, args.pid).await;

    if !args.verbose {
        println!("Note: To view verbose output, pass the \"-v\" flag.");
    }
}
