// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::{client, daemon};
use clap::Parser;
use std::{thread, time};

#[derive(Debug, Parser)]
pub struct Options {
    /// Build and run the release target.
    #[clap(long)]
    release: bool,
}

pub fn test(opts: Options) {
    // spawn daemon
    let shutdown = daemon::run(daemon::Options {
        release: opts.release,
        android: true,
        runner: "sudo -E".to_string(),
        background: true,
    })
    .expect("Daemon should run")
    .expect("Daemon should return shutdown channel");

    println!("Waiting one second for daemon to start.");
    thread::sleep(time::Duration::from_secs(1));

    // spawn client
    client::run(client::Options {
        release: opts.release,
        run_args: vec![],
        android: true,
        test: true,
    })
    .expect("Client should run");
    
    shutdown.send(()).expect("failed to send shutdown signal");
}
