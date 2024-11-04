use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};
use crate::main_helpers;
use shared::ziofa::{ListEbpfProgramsRequest, ListEbpfProgramsResponse, EbpfProgram};
use shared::ziofa::ziofa_server::{Ziofa, ZiofaServer};

#[derive(Default)]
pub struct ZiofaImpl {}

#[tonic::async_trait]
impl Ziofa for ZiofaImpl {
    async fn list_ebpf_programs(
        &self,
        request: Request<ListEbpfProgramsRequest>,
    ) -> Result<Response<ListEbpfProgramsResponse>, Status> {
        let ret = vec![
            EbpfProgram{
                id: 1,
                description: format!("Test1")
            },
            EbpfProgram{
                id: 2,
                description: format!("Test2")
            }
        ];

        let response = ListEbpfProgramsResponse {
            program: ret
        };
        Ok(Response::new(response))
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