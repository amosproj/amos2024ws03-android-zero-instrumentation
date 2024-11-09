mod helpers;
mod server;
mod configuration;
mod constants;

use tokio;


#[tokio::main]
async fn main() {
    env_logger::init();

    // apparently needed...
    helpers::bump_rlimit();

    server::serve_forever().await;
}

