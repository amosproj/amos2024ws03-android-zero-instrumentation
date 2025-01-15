// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

mod client;
mod daemon;
mod integration_test;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Options {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Daemon(daemon::Options),
    Client(client::Options),
    IntegrationTest(integration_test::Options),
}

fn main() -> Result<()> {
    let Options { command } = Parser::parse();

    match command {
        Command::Daemon(opts) => {
            daemon::run(opts)?;
            Ok(())
        }
        Command::Client(opts) => client::run(opts),
        Command::IntegrationTest(opts) => {
            integration_test::test(opts);
            Ok(())
        }
    }
}
