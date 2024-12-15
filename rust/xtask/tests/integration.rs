// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{process::Command, thread, time};

use anyhow::Context as _;
use xtask::{android_launch_path, AYA_BUILD_EBPF};

/// Build, push and run the daemon.
fn run_daemon() {
    let android_script = android_launch_path();

    // pkill any left backend-daemon process
    let mut pkill = Command::new("adb");
    pkill.args(["shell", "su", "root", "pkill", "backend-daemon"]);
    pkill
        .status()
        .with_context(|| format!("failed to run {pkill:?}"))
        .unwrap();

    let mut cmd = Command::new("cargo");
    cmd.env(AYA_BUILD_EBPF, "true");
    cmd.args([
        "ndk",
        "-t",
        "x86_64",
        "run",
        "--package",
        "backend-daemon",
        "--bin",
        "backend-daemon",
        "--config",
    ]);
    cmd.arg(format!(
        "target.\"cfg(all())\".runner=\"{}\"",
        android_script.display(),
    ));
    cmd.spawn().expect("Spawning process should work.");

    println!("Waiting two seconds for daemon to start.");
    thread::sleep(time::Duration::from_secs(2));
}

/// build, push and test client
fn run_client() {
    let android_script = android_launch_path();

    let mut cmd = Command::new("cargo");
    cmd.args([
        "ndk",
        "-t",
        "x86_64",
        "test",
        "--package",
        "client",
        "--config",
    ]);
    cmd.arg(format!(
        "target.\"cfg(all())\".runner=\"{}\"",
        android_script.display(),
    ));
    let status = cmd
        .status()
        .with_context(|| format!("failed to run {cmd:?}"))
        .unwrap();

    assert_eq!(status.code().unwrap(), 0);
}

#[test]
pub fn integration() {
    run_daemon();
    run_client();
}
