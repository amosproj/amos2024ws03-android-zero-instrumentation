
use shared::ziofa::{process::Cmd, ziofa_client::ZiofaClient};
use tonic::transport::Channel;

/* 
static RUNNING: Mutex<bool> = Mutex::new(false);

async fn ensure_running() {
    if *RUNNING.lock().unwrap() == false {
        tokio::spawn(async move {
            run_server().await;
        });
        sleep(Duration::from_secs(1)).await;
    }
}
*/

async fn setup() -> ZiofaClient<Channel> {
    //ensure_running().await;
    ZiofaClient::connect("http://[::1]:50051").await.expect("server should run")
}


#[tokio::test]
async fn test_list_processes() {
    let mut client = setup().await;

    let processes = client.list_processes(())
        .await
        .expect("should work")
        .into_inner()
        .processes;

    let server_process = processes.iter().find(|process| {
        match &process.cmd {
            Some(Cmd::Cmdline(d)) => {
                if let Some(name) = d.args.first() {
                    name.split('/').last() == Some("backend-daemon")
                } else {
                    false
                }
            }
            None | Some(_) => false,
        }
    });

    assert!(server_process.is_some());
}