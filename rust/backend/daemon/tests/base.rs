// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::ziofa::{process::Cmd, ziofa_client::ZiofaClient};
use tonic::transport::Channel;
use shared::config::{Configuration, EbpfEntry};

/* 
static RUNNING: Mutex<bool> = Mutex::new(false);

async fn ensure_running() {
    if *RUNNING.lock().unwrap() == false {
        tokio::spawn(async move {
            run_server().await;
        });
        sleep(Duration::from_secs(1)).await;
    }
}
*/

async fn setup() -> ZiofaClient<Channel> {
    //ensure_running().await;
    ZiofaClient::connect("http://[::1]:50051").await.expect("server should run")
}


#[tokio::test]
async fn list_processes() {
    let mut client = setup().await;

    let processes = client.list_processes(())
        .await
        .expect("should work")
        .into_inner()
        .processes;

    let server_process = processes.iter().find(|process| {
        match &process.cmd {
            Some(Cmd::Cmdline(d)) => {
                if let Some(name) = d.args.first() {
                    name.split('/').last() == Some("backend-daemon")
                } else {
                    false
                }
            }
            None | Some(_) => false,
        }
    });

    assert!(server_process.is_some());
}

#[tokio::test]
async fn check_server() {
    let mut client = setup().await;
    client.check_server(())
    .await
    .expect("should work");
}

#[tokio::test]
async fn set_get_configuration() {
    let mut client = setup().await;
    let default_config: Vec<EbpfEntry> = vec![
        EbpfEntry {
            hr_name: "vfs_write".to_string(),
            description: "vfs_write".to_string(),
            ebpf_name: "vfs_write".to_string(),
            fn_id: 0,
            hook: "vfs_write".to_string(),
            attach: false,
            uprobe_info: None,
        },
        EbpfEntry {
            hr_name: "vfs_write_ret".to_string(),
            description: "vfs_write_ret".to_string(),
            ebpf_name: "vfs_write_ret".to_string(),
            fn_id: 0,
            hook: "vfs_write".to_string(),
            attach: false,
            uprobe_info: None,
        },
    ];

    assert_eq!(client.set_configuration(Configuration { entries: default_config.clone() })
    .await
    .expect("should work")
    .into_inner()
    .response_type, 0);

    let res_config = client.get_configuration(())
    .await
    .expect("should work")
    .into_inner()
    .entries;

    assert_eq!(res_config, default_config);
}