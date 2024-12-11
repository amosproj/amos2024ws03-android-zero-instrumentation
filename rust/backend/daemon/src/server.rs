// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::registry;
use crate::symbols::SymbolHandler;
use crate::{
    configuration, constants,
    ebpf_utils::EbpfErrorWrapper,
    procfs_utils::{list_processes, ProcErrorWrapper},
    features::Features,
};
use async_broadcast::{broadcast, Receiver, Sender};
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

pub struct ZiofaImpl {
    features: Arc<Mutex<Features>>,
    channel: Arc<Channel>,
    symbol_handler: Arc<Mutex<SymbolHandler>>,
}

impl ZiofaImpl {
    pub fn new(
        features: Arc<Mutex<Features>>,
        channel: Arc<Channel>,
        symbol_handler: Arc<Mutex<SymbolHandler>>,
    ) -> ZiofaImpl {
        ZiofaImpl {
            features,
            channel,
            symbol_handler,
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
impl Ziofa for ZiofaImpl {
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
        let config = configuration::load_from_file(constants::DEV_DEFAULT_FILE_PATH)?;
        Ok(Response::new(config))
    }

    async fn set_configuration(
        &self,
        request: Request<Configuration>,
    ) -> Result<Response<()>, Status> {
        let config = request.into_inner();

        configuration::save_to_file(&config, constants::DEV_DEFAULT_FILE_PATH)?;

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
            let odex_paths = match symbol_handler_guard.get_odex_paths(pid) {
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
        let odex_file_path_string = process_request.odex_file_path;
        let odex_file_path = PathBuf::from(odex_file_path_string);

        let (tx, rx) = mpsc::channel(4);

        let symbol_handler = self.symbol_handler.clone();

        tokio::spawn(async move {
            let mut symbol_handler_guard = symbol_handler.lock().await;

            let symbol = match symbol_handler_guard.get_symbols(&odex_file_path).await {
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
}

pub async fn serve_forever() {
    let registry = registry::load_and_pin().unwrap();
    let channel = Arc::new(Channel::new());

    let features = Features::init_all_features(&registry);

    let symbol_handler = Arc::new(Mutex::new(SymbolHandler::new()));

    let features = Arc::new(Mutex::new(features));
    let ziofa_server = ZiofaServer::new(ZiofaImpl::new(features, channel, symbol_handler));

    Server::builder()
        .add_service(ziofa_server)
        .serve(constants::sock_addr())
        .await
        .unwrap();
}
