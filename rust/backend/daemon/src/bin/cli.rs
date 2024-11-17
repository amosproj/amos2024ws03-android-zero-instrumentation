// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::{
    config::{Configuration, EbpfEntry},
    ziofa::{process::Cmd, ziofa_client::ZiofaClient},
};
use std::env;
use tonic::transport::Channel;

struct Flags {
    verbose: bool,
    addr: String,
}

fn print_usage() {
    println!("Usage: backend-daemon-cli [--addr http://<addr>:<port>] [-v]");
    println!("");
    println!("Note: When using cargo run, you have to put two dashes in front of the arguments:");
    println!("cargo run --bin backend-daemon-cli -- [--addr http://<addr>:<port>] [-v]");
    println!("");
    println!("--addr        set the address the client binds to. If ommitted binds to \"http://127.0.0.1:50051\"");
    println!("-v            verbose");
    println!("-h, --help    view this message");
}

// returns if program flow should continue
fn parse_args(args: &mut Vec<String>, flags: &mut Flags) -> bool {
    let mut it = args.iter_mut();
    it.next(); // skip program name
    let mut cont = true;

    while let Some(s) = it.next() {
        match s.as_str() {
            "backend-daemon-cli" => (),
            "--addr" => match it.next() {
                Some(a) => flags.addr = a.to_string(),
                None => {
                    print_usage();
                    cont = false;
                }
            },
            "-v" => flags.verbose = true,
            "-h" | "--help" | &_ => {
                print_usage();
                cont = false;
            }
        }
    }

    cont
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
    flags: &Flags,
) -> Vec<EbpfEntry> {
    println!("TEST get_configuration");
    let config = match client.get_configuration(()).await {
        Ok(t) => {
            let res = t.into_inner().entries;
            println!("SUCCESS");
            if flags.verbose {
                for i in 0..res.len() {
                    println!("Entry {}: {:?}", i, res[i]);
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

async fn test_list_processes(client: &mut ZiofaClient<Channel>, flags: &Flags) {
    println!("TEST list_processes");
    match client.list_processes(()).await {
        Ok(t) => {
            let res = t.into_inner().processes;
            println!("SUCCESS");
            if flags.verbose {
                for i in 0..res.len() {
                    println!("Process {}: {:?}", i, res[i]);
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
    let mut flags = Flags {
        addr: "http://127.0.0.1:50051".to_string(),
        verbose: false,
    };
    let mut args: Vec<String> = env::args().collect();
    if !parse_args(&mut args, &mut flags) {
        return;
    }

    println!("Trying to connect to server: \"{}\"", flags.addr.clone());
    let mut client = ZiofaClient::connect(flags.addr.clone()).await.unwrap();

    test_check_server(&mut client).await;
    let config = test_get_configuration(&mut client, &flags).await;
    test_set_configuration(&mut client, config).await;
    test_list_processes(&mut client, &flags).await;

    if !flags.verbose {
        println!("Note: To view verbose output, pass the \"-v\" flag.");
    }
}
