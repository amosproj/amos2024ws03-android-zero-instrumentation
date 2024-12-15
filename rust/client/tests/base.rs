// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use client::Client;
use shared::config::{Configuration, SysSendmsgConfig, VfsWriteConfig};
use shared::ziofa::process::Cmd;

// client tests assume daemon is running!
async fn setup() -> Client {
    Client::connect("http://[::1]:50051".to_string())
        .await
        .expect("Daemon must be reachable.")
}

#[tokio::test]
async fn list_processes() {
    let mut client = setup().await;

    let processes = client.list_processes().await.expect("should work");

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
async fn check_server() {
    let mut client = setup().await;
    client.check_server().await.expect("should work");
}

#[tokio::test]
async fn set_get_configuration() {
    let mut client = setup().await;

    let default_config = Configuration {
        uprobes: vec![],
        vfs_write: Some(VfsWriteConfig {
            entries: std::collections::HashMap::new(),
        }),
        sys_sendmsg: Some(SysSendmsgConfig {
            entries: std::collections::HashMap::new(),
        }),
        // jni_references: Some(JniReferencesConfig { pids: vec![] }),
        jni_references: None,
    };

    client
        .set_configuration(default_config.clone())
        .await
        .expect("should work");

    let res_config = client
        .get_configuration()
        .await
        .expect("should work");

    assert_eq!(res_config, default_config);
}

// TODO sendmsg
// TODO get odex files
// TODO get so files
// TODO get symbols