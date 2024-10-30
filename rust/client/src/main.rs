use std::process::exit;

use shared::counter::counter_client::CounterClient;
use shared::counter::{Count, LoadProgramRequest, LoadProgramResponse};
use tonic::Request;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = CounterClient::connect("http://[::1]:50051").await?;

    let name = "example".to_owned();
    let request = LoadProgramRequest { name: name.clone() };

    let LoadProgramResponse { loaded } = client.load_program(request).await?.into_inner();

    if !loaded {
        println!("Could not load program {name}!");
        exit(1);
    }

    let mut stream = client.server_count(Request::new(()))
        .await?
        .into_inner();

    while let Some(Count { count }) = stream.message().await? {
        println!("Counter: {count}")
    }

    Ok(())
}
