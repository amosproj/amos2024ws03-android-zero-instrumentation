// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use clap::Parser;
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

async fn test_get_symbols(
    _client: &mut ZiofaClient<Channel>,
    _pid: i32,
    _package_name: String,
    _verbose: bool,
) {
    todo!("implement");
    // println!("TEST get_symbols_of_process");
    //
    // match () {
    //     Ok(res) => {
    //         let names = res.into_inner().names;
    //         println!("SUCCESS");
    //         if verbose {
    //             for (i, s) in names.iter().enumerate() {
    //                 println!("Symbol {}: {}", i, s);
    //             }
    //         }
    //     }
    //     Err(e) => println!("ERROR: {:?}", e),
    // };
    // println!();
}

async fn test_get_address_of_symbol(
    _client: &mut ZiofaClient<Channel>,
    _name: String,
    _pid: i32,
    _package_name: String,
) {
    todo!("implement");
    // println!("TEST get_address_of_symbol");
    //
    // match Ok(())
    // {
    //     Ok(res) => {
    //         let offset = res.into_inner().offset;
    //         println!("SUCCESS: {}", offset);
    //     }
    //     Err(e) => println!("ERROR: {:?}", e),
    // };
    //
    // println!();
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
                vfs_write: Some(VfsWriteConfig {
                    entries: std::collections::HashMap::new(),
                }),
                sys_sendmsg: Some(SysSendmsgConfig {
                    entries: std::collections::HashMap::new(),
                }),
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
    test_get_symbols(
        &mut client,
        args.pid,
        "de.amosproj3.ziofa".to_string(),
        args.verbose,
    )
    .await;
    test_get_address_of_symbol(
        &mut client,
        "java.lang.String uniffi.shared.UprobeConfig.component1()".to_string(),
        args.pid,
        "de.amosproj3.ziofa".to_string(),
    )
    .await;

    if !args.verbose {
        println!("Note: To view verbose output, pass the \"-v\" flag.");
    }
}
