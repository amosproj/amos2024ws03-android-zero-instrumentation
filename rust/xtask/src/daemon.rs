// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::process::{Child, Command};

use anyhow::{bail, Context as _, Result};
use clap::Parser;
use xtask::{android_launch_path, AYA_BUILD_EBPF};

#[derive(Debug, Parser)]
pub struct Options {
    /// Build and run the release target.
    #[clap(long)]
    pub release: bool,
    /// The command used to wrap your application.
    #[clap(short, long, default_value = "sudo -E")]
    pub runner: String,
    /// Arguments to pass to your application.
    #[clap(global = true, last = true)]
    pub run_args: Vec<String>,
    /// Whether to run on Android
    #[clap(long)]
    pub android: bool,
}

/// Build and run the project.
pub fn run(opts: Options, wait_for_exit: bool) -> Result<Option<Child>> {
    let Options {
        release,
        runner,
        run_args,
        android,
    } = opts;

    let android_script = android_launch_path();

    if android {
        // pkill any left backend-daemon process
        let mut pkill = Command::new("adb");
        pkill.args(["shell", "su", "root", "pkill", "backend-daemon"]);
        pkill
            .status()
            .with_context(|| format!("failed to run {pkill:?}"))?;

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
            "target.\"cfg(all())\".runner=\"{} {}\"",
            android_script.display(),
            run_args.join(" ")
        ));
        if release {
            cmd.arg("--release");
        }

        if wait_for_exit {
            let status = cmd
                .status()
                .with_context(|| format!("failed to run {cmd:?}"))?;
            if status.code() != Some(0) {
                bail!("{cmd:?} failed: {status:?}")
            }
            Ok(None)
        } else {
            let child = cmd.spawn().expect("Spawning process should work.");
            Ok(Some(child))
        }
    } else {
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
        if !run_args.is_empty() {
            cmd.arg("--").args(run_args);
        }

        if wait_for_exit {
            let status = cmd
                .status()
                .with_context(|| format!("failed to run {cmd:?}"))?;
            if status.code() != Some(0) {
                bail!("{cmd:?} failed: {status:?}")
            }
            Ok(None)
        } else {
            let child = cmd.spawn().expect("Spawning process should work.");
            Ok(Some(child))
        }
    }
}
