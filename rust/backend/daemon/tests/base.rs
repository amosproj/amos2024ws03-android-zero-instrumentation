// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::config::{Configuration, JniReferencesConfig, SysSendmsgConfig, VfsWriteConfig};
use shared::ziofa::{process::Cmd, ziofa_client::ZiofaClient};
use tonic::transport::Channel;

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
    ZiofaClient::connect("http://[::1]:50051")
        .await
        .expect("server should run")
}

#[tokio::test]
async fn check_server() {
    let mut client = setup().await;
    client.check_server(()).await.expect("server should be available");
}

#[tokio::test]
async fn list_processes() {
    let mut client = setup().await;

    let processes = client
        .list_processes(())
        .await
        .expect("processes should be available")
        .into_inner()
        .processes;

    let server_process = processes.iter().find(|process| match &process.cmd {
        Some(Cmd::Cmdline(d)) => {
            if let Some(name) = d.args.first() {
                name.split('/').last() == Some("backend-daemon")
            } else {
                false
            }
        }
        None | Some(_) => false,
    });

    assert!(server_process.is_some());
}



#[tokio::test]
async fn set_get_empty_config() {
    let mut client = setup().await;
    let default_config = Configuration {
        uprobes: vec![],
        vfs_write: Some(VfsWriteConfig {
            entries: std::collections::HashMap::new(),
        }),
        sys_sendmsg: Some(SysSendmsgConfig {
            entries: std::collections::HashMap::new(),
        }),
        jni_references: Some(JniReferencesConfig { pids: vec![] }),
    };
    client
        .set_configuration(default_config.clone())
        .await
        .expect("set_config should work for a config with empty fields");

    let res_config = client
        .get_configuration(())
        .await
        .expect("get_config should work when config was set")
        .into_inner();

    assert_eq!(res_config, default_config);
}

#[tokio::test]
async fn init_stream() {
    let mut client = setup().await;

    let stream = client.init_stream(()).await.expect("init_stream should return a stream").into_inner();
}