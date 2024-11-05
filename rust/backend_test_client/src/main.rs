use tonic::Request;
use shared::ziofa::{ListEbpfProgramsResponse, LoadEbpfProgramRequest, LoadEbpfProgramResponse};
use shared::ziofa::ziofa_client::ZiofaClient;

#[tokio::main]
async fn main() {
    let mut client = ZiofaClient::connect("http://[::1]:50051").await.unwrap();
    let ListEbpfProgramsResponse { programs } = client.list_ebpf_programs(Request::new(())).await.unwrap().into_inner();

    println!("Possible Programs");
    for x in programs.clone() {
        println!("---");
        println!("name : {}", x.name);
        println!("description : {}", x.description);
    }
    println!(" ============== ");

    println!("Test executing the first one");
    let mut result_stream = client.load_ebpf_program(
        LoadEbpfProgramRequest {
            programs: vec![programs[0].clone()]
        }
    )
        .await.unwrap().into_inner();
    let mut counter = 0;

    while let Ok(
        Some(LoadEbpfProgramResponse {
            pr1,
            pr2,
            ..
        })) = result_stream.message().await {
        println!("------");
        let pr1_str = match pr1{
            Some(x) => x.time,
            None => "nothing".parse().unwrap()
        };
        println!("pr1: {}", pr1_str);
        let pr2_str = match pr2{
            Some(x) => x.time,
            None => "nothing".parse().unwrap()
        };
        println!("pr2: {}", pr2_str);
        counter += 1;
        if counter > 20 {
            break;
        }
    }
    println!(" ============== ");
    println!("Test executing the first and second one");
    let load_ebpf_program_request = LoadEbpfProgramRequest {
        programs: vec![programs[0].clone(), programs[1].clone()]
    };
    let mut result_stream = client.load_ebpf_program(
        load_ebpf_program_request
    )
        .await.unwrap().into_inner();
    let mut counter = 0;
    while let Ok(
        Some(
            LoadEbpfProgramResponse {
                pr1,
                pr2,
                ..
            }
        )) = result_stream.message().await {
        println!("------");
        let pr1_str = match pr1{
            Some(x) => x.time,
            None => "nothing".parse().unwrap()
        };
        println!("pr2: {}", pr1_str);
        let pr2_str = match pr2{
            Some(x) => x.time,
            None => "nothing".parse().unwrap()
        };
        println!("pr2: {}", pr2_str);
        counter += 1;
        if counter > 20 {
            break;
        }
    }
}
