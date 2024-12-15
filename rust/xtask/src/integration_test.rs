// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{thread, time};
use clap::Parser;
use crate::{client, daemon};

#[derive(Debug, Parser)]
pub struct Options {
    /// Build and run the release target.
    #[clap(long)]
    release: bool,
}

pub fn test(opts: Options) {
    // spawn daemon
    let mut daemon = daemon::run(
        daemon::Options {
            release: opts.release,
            run_args: vec![],
            android: true,
            runner: "sudo -E".to_string(),
        },
        false,
    )
    .expect("Should work")
    .expect("Should return child handle");

    println!("Waiting two seconds for daemon to start.");
    thread::sleep(time::Duration::from_secs(2));
    if daemon.try_wait().unwrap().is_some() {
        println!("Spawning daemon failed.");
        return;
    }
    // spawn client
    client::run(client::Options {
        release: opts.release,
        run_args: vec![],
        android: true,
        test: true,
    })
    .expect("Client should run");

    // kill daemon
    daemon.kill().unwrap();
}
