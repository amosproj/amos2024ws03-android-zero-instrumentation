// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::{
    config::{Configuration, EbpfEntry},
    ziofa::{process::Cmd, ziofa_client::ZiofaClient},
};

#[tokio::main]
async fn main() {
    let mut client = ZiofaClient::connect("http://[::1]:50051").await.unwrap();
    match client.check_server(()).await {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
    let config = match client.get_configuration(()).await {
        Ok(t) => t.into_inner(),
        Err(e) => {
            println!("Problem loading configuration: {:?}", e);
            Configuration {
                entries: vec![EbpfEntry {
                    hr_name: "Some entry".to_string(),
                    description: "This is the description".to_string(),
                    ebpf_name: "Some entry".to_string(),
                    fn_id: 0,
                    ..Default::default()
                }],
            }
        }
    };
    print!("{:?}", config);
    let response = client.set_configuration(config).await;
    match response {
        Ok(_) => {}
        Err(e) => {
            println!("Error trying to set configuration");
            println!("{:?}", e);
        }
    };

    let processes = client.list_processes(()).await.map(|op| op.into_inner());
    match processes {
        Err(e) => println!("Error getting the process list: {:?}", e),
        Ok(pl) => {
            println!("Processes:");
            println!("pid | ppid | cmdline / comm (fallback) | state");
            for p in pl.processes {
                let cmd = match p.cmd {
                    Some(Cmd::Cmdline(c)) => c.args.join(" "),
                    Some(Cmd::Comm(s)) => s,
                    None => "".to_string(),
                };
                println!("{} | {} | {} | {}", p.pid, p.ppid, cmd, p.state)
            }
        }
    }
}
