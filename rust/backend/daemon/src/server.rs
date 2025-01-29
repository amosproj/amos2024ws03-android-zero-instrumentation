// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{path::PathBuf, sync::Arc};

use async_broadcast::{broadcast, Receiver, Sender};
use ractor::{call, Actor, ActorRef};
use shared::{
    config::Configuration,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse, Event, GetSymbolOffsetRequest, GetSymbolOffsetResponse,
        GetSymbolsRequest, PidMessage, ProcessList, SearchSymbolsRequest, SearchSymbolsResponse,
        StringResponse, Symbol,
    },
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::{collector::{CollectorSupervisor, CollectorSupervisorArguments}, constants, ebpf_utils::EbpfErrorWrapper, features::Features, filesystem::{ConfigurationStorage, NormalConfigurationStorage}, procfs_utils::{list_processes, ProcErrorWrapper}, registry, symbols::{actors::{GetOffsetRequest, SearchReq, SymbolActor, SymbolActorMsg}, SymbolHandler}};

pub struct ZiofaImpl<C>
where C: ConfigurationStorage {
    features: Arc<Mutex<Features>>,
    channel: Arc<Channel>,
    symbol_handler: Arc<Mutex<SymbolHandler>>,
    configuration_storage: C,
    symbol_actor_ref: ActorRef<SymbolActorMsg>,
}

impl<C> ZiofaImpl<C>
where
    C: ConfigurationStorage,
{
    pub fn new(
        features: Arc<Mutex<Features>>,
        channel: Arc<Channel>,
        symbol_handler: Arc<Mutex<SymbolHandler>>,
        configuration_storage: C,
        symbol_actor_ref: ActorRef<SymbolActorMsg>,
    ) -> ZiofaImpl<C> {
        ZiofaImpl {
            features,
            channel,
            symbol_handler,
            configuration_storage,
            symbol_actor_ref,
        }
    }
}

pub struct Channel {
    tx: Sender<Result<Event, Status>>,
    rx: Receiver<Result<Event, Status>>,
}

impl Channel {
    pub fn new() -> Self {
        let (mut tx, rx) = broadcast(8192);
        tx.set_overflow(true);
        Self { tx, rx }
    }
}

