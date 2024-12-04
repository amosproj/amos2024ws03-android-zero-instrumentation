// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::collector::MultiCollector;
use crate::symbols::{self, get_symbol_offset_for_function_of_process, SymbolHandler};
use crate::{
    configuration, constants,
    counter::Counter,
    ebpf_utils::{EbpfErrorWrapper, State},
    procfs_utils::{list_processes, ProcErrorWrapper},
};
use async_broadcast::{broadcast, Receiver, Sender};
use aya::Ebpf;
use aya_log::EbpfLogger;
use shared::ziofa::{
    Event, GetAddressOfSymbolRequest, GetAddressOfSymbolResponse, GetSymbolsOfProcessRequest,
    PidMessage, StringResponse, Symbol,
};
use shared::{
    config::Configuration,
    counter::counter_server::CounterServer,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse, ProcessList, SetConfigurationResponse,
    },
};
use std::{ops::DerefMut, sync::Arc};
use tokio::join;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

pub struct ZiofaImpl {
    // tx: Option<Sender<Result<EbpfStreamObject, Status>>>,
    ebpf: Arc<Mutex<Ebpf>>,
    state: Arc<Mutex<State>>,
    channel: Arc<Channel>,
    symbol_handler: Arc<Mutex<SymbolHandler>>,
}

impl ZiofaImpl {
    pub fn new(
        ebpf: Arc<Mutex<Ebpf>>,
        state: Arc<Mutex<State>>,
        channel: Arc<Channel>,
        symbol_handler: Arc<Mutex<SymbolHandler>>,
    ) -> ZiofaImpl {
        ZiofaImpl {
            ebpf,
            state,
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
    ) -> Result<Response<SetConfigurationResponse>, Status> {
        let config = request.into_inner();

        // TODO: Implement function 'validate'
        // TODO: if ? fails needs valid return value for the function so that the server doesn't fail
        configuration::validate(&config)?;
        configuration::save_to_file(&config, constants::DEV_DEFAULT_FILE_PATH)?;

        let mut ebpf_guard = self.ebpf.lock().await;
        let mut state_guard = self.state.lock().await;

        // TODO: set config path
        state_guard
            .update_from_config(ebpf_guard.deref_mut(), &config)
            .map_err(EbpfErrorWrapper::from)?;

        Ok(Response::new(SetConfigurationResponse { response_type: 0 }))
    }

    type InitStreamStream = Receiver<Result<Event, Status>>;

    async fn init_stream(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::InitStreamStream>, Status> {
        Ok(Response::new(self.channel.rx.clone()))
    }

    type GetOdexFilesStream = ReceiverStream<Result<StringResponse, Status>>;
    async fn get_odex_files(
        &self,
        request: Request<PidMessage>,
    ) -> Result<Response<Self::GetOdexFilesStream>, Status> {
        let pid = request.into_inner().pid;
        let symbol_handler_guard = self.symbol_handler.lock().await;
        let odex_paths = symbol_handler_guard.get_odex_paths(pid)?.clone();

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for path in odex_paths {
                tx.send(Ok(StringResponse {
                    name: path.to_str().unwrap().to_string(),
                }));
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetSymbolsOfProcessStream = ReceiverStream<Result<Symbol, Status>>;

    async fn get_symbols_of_process(
        &self,
        request: Request<GetSymbolsOfProcessRequest>,
    ) -> Result<Response<Self::GetSymbolsOfProcessStream>, Status> {
        let process = request.into_inner();
        let pid = process.pid;
        let package_name = process.package_name;

        Ok(Response::new(SymbolList {
            names: symbols::get_symbols_of_pid(pid, &package_name).await?,
        }))
    }

    async fn get_address_of_symbol(
        &self,
        request: Request<GetAddressOfSymbolRequest>,
    ) -> Result<Response<GetAddressOfSymbolResponse>, Status> {
        let request_inner = request.into_inner();
        let symbol_name = request_inner.name;
        let pid = request_inner.pid;
        let package_name = request_inner.package_name;

        let offset =
            get_symbol_offset_for_function_of_process(pid, &package_name, &symbol_name).await?;

        Ok(Response::new(GetAddressOfSymbolResponse { offset }))
    }
}

pub async fn serve_forever() {
    let mut ebpf = Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/backend-ebpf"
    )))
    .unwrap();

    EbpfLogger::init(&mut ebpf).unwrap();

    let mut collector = MultiCollector::from_ebpf(&mut ebpf).unwrap();
    let channel = Arc::new(Channel::new());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let event_tx = channel.tx.clone();

    let mut state = State::new();
    state.init(&mut ebpf).expect("should work");

    let mut symbol_handler = Arc::new(Mutex::new(SymbolHandler::new()));

    let ebpf = Arc::new(Mutex::new(ebpf));
    let state = Arc::new(Mutex::new(state));
    let ziofa_server =
        ZiofaServer::new(ZiofaImpl::new(ebpf.clone(), state, channel, symbol_handler));
    let counter_server = CounterServer::new(Counter::new(ebpf).await);

    let serve = async move {
        Server::builder()
            .add_service(ziofa_server)
            .add_service(counter_server)
            .serve(constants::sock_addr())
            .await
            .unwrap();
        shutdown_tx.send(()).unwrap();
    };

    let collect = async move {
        collector.collect(event_tx, shutdown_rx).await.unwrap();
    };

    let (_, _) = join!(serve, collect);
}
