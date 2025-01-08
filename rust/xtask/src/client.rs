// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, process::Command};

use anyhow::{bail, Context as _, Result};
use clap::Parser;
use runner::HostSpec;
use xtask::{build_runner_client, build_runner_server, AYA_BUILD_EBPF};

#[derive(Debug, Parser)]
pub struct Options {
    /// Build and run the release target.
    #[clap(long)]
    pub release: bool,
    /// Arguments to pass to your application.
    #[clap(global = true, last = true)]
    pub run_args: Vec<String>,
    /// Whether to run on Android
    #[clap(long)]
    pub android: bool,
    /// Run the client tests
    #[clap(long)]
    pub test: bool,
}

/// Build and run the project.
pub fn run(opts: Options) -> Result<()> {
    let Options {
        release,
        run_args,
        android,
        test,
    } = opts;


    if android {
        let android_runner = build_runner_client();
        let android_server = build_runner_server();
        
        let spec = HostSpec {
            args: run_args,
            env: HashMap::from([("RUST_LOG".to_string(), "error".to_string())]),
            root: false,
            runner_path: android_server
        };

        let mut cmd = Command::new("cargo");
        cmd.env(AYA_BUILD_EBPF, "true");
        cmd.args(["ndk", "-t", "x86_64"]);
        if test {
            cmd.arg("test");
        } else {
            cmd.args(["run", "--bin", "client", "--features", "cli"]);
        }
        cmd.args([
            "--package",
            "client",
            "--config",
            &format!(
                "target.'cfg(all())'.runner='{} {}'",
                android_runner,
                serde_json::to_string(&spec).unwrap()
            ),
        ]);
        let status = cmd
            .status()
            .with_context(|| format!("failed to run {cmd:?}"))?;
        if status.code() != Some(0) {
            bail!("{cmd:?} failed: {status:?}")
        }
    } else {
        let mut cmd = Command::new("cargo");
        cmd.env(AYA_BUILD_EBPF, "true");
        if test {
            cmd.arg("test");
        } else {
            cmd.args([
                "run",
                "--package",
                "client",
                "--bin",
                "client",
                "--features",
                "cli",
            ]);
        }
        if release {
            cmd.arg("--release");
        }
        if !run_args.is_empty() {
            cmd.arg("--").args(run_args);
        }
        let status = cmd
            .status()
            .with_context(|| format!("failed to run {cmd:?}"))?;
        if status.code() != Some(0) {
            bail!("{cmd:?} failed: {status:?}")
        }
    }

    Ok(())
}
