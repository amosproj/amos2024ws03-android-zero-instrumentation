mod ebpf_info;
mod main_helpers;
mod server;

use std::net::ToSocketAddrs;
use tokio;
#[tokio::main]
async fn main() {
    env_logger::init();

    // apparently needed...
    main_helpers::bump_rlimit();

    server::server_forever().await;
}

