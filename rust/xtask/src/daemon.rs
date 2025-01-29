// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, io::Write, process::{Command, Stdio}, sync::mpsc::Sender};

use anyhow::{bail, Context as _, Result};
use clap::Parser;
use runner::HostSpec;
use xtask::{build_runner_client, build_runner_server, AYA_BUILD_EBPF};

#[derive(Debug, Parser)]
pub struct Options {
    /// Build and run the release target.
    #[clap(long)]
    pub release: bool,
    /// The command used to wrap the daemon. Only used when running on host.
    #[clap(short, long, default_value = "sudo -E")]
    pub runner: String,
    /// Arguments to pass to the daemon.
    // #[clap(global = true, last = true)]
    // pub run_args: Vec<String>,
    /// Whether to run on Android
    #[clap(long)]
    pub android: bool,
    /// Don't wait for the daemon to exit
    #[clap(long)]
    pub background: bool,
}

/// Build and run the project.
/// wait_for_exit specifies, if the daemon should run synchronously or in the background
/// pkill_before_run specifies if the given binary should be pkill'ed before running
pub fn run(opts: Options) -> Result<Option<Sender<()>>> {
    let Options {
        release,
        runner,
        android,
        background,
    } = opts;

    if android {
        let android_runner = build_runner_client();
        let android_server = build_runner_server();
        
        let spec = HostSpec {
            args: Vec::default(),
            env: HashMap::from([("RUST_LOG".to_string(), "error".to_string())]),
            root: true,
            runner_path: android_server
        };
        
        // build
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
        ]);
        cmd.arg("--config");
        cmd.arg(format!("target.'cfg(all())'.runner='{} {}'", android_runner, serde_json::to_string(&spec).unwrap()));
        if background {
            cmd.stdin(Stdio::piped());
        }
        if release {
            cmd.arg("--release");
        }

        if !background {
            let status = cmd
                .status()
                .with_context(|| format!("failed to run {cmd:?}"))?;
            if status.code() != Some(0) {
                bail!("{cmd:?} failed: {status:?}")
            }
        } else {
            let mut child = cmd.spawn().expect("Spawning process should work.");
            let mut stdin = child.stdin.take().expect("stdin should be available");
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            std::thread::spawn(move || {
                let _ = rx.recv();
                writeln!(stdin).expect("failed to write to stdin");
                stdin.flush().expect("failed to flush stdin");
                child.wait().expect("failed to wait for child");
            });
            
            return Ok(Some(tx));
        }
    } else {
        // build & run
        let mut cmd = Command::new("cargo");
        cmd.env(AYA_BUILD_EBPF, "true");
        cmd.args([
            "run",
            "--package",
            "backend-daemon",
            "--bin",
            "backend-daemon",
            "--config",
        ]);
        cmd.arg(format!("target.\"cfg(all())\".runner=\"{}\"", runner));
        if release {
            cmd.arg("--release");
        }

        if !background {
            let status = cmd
                .status()
                .with_context(|| format!("failed to run {cmd:?}"))?;
            println!("Status: {}", status.code().unwrap());
            if status.code() != Some(0) {
                bail!("{cmd:?} failed: {status:?}")
            }
        } else {
            panic!("background mode is only for integration tests")
        }
    }
    
    Ok(None)
}
