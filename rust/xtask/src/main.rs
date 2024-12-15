// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
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
    IntegrationTest,
}

fn main() -> Result<()> {
    let Options { command } = Parser::parse();

    match command {
        Command::Daemon(opts) => daemon::run(opts),
        Command::Client(opts) => client::run(opts),
        Command::IntegrationTest => {
            integration_test::test();
            Ok(())
        }
    }
}
