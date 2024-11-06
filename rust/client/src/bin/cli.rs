use client::Client;
use tokio_stream::StreamExt;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let mut client = Client::connect("http://[::1]:50051".to_owned()).await?;
    client.load_program("example".to_owned()).await?;

    let mut stream = client.server_count().await?;

    while let Some(count) = stream.next().await {
        println!("{}", count?)
    }
    
    Ok(())
}