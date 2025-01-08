// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
//
// SPDX-License-Identifier: MIT

use client::Client;
use shared::config::{Configuration, SysSendmsgConfig, VfsWriteConfig, SysSigquitConfig};
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

    let processes = client.list_processes().await.expect("processes should be available");

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
        // jni_references: Some(JniReferencesConfig { pids: vec![] }),
        jni_references: None,
        sys_sigquit: Some(SysSigquitConfig { pids: vec![] }),
    };

    client
        .set_configuration(default_config.clone())
        .await
        .expect("set_config should work for a config with empty fields");

    let res_config = client
        .get_configuration()
        .await
        .expect("get_config should work when config was set");

    assert_eq!(res_config, default_config);
}



#[tokio::test]
async fn init_stream() {
    let mut client = setup().await;

    let _ = client.init_stream().await.expect("init_stream should return a stream");
}