// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::collector::MultiCollector;
use crate::symbols_stuff::{some_entry_method, SymbolError};
use crate::{
    configuration, constants,
    counter::Counter,
    ebpf_utils::{EbpfErrorWrapper, State},
    procfs_utils::{list_processes, ProcErrorWrapper},
    symbols_stuff::get_oat_files,
};
use async_broadcast::{broadcast, Receiver, Sender};
use aya::Ebpf;
use aya_log::EbpfLogger;
use shared::ziofa::{Event, OatFileExistsResponse, PidMessage, SomeEntryMethodResponse};
use shared::{
    config::Configuration,
    counter::counter_server::CounterServer,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse, ProcessList, SetConfigurationResponse,
    },
};
use std::path::PathBuf;
use std::{ops::DerefMut, sync::Arc};
use tokio::join;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

pub struct ZiofaImpl {
    // tx: Option<Sender<Result<EbpfStreamObject, Status>>>,
    ebpf: Arc<Mutex<Ebpf>>,
    state: Arc<Mutex<State>>,
    channel: Arc<Channel>,
}

impl ZiofaImpl {
    pub fn new(
        ebpf: Arc<Mutex<Ebpf>>,
        state: Arc<Mutex<State>>,
        channel: Arc<Channel>,
    ) -> ZiofaImpl {
        ZiofaImpl {
            ebpf,
            state,
            channel,
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

    async fn test_oat_file_exists(
        &self,
        pid_message: Request<PidMessage>,
    ) -> Result<Response<OatFileExistsResponse>, Status> {
        let pid = pid_message.into_inner().pid;
        let paths: Vec<String> = get_oat_files(pid)
            .map_err(ProcErrorWrapper::from)?
            .into_iter()
            .map(|path_thing: PathBuf| path_thing.to_str().unwrap().to_string())
            .collect();
        Ok(Response::new(OatFileExistsResponse { paths }))
    }

    async fn test_some_entry_method(
        &self,
        pid_message: Request<PidMessage>,
    ) -> Result<Response<SomeEntryMethodResponse>, Status> {
        let pid = pid_message.into_inner().pid;
        let content_length = some_entry_method(pid).await.map_err(SymbolError::from)?;
        Ok(Response::new(SomeEntryMethodResponse { content_length }))
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

    let ebpf = Arc::new(Mutex::new(ebpf));
    let state = Arc::new(Mutex::new(state));
    let ziofa_server = ZiofaServer::new(ZiofaImpl::new(ebpf.clone(), state, channel));
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