#[tonic::async_trait]
impl<C> Ziofa for ZiofaImpl<C>
where C: ConfigurationStorage {
    async fn check_server(&self, _: Request<()>) -> Result<Response<CheckServerResponse>, Status> {
        // dummy data
        let response = CheckServerResponse {};
        Ok(Response::new(response))
    }

    async fn list_processes(&self, _: Request<()>) -> Result<Response<ProcessList>, Status> {
        let processes = list_processes().map_err(ProcErrorWrapper::from)?;
        Ok(Response::new(processes))
    }

    async fn get_configuration(&self, _: Request<()>) -> Result<Response<Configuration>, Status> {
        //TODO: if ? fails needs valid return value for the function so that the server doesn't crash.
        let res = self.configuration_storage.load(constants::DEV_DEFAULT_FILE_PATH).await;
        let config = res?;
        Ok(Response::new(config))
    }

    async fn set_configuration(
        &self,
        request: Request<Configuration>,
    ) -> Result<Response<()>, Status> {
        let config = request.into_inner();

        self.configuration_storage.save(&config, constants::DEV_DEFAULT_FILE_PATH).await?;

        let mut features_guard = self.features.lock().await;

        // TODO: set config path
        features_guard
            .update_from_config(&config)
            .await
            .map_err(EbpfErrorWrapper::from)?;

        Ok(Response::new(()))
    }

    type InitStreamStream = Receiver<Result<Event, Status>>;

    async fn init_stream(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::InitStreamStream>, Status> {
        Ok(Response::new(self.channel.rx.clone()))
    }

    type GetOdexFilesStream = ReceiverStream<Result<StringResponse, Status>>;

    // TODO: What is this function for?
    async fn get_odex_files(
        &self,
        request: Request<PidMessage>,
    ) -> Result<Response<Self::GetOdexFilesStream>, Status> {
        let pid = request.into_inner().pid;

        let (tx, rx) = mpsc::channel(4);

        let symbol_handler = self.symbol_handler.clone();

        tokio::spawn(async move {
            let mut symbol_handler_guard = symbol_handler.lock().await;
            // TODO Error Handling
            let odex_paths = match symbol_handler_guard.get_paths(pid, ".odex") {
                Ok(paths) => paths,
                Err(e) => {
                    tx.send(Err(Status::from(e)))
                        .await
                        .expect("Error sending Error to client ._.");
                    return;
                }
            };

            for path in odex_paths {
                tx.send(Ok(StringResponse {
                    name: path.to_str().unwrap().to_string(),
                }))
                .await
                .expect("Error sending odex file to client");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetSoFilesStream = ReceiverStream<Result<StringResponse, Status>>;

    async fn get_so_files(
        &self,
        request: Request<PidMessage>,
    ) -> Result<Response<Self::GetSoFilesStream>, Status> {
        let pid = request.into_inner().pid;

        let (tx, rx) = mpsc::channel(4);

        let symbol_handler = self.symbol_handler.clone();

        tokio::spawn(async move {
            let mut symbol_handler_guard = symbol_handler.lock().await;
            // TODO Error Handling
            let odex_paths = match symbol_handler_guard.get_paths(pid, ".so") {
                Ok(paths) => paths,
                Err(e) => {
                    tx.send(Err(Status::from(e)))
                        .await
                        .expect("Error sending Error to client ._.");
                    return;
                }
            };

            for path in odex_paths {
                tx.send(Ok(StringResponse {
                    name: path.to_str().unwrap().to_string(),
                }))
                .await
                .expect("Error sending odex file to client");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetSymbolsStream = ReceiverStream<Result<Symbol, Status>>;

    async fn get_symbols(
        &self,
        request: Request<GetSymbolsRequest>,
    ) -> Result<Response<Self::GetSymbolsStream>, Status> {
        let process_request = request.into_inner();
        let file_path_string = process_request.file_path;
        let file_path = PathBuf::from(file_path_string);

        let (tx, rx) = mpsc::channel(4);

        let symbol_handler = self.symbol_handler.clone();

        tokio::spawn(async move {
            let mut symbol_handler_guard = symbol_handler.lock().await;

            let symbol = match symbol_handler_guard.get_symbols(&file_path).await {
                Ok(symbol) => symbol,
                Err(e) => {
                    tx.send(Err(Status::from(e)))
                        .await
                        .expect("Error sending Error to client ._.");
                    return;
                }
            };
            for (symbol, offset) in symbol.iter() {
                tx.send(Ok(Symbol {
                    method: symbol.to_string(),
                    offset: *offset,
                    path: file_path.to_string_lossy().into_owned(),
                }))
                .await
                .expect("Error sending odex file to client");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn index_symbols(&self, _: Request<()>) -> Result<Response<()>, Status> {
        call!(self.symbol_actor_ref, SymbolActorMsg::ReIndex)
            .map_err(|e| Status::from_error(Box::new(e)))?;
        Ok(Response::new(()))
    }

    async fn search_symbols(
        &self,
        request: Request<SearchSymbolsRequest>,
    ) -> Result<Response<SearchSymbolsResponse>, Status> {
        let SearchSymbolsRequest { query, limit } = request.into_inner();
        let symbols = call!(
            self.symbol_actor_ref,
            SymbolActorMsg::Search,
            SearchReq { query, limit }
        )
        .map_err(|e| Status::from_error(Box::new(e)))??;

        Ok(Response::new(SearchSymbolsResponse { symbols }))
    }

    async fn get_symbol_offset(
        &self,
        request: Request<GetSymbolOffsetRequest>,
    ) -> Result<Response<GetSymbolOffsetResponse>, Status> {
        let GetSymbolOffsetRequest {
            symbol_name,
            library_path,
        } = request.into_inner();
        let offset = call!(
            self.symbol_actor_ref,
            SymbolActorMsg::GetOffset,
            GetOffsetRequest {
                symbol_name,
                library_path
            }
        )
        .map_err(|e| Status::from_error(Box::new(e)))?;

        Ok(Response::new(GetSymbolOffsetResponse { offset }))
    }
}


async fn setup() -> (ActorRef<()>, ZiofaServer<ZiofaImpl<NormalConfigurationStorage>>) {
    let registry = registry::load_and_pin().unwrap();

    let symbol_actor_ref = SymbolActor::spawn().await.unwrap();

    let channel = Channel::new();
    let (collector_ref, _) = Actor::spawn(
        None,
        CollectorSupervisor,
        CollectorSupervisorArguments::new(registry.event.clone(), channel.tx.clone()),
    )
    .await
    .unwrap();
    let channel = Arc::new(channel);

    let features = Features::init_all_features(&registry, symbol_actor_ref.clone());

    let symbol_handler = Arc::new(Mutex::new(SymbolHandler::new()));

    let features = Arc::new(Mutex::new(features));

    let filesystem = NormalConfigurationStorage;

    let ziofa_server = ZiofaServer::new(ZiofaImpl::new(
        features,
        channel,
        symbol_handler,
        filesystem,
        symbol_actor_ref,
    ));

    (collector_ref, ziofa_server)
}

pub async fn serve_forever_socket() {
    let (collector_ref, ziofa_server) = setup().await;

    Server::builder()
        .add_service(ziofa_server)
        .serve(constants::sock_addr())
        .await
        .unwrap();

    collector_ref.stop_and_wait(None, None).await.unwrap();
}

#[cfg(test)]
mod tests {
    use std::io;

    use hyper_util::rt::TokioIo;
    use shared::ziofa::ziofa_client::ZiofaClient;
    use tokio::io::duplex;
    use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
    use tonic::transport::Endpoint;
    use tower::service_fn;

    use super::*;

    #[tokio::test]
    async fn test_in_memory_connection() {
        // We use a channel, this is plays the role of our operating system, that would normally give us a tcp socket when we connect to the server
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Normal setup like in the default case
        let (collector_ref, ziofa_server) = setup().await;

        // We create a new endpoint, the connection url is ignored in the `connect_with_connector` call
        let channel = Endpoint::try_from("http://[::1]:50051")
            .unwrap()
            .connect_with_connector(service_fn({
                move |_| {
                    // We create a new duplex stream, this plays the role of the tcp socket
                    let (left, right) = duplex(64);
                    // We send the duplex stream over the channel, so our server gets the other end of it
                    tx.send(right).unwrap();
                    async move {
                        // TokioIo is just a wrapper to make it compatible with the hyper webserver
                        Ok::<_, io::Error>(TokioIo::new(left))
                    }
                }
            }))
            .await
            .unwrap();

        // We can create multiple clients
        let mut client = ZiofaClient::new(channel.clone());
        let mut other = ZiofaClient::new(channel);

        // stop condition
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();

        let stop = async move {
            let _ = stop_rx.await;
        };

        let server_task = tokio::spawn(async move {
            Server::builder()
                .add_service(ziofa_server)
                // UnboundedReceiverStream is a wrapper to turn a Receiver into a Stream
                // Network operations can fail so it expects a result type, we just wrap it in Ok
                .serve_with_incoming_shutdown(
                    UnboundedReceiverStream::new(rx).map(Ok::<_, io::Error>),
                    stop,
                )
                .await
                .unwrap();
        });

        // We can now call the server as we like
        let _ = client.check_server(()).await.unwrap();
        let _ = other.check_server(()).await.unwrap();

        // gracefully shutdown the server
        stop_tx.send(()).expect("still running");

        // wait for the task/server to be done
        server_task.await.unwrap();

        // stop the collector
        collector_ref.stop_and_wait(None, None).await.unwrap();
    }
}
