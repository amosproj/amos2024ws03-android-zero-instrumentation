use crate::{dummy_functions, main_helpers};
use shared::ziofa::ziofa_server::{Ziofa, ZiofaServer};
use shared::ziofa::{
    EbpfProgram, ListEbpfProgramsResponse, LoadEbpfProgramRequest, LoadEbpfProgramResponse,
};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct ZiofaImpl {}

#[tonic::async_trait]
impl Ziofa for ZiofaImpl {
    async fn list_ebpf_programs(
        &self,
        _: Request<()>,
    ) -> Result<Response<ListEbpfProgramsResponse>, Status> {
        let ret = vec![
            EbpfProgram {
                name: format!("ebpf_program1"),
                description: format!("Test1"),
            },
            EbpfProgram {
                name: format!("ebpf_program2"),
                description: format!("Test2"),
            },
        ];

        let response = ListEbpfProgramsResponse { programs: ret };
        Ok(Response::new(response))
    }

    type LoadEbpfProgramStream = ReceiverStream<Result<LoadEbpfProgramResponse, Status>>;

    async fn load_ebpf_program(
        &self,
        request: Request<LoadEbpfProgramRequest>,
    ) -> Result<Response<Self::LoadEbpfProgramStream>, Status> {
        // get all requested programs
        let programs = request.into_inner().programs;

        // load each requested program
        for program in programs {
            let name = program.name;
            
            if (name == "ebpf_program1") {
                let ret = dummy_functions::ebpf_program1();
                
            } else if (name == "ebpf_program2") {
                let ret = dummy_functions::ebpf_program2();
                
            } else {
                
            }
        }

    }
}

pub async fn serve_forever() {
    let sock_addr = main_helpers::get_socket_addr();
    let server_impl = ZiofaImpl::default();

    // let server_impl = ZiofaImpl::new();
    let service = ZiofaServer::new(server_impl);
    Server::builder()
        .add_service(service)
        .serve(sock_addr)
        .await
        .unwrap();
}
