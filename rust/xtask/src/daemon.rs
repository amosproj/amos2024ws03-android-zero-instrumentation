// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{fs::File, process::Command};

use adb_client::{ADBDeviceExt, ADBServer};
use anyhow::{bail, Context as _, Result};
use clap::Parser;
use xtask::AYA_BUILD_EBPF;

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
    /// pkill the backend-daemon before running a new instance
    #[clap(long)]
    pub kill: bool,
}

/// Build and run the project.
/// wait_for_exit specifies, if the daemon should run synchronously or in the background
/// pkill_before_run specifies if the given binary should be pkill'ed before running
pub fn run(opts: Options) -> Result<()> {
    let Options {
        release,
        runner,
        android,
        background,
        kill,
    } = opts;

    let binary_name = "backend-daemon";
    let binary_path = format!("./target/x86_64-linux-android/debug/{}", binary_name);
    let remote_dir = "/data/local/tmp/";
    let remote_path = format!("{}{}", remote_dir, binary_name);

    ctrlc::set_handler(move || {
        pkill(android).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    // pkill
    if kill {
        pkill(android)?;
    }

    if android {

        // build
        let mut cmd = Command::new("cargo");
        cmd.env(AYA_BUILD_EBPF, "true");
        cmd.args([
            "ndk",
            "-t",
            "x86_64",
            "build",
            "--package",
            "backend-daemon",
            "--bin",
            "backend-daemon",
        ]);
        cmd.arg(format!(
            "target.\"cfg(all())\".runner=\"{} {}\"",
            android_script.display(),
            run_args.join(" ")
        ));

        if release {
            cmd.arg("--release");
        }

        let status = cmd
            .status()
            .with_context(|| format!("failed to run {cmd:?}"))?;
        if status.code() != Some(0) {
            bail!("{cmd:?} failed: {status:?}")
        }

        let mut server = ADBServer::default();
        let mut device = server.get_device().expect("cannot get device");

        // push
        println!(
            "Pushing {} to {} on the Android device...",
            binary_path, remote_path
        );

        let binary = File::open(&binary_path).expect("cannot open binary file");
        device.push(binary, remote_path)?;

        // run
        println!("Running {} on the Android device as root...", binary_name);

        let executable = format!("./{}", binary_name);

        device.shell_command(
            &["cd", remote_dir, "&&", "su", "root", &executable],
            &mut std::io::stdout(),
        )?;
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

        if background {
            let status = cmd
                .status()
                .with_context(|| format!("failed to run {cmd:?}"))?;
            println!("Status: {}", status.code().unwrap());
            if status.code() != Some(0) {
                bail!("{cmd:?} failed: {status:?}")
            }
        } else {
            cmd.spawn().expect("Spawning process should work.");
        }
    }
    Ok(())
}

pub fn pkill(android: bool) -> Result<()> {
    let binary_name = "backend-daemon";
    if android {
        let mut server = ADBServer::default();
        let mut device = server.get_device().expect("cannot get device");

        println!(
            "Running `pkill {}` on the Android device as root...",
            binary_name
        );

        device.shell_command(
            &["su", "root", "pkill", binary_name],
            &mut std::io::stdout(),
        )?;
    } else {
        println!("Running `pkill {}` with root...", binary_name);
        let mut cmd = Command::new("sudo");
        cmd.args(&["pkill", binary_name]);

        cmd.status()
            .with_context(|| format!("failed to run {cmd:?}"))?;
    }
    Ok(())
}
