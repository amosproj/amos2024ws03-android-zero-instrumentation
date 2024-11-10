use shared::config::Configuration;
use shared::ziofa::ziofa_client::ZiofaClient;

#[tokio::main]
async fn main() {
    let mut client = ZiofaClient::connect("http://[::1]:50051").await.unwrap();
    match client.check_server(()).await {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
    let config = match client.get_configuration(()).await {
        Ok(T) => T.into_inner(),
        Err(e) => {
            println!("Problem loading configuration: {:?}", e);
            Configuration{
                entries: vec![],
            }
        }
    };
    print!("{:?}", config);
    let response = client.set_configuration(config).await;
    match response {
        Ok(_) => {}
        Err(e) => {
            println!("Error trying to set configuration");
            println!("{:?}", e);
        }
    }
}
