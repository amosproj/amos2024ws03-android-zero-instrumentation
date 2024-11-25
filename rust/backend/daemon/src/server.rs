// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, ops::DerefMut, sync::Arc};
use async_broadcast::{broadcast, Receiver, Sender};
use aya::Ebpf;
use aya::programs::{KProbe, ProgramError};
use aya_log::EbpfLogger;
use tokio::join;
use shared::{
    config::Configuration,
    counter::counter_server::CounterServer,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse, ProcessList, SetConfigurationResponse,
    },
};
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use shared::ziofa::Event;
use crate::{
    configuration, constants,
    counter::Counter,
    ebpf_utils::{update_from_config, ProbeID},
    procfs_utils::{list_processes, ProcErrorWrapper},
};
use crate::collector::VfsWriteCollector;

pub struct ZiofaImpl {
    // tx: Option<Sender<Result<EbpfStreamObject, Status>>>,
    probe_id_map: Arc<Mutex<HashMap<String, ProbeID>>>,
    ebpf: Arc<Mutex<Ebpf>>,
    channel: Arc<Channel>
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

impl ZiofaImpl {
    pub fn new(probe_id_map: HashMap<String, ProbeID>, ebpf: Arc<Mutex<Ebpf>>, channel: Arc<Channel>) -> ZiofaImpl {
        ZiofaImpl {
            probe_id_map: Arc::new(Mutex::new(probe_id_map)),
            ebpf,
            channel,
        }
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
        let mut probe_id_map_guard = self.probe_id_map.lock().await;

        // TODO: set config path
        update_from_config(
            ebpf_guard.deref_mut(),
            "ziofa.json",
            probe_id_map_guard.deref_mut(),
        );

        Ok(Response::new(SetConfigurationResponse { response_type: 0 }))
    }

    type InitStreamStream = Receiver<Result<Event, Status>>;

    async fn init_stream(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::InitStreamStream>, Status> {
        Ok(Response::new(self.channel.rx.clone()))
    }
}

fn load_programs(ebpf: &mut Ebpf) -> Result<(), ProgramError> {
    let vfs_write: &mut KProbe = ebpf.program_mut("vfs_write")
        .ok_or(ProgramError::InvalidName { name: "vfs_write".to_owned() })?
        .try_into()?;
    vfs_write.load()?;

    let vfs_write_ret: &mut KProbe = ebpf.program_mut("vfs_write_ret")
        .ok_or(ProgramError::InvalidName { name: "vfs_write_ret".to_owned() })?
        .try_into()?;
    vfs_write_ret.load()?;

    Ok(())
}

pub async fn serve_forever() {
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/backend-ebpf"
    ))).unwrap();

    EbpfLogger::init(&mut ebpf).unwrap();
    load_programs(&mut ebpf).unwrap();

    let mut collector = VfsWriteCollector::from_ebpf(&mut ebpf).unwrap();
    let channel = Arc::new(Channel::new());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let event_tx = channel.tx.clone();

    let probe_id_map = HashMap::new();
    let ebpf = Arc::new(Mutex::new(ebpf));
    let ziofa_server = ZiofaServer::new(ZiofaImpl::new(probe_id_map, ebpf.clone(), channel));
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
