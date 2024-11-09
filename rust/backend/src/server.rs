use crate::{configuration, constants};
use shared::config::Configuration as ProtoConfig;
use shared::ziofa::ziofa_server::{Ziofa, ZiofaServer};
use shared::ziofa::{
    CheckServerResponse, EbpfStreamObject, Process, ProcessList, SetConfigurationResponse,
};
use configuration::Configuration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct ZiofaImpl {
    tx: Option<Sender<Result<EbpfStreamObject, Status>>>,
}

#[tonic::async_trait]
impl Ziofa for ZiofaImpl {
    fn check_server(&self, _: Request()) -> Result<Response<CheckServerResponse>, Status> {
        // dummy data
        let response = CheckServerResponse {};
        Ok(Response::new(response))
    }

    fn list_processes(&self, _: Request<()>) -> Result<Response<ProcessList>, Status> {
        // dummy data
        let response = ProcessList {
            processes: vec![
                Process {
                    pid: 1,
                    package: "systemd".to_string(),
                },
                Process {
                    pid: 1741231,
                    package: "com.example.org".to_string(),
                },
            ],
        };
        Ok(Response::new(response))
    }

    async fn get_configuration(
        &self,
        _: Request<()>,
    ) -> Result<Response<ProtoConfig>, Status> {
        let config = Configuration::load_from_file(constants::DEV_DEFAULT_FILE_PATH)?;
        Ok(Response::new(ProtoConfig::try_from(config).unwrap()))
    }

    async fn set_configuration(
        &self,
        request: Request<ProtoConfig>,
    ) -> Result<Response<SetConfigurationResponse>, Status> {
        let conf = Configuration::try_from(request.into_inner()).unwrap();

        // TODO: Implement function 'validate'
        // TODO: if ? failes needs valid return value for the function so that the server doesn't fail
        conf.validate()?;
        conf.save_to_file(constants::DEV_DEFAULT_FILE_PATH)?;
        Ok(Response::new(SetConfigurationResponse{ response_type: 0}))
    }

    type InitStreamStream = ReceiverStream<Result<EbpfStreamObject, Status>>;
    fn init_stream(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::InitStreamStream>, Status> {
        let (tx, rx) = mpsc::channel(1);

        Ok(Response::new(Self::InitStreamStream::new(rx)))
    }
}

pub async fn serve_forever() {
    let service = ZiofaServer::new(ZiofaImpl::default());
    Server::builder()
        .add_service(service)
        .serve(constants::sock_addr())
        .await
        .unwrap();
}
