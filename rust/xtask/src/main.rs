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
