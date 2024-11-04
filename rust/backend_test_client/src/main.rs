use tonic::Request;
use shared::ziofa::ListEbpfProgramsResponse;
use shared::ziofa::ziofa_client::ZiofaClient;

#[tokio::main]
async fn main() {
    let mut client = ZiofaClient::connect("http://[::1]:50051").await.unwrap();
    let ListEbpfProgramsResponse{ program, .. } = client.list_ebpf_programs(Request::new(())).await.unwrap().into_inner();
    for x in program{
        println!("---");
        println!("id : {}", x.id);
        println!("descr : {}", x.description);
    }
    println!("---");
}
