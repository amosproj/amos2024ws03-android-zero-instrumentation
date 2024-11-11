// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

mod server;
mod client;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Options {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Server(server::Options),
    Client(client::Options),
}

fn main() -> Result<()> {
    let Options { command } = Parser::parse();

    match command {
        Command::Server(opts) => server::run(opts),
        Command::Client(opts) => client::run(opts),
    }
}
