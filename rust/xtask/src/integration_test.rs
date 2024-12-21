// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
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
    daemon::run(daemon::Options {
        release: opts.release,
        android: true,
        runner: "sudo -E".to_string(),
        background: false,
        kill: true,
    })
    .expect("Daemon should run");

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

    // kill daemon
    // daemon::pkill(true).expect("Daemon should be killed");
}
