// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::constants::BUFFER_CAPACITY;
use crate::utils::write_buffer_to_pipe;
use crate::{
    configuration, constants,
    counter::Counter,
    ebpf_utils::{update_from_config, ProbeID}, procfs_utils::{list_processes, ProcErrorWrapper},
};
use aya::Ebpf;
use ringbuf::storage::Heap;
use ringbuf::traits::Split;
use ringbuf::{CachingCons, CachingProd, HeapRb, SharedRb};
use shared::ziofa::EbpfStreamObject;
use shared::{
    config::Configuration,
    counter::counter_server::CounterServer,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse,
        ProcessList,
        SetConfigurationResponse
        ,
    },
};
use std::thread::spawn;
use std::{collections::HashMap, ops::DerefMut, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

pub struct ZiofaImpl {
    tx: Arc<Mutex<Option<Sender<Result<EbpfStreamObject, Status>>>>>,
    probe_id_map: Arc<Mutex<HashMap<String, ProbeID>>>,
    ebpf: Arc<Mutex<Ebpf>>,
    prod_buffer: Arc<Mutex<CachingProd<Arc<SharedRb<Heap<EbpfStreamObject>>>>>>,
    cons_buffer: Arc<Mutex<CachingCons<Arc<SharedRb<Heap<EbpfStreamObject>>>>>>,
}

impl ZiofaImpl {
    pub async fn new(probe_id_map: HashMap<String, ProbeID>, ebpf: Arc<Mutex<Ebpf>>) -> ZiofaImpl {
        let rb = HeapRb::<EbpfStreamObject>::new(BUFFER_CAPACITY);
        let (b_prod, b_cons) = rb.split();
        ZiofaImpl {
            tx: Arc::new(Mutex::new(None)),
            probe_id_map: Arc::new(Mutex::new(probe_id_map)),
            ebpf,
            prod_buffer: Arc::new(Mutex::new(b_prod)),
            cons_buffer: Arc::new(Mutex::new(b_cons)),
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
        
        let cons_buffer_cloned = Arc::clone(&self.cons_buffer);
        let tx_cloned = Arc::clone(&self.tx);
        
        spawn(
            ||{ 
                write_buffer_to_pipe(
                    cons_buffer_cloned,
                    tx_cloned
                )
            }
        );
        
        Ok(Response::new(SetConfigurationResponse { response_type: 0 }))
    }
    type InitStreamStream = ReceiverStream<Result<EbpfStreamObject, Status>>;

    async fn init_stream(
        &self, _: Request<()>, ) -> Result<Response<Self::InitStreamStream>, Status> {
        let mut guard = self.tx.lock().await;
        let (tx, rx) = mpsc::channel(1);
        *guard = Some(tx);

        Ok(Response::new(Self::InitStreamStream::new(rx)))
    }
}

pub async fn serve_forever() {
    let ebpf = Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/backend-ebpf"
    )))
        .unwrap();
    let probe_id_map = HashMap::new();
    let ebpf = Arc::new(Mutex::new(ebpf));
    let ziofa_server = ZiofaServer::new(ZiofaImpl::new(probe_id_map, ebpf.clone()));
    let counter_server = CounterServer::new(Counter::new(ebpf).await);
    Server::builder()
        .add_service(ziofa_server)
        .add_service(counter_server)
        .serve(constants::sock_addr())
        .await
        .unwrap();
}


