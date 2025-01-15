// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{io::{BufRead, BufReader}, path::PathBuf, process::{Child, Command, Stdio}};

use cargo_metadata::{Artifact, Message, MetadataCommand};

pub const AYA_BUILD_EBPF: &str = "AYA_BUILD_EBPF";

pub fn workspace_root() -> PathBuf {
    let metadata = MetadataCommand::new()
        .no_deps() // You don't need to fetch dependency info here
        .exec()
        .expect("Failed to get cargo metadata");

    metadata.workspace_root.into()
}

pub fn build_runner_client() -> String {
    build_executable(&["build", "--message-format=json", "--package", "runner", "--bin", "runner-client", "--features", "client", "--release"])
}

pub fn build_runner_server() -> String {
    build_executable(&["ndk", "-t", "x86_64", "build", "--message-format=json", "--package", "runner", "--bin", "runner-server", "--features", "server", "--release"])
}

pub fn build_executable(args: &[&str]) -> String {
    let mut cmd = Command::new("cargo");
    cmd.args(args);

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to spawn {cmd:?}: {err}"));

    let Child { stdout, stderr, .. } = &mut child;

    // Trampoline stdout to cargo warnings.
    let stderr = stderr.take().unwrap();
    let stderr = BufReader::new(stderr);
    let stderr = std::thread::spawn(move || {
        for line in stderr.lines() {
            let line = line.unwrap();
            println!("{line}");
        }
    });

    let stdout = stdout.take().unwrap();
    let stdout = BufReader::new(stdout);
    let mut executables = Vec::new();
    for message in Message::parse_stream(stdout) {
        #[allow(clippy::collapsible_match)]
        if let Message::CompilerArtifact(Artifact { executable: Some(exec), .. }) = message.expect("valid JSON") {
            executables.push(exec);
        }
    }

    let status = child
        .wait()
        .unwrap_or_else(|err| panic!("failed to wait for {cmd:?}: {err}"));
    assert_eq!(status.code(), Some(0), "{cmd:?} failed: {status:?}");

    stderr.join().map_err(std::panic::resume_unwind).unwrap();
    
    assert_eq!(executables.len(), 1);
    
    let executable = executables.pop().unwrap();
    
    executable.to_string()
}