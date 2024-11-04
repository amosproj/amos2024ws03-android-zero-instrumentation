mod main_helpers;
mod server;

use tokio;
#[tokio::main]
async fn main() {
    env_logger::init();

    // apparently needed...
    main_helpers::bump_rlimit();

    server::serve_forever().await;
}

