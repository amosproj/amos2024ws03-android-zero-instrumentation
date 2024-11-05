use crate::{dummy_functions, main_helpers};
use shared::ziofa::ziofa_server::{Ziofa, ZiofaServer};
use shared::ziofa::{EbpfProgram, EbpfProgrammList, EbpfProgrammStatusList, CheckServerResponse, ProcessList, Process, EbpfStreamObject, EbpfProgrammStatus, EbpfLoadingStatus};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Code, Request, Response, Status};
use tonic::Code::Cancelled;

#[derive(Default)]
pub struct ZiofaImpl {
    tx: Option<Sender<Result<EbpfStreamObject, Status>>>
}

#[tonic::async_trait]
impl Ziofa for ZiofaImpl {
    fn check_server (
        &self,
        _: Request(),
    ) -> Result<Response<CheckServerResponse>, Status> {
        // dummy data
        let response = CheckServerResponse {};
        Ok(Response::new(response))
    }

    fn list_ebpf_programs (
        &self,
        _: Request<()>,
    ) -> Result<Response<EbpfProgrammList>, Status> {
        // dummy data
        let response = EbpfProgrammList { programs: vec![
            EbpfProgram {
                name: "ebpf_program1".to_string(),
                description: Some("Test1".to_string()),
            },
            EbpfProgram {
                name: "ebpf_program2".to_string(),
                description: Some("Test2".to_string()),
            },
        ]};
        Ok(Response::new(response))
    }

    fn list_processes (
        &self,
        _: Request<()>,
    ) -> Result<Response<ProcessList>, Status> {
        // dummy data
        let response = ProcessList {processes: vec![
            Process {
                pid: 1,
                package: "systemd".to_string(),
            },
            Process {
                pid: 1741231,
                package: "com.example.org".to_string(),
            },
        ]};
        Ok(Response::new(response))
    }

    type InitStreamStream = ReceiverStream<Result<EbpfStreamObject, Status>>;
    fn init_stream(&self, request: Request<()>) -> Result<Response<Self::InitStreamStream>, Status> {
        let (tx, rx) = mpsc::channel(1);

        Ok(Response::new(Self::InitStreamStream::new(rx)))
    }

    async fn load_ebpf_programs (
        &self,
        request: Request<EbpfProgrammList>,
    ) -> Result<Response<EbpfProgrammStatusList>, Status> {
        match self.tx {
            None => Err(Status::new(Code::Cancelled, "wrong")),
            Ok(tx) => {
                // get all requested programs
                let programs = request.into_inner().programs;
                let mut result = EbpfProgrammStatusList {
                    programs: Vec::new(),
                };

                // TODO: Try to attach requested functions
                // dummy functions: load each requested program
                for program in programs {
                    let name = program.name.clone();
                    let mut status: EbpfLoadingStatus;
                    let mut error_message: Option<String> = None;
                    if "ebpf_program1" == name {
                        status = dummy_functions::ebpf_program1(tx.clone()).await;
                    } else if name == "ebpf_program2" {
                        status = dummy_functions::ebpf_program2(tx.clone()).await;
                    } else {
                        status = EbpfLoadingStatus::Error;
                        error_message = "Invalid program specified.";
                    }

                    result.programs.push(EbpfProgrammStatus {
                        program,
                        status,
                        error_message
                    });
                }
                Ok(Response::new(result))
            }
        }
    }

    fn unload_ebpf_programms(&self, request: Request<EbpfProgrammList>) -> Result<Response<EbpfProgrammStatusList>, Status> {
        todo!();
    }
}

pub async fn serve_forever() {
    let sock_addr = main_helpers::get_socket_addr();
    let server_impl = ZiofaImpl::default();

    let service = ZiofaServer::new(server_impl);
    Server::builder()
        .add_service(service)
        .serve(sock_addr)
        .await
        .unwrap();
}
