// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::collector::{CollectorSupervisor, CollectorSupervisorArguments};
use crate::filesystem::{Filesystem, NormalFilesystem};
use crate::registry;
use crate::symbols::actors::{SymbolActor, SymbolActorMsg};
use crate::symbols::SymbolHandler;
use crate::{
    constants,
    ebpf_utils::EbpfErrorWrapper,
    procfs_utils::{list_processes, ProcErrorWrapper},
    features::Features,
};
use async_broadcast::{broadcast, Receiver, Sender};
use ractor::{call, Actor, ActorRef};
use shared::ziofa::{Event, GetSymbolsRequest, PidMessage, StringResponse, Symbol};
use shared::{
    config::Configuration,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse, ProcessList,
    },
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

pub struct ZiofaImpl<F>
where F: Filesystem {
    features: Arc<Mutex<Features>>,
    channel: Arc<Channel>,
    symbol_handler: Arc<Mutex<SymbolHandler>>,
    filesystem: F,
    symbol_actor_ref: ActorRef<SymbolActorMsg>,
}

impl<F> ZiofaImpl<F> 
where F: Filesystem {
    pub fn new(
        features: Arc<Mutex<Features>>,
        channel: Arc<Channel>,
        symbol_handler: Arc<Mutex<SymbolHandler>>,
        filesystem: F,
        symbol_actor_ref: ActorRef<SymbolActorMsg>,
    ) -> ZiofaImpl<F> {
        ZiofaImpl {
            features,
            channel,
            symbol_handler,
            filesystem,
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
        let (tx, rx) = broadcast(8192);
        Self { tx, rx }
    }
}

#[tonic::async_trait]
impl<F> Ziofa for ZiofaImpl<F>
where F: Filesystem {
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
        let config = self.filesystem.load(constants::DEV_DEFAULT_FILE_PATH)?;
        Ok(Response::new(config))
    }

    async fn set_configuration(
        &self,
        request: Request<Configuration>,
    ) -> Result<Response<()>, Status> {
        let config = request.into_inner();

        self.filesystem.save(&config, constants::DEV_DEFAULT_FILE_PATH)?;

        let mut features_guard = self.features.lock().await;

        // TODO: set config path
        features_guard
            .update_from_config(&config)
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
                }))
                .await
                .expect("Error sending odex file to client");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
    
    async fn index_symbols(&self, request: Request<()>) -> Result<Response<()>, Status> {
        call!(self.symbol_actor_ref, SymbolActorMsg::ReIndex).map_err(|e| Status::from_error(Box::new(e)))?;
        Ok(Response::new(()))
    }
}



pub async fn serve_forever() {
    let registry = registry::load_and_pin().unwrap();
    
    let symbol_actor_ref = SymbolActor::spawn().await.unwrap();

    let channel = Channel::new();
    let (collector_ref, _) = Actor::spawn(
        None, 
        CollectorSupervisor, 
        CollectorSupervisorArguments::new(registry.event.clone(), 
        channel.tx.clone())
    ).await.unwrap();
    let channel = Arc::new(channel);

    let features = Features::init_all_features(&registry);

    let symbol_handler = Arc::new(Mutex::new(SymbolHandler::new()));

    let features = Arc::new(Mutex::new(features));

    let filesystem = NormalFilesystem;

    let ziofa_server = ZiofaServer::new(ZiofaImpl::new(features, channel, symbol_handler, filesystem, symbol_actor_ref));

    Server::builder()
        .add_service(ziofa_server)
        .serve(constants::sock_addr())
        .await
        .unwrap();
    
    collector_ref.stop_and_wait(None, None).await.unwrap();
}
