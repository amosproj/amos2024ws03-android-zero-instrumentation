// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{process::{Child, Command}, thread, time};

use anyhow::{bail, Context as _};
use xtask::{android_launch_path, AYA_BUILD_EBPF};

/// Build, push and run the daemon.
fn run_daemon() -> Child {
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
    let child = cmd.spawn().expect("Spawning process should work.");

    println!("Waiting two seconds for daemon to start.");
    thread::sleep(time::Duration::from_secs(2));

    child
}

/// build, push and test client
fn run_client() -> Result<(), anyhow::Error> {
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
        .with_context(|| format!("failed to run {cmd:?}"))?;

    if status.code() != Some(0) {
        bail!(format!("failed to run {cmd:?}"));
    }

    Ok(())
}

pub fn test() {
    let mut child = run_daemon();

    run_client().ok();

    child.kill().unwrap();
}
