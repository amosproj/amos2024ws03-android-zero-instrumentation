// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, ops::DerefMut, sync::Arc};

use aya::Ebpf;
use shared::{
    config::Configuration,
    counter::counter_server::CounterServer,
    ziofa::{
        ziofa_server::{Ziofa, ZiofaServer},
        CheckServerResponse, Process, ProcessList, SetConfigurationResponse,
    },
};
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    configuration, constants,
    counter::Counter,
    ebpf_utils::{update_from_config, ProbeID},
    constants::DEV_DEFAULT_CONFIG_PATH,
};

pub struct ZiofaImpl {
    // tx: Option<Sender<Result<EbpfStreamObject, Status>>>,
    probe_id_map: Arc<Mutex<HashMap<String, ProbeID>>>,
    ebpf: Arc<Mutex<Ebpf>>,
}

impl ZiofaImpl {
    pub fn new(probe_id_map: HashMap<String, ProbeID>, ebpf: Arc<Mutex<Ebpf>>) -> ZiofaImpl {
        ZiofaImpl {
            probe_id_map: Arc::new(Mutex::new(probe_id_map)),
            ebpf,
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

    async fn get_configuration(&self, _: Request<()>) -> Result<Response<Configuration>, Status> {
        let config = configuration::load_from_file(DEV_DEFAULT_CONFIG_PATH)?;
        Ok(Response::new(config))
    }

    async fn set_configuration(
        &self,
        request: Request<Configuration>,
    ) -> Result<Response<SetConfigurationResponse>, Status> {
        let config = request.into_inner();

        //TODO: implement validate
        configuration::validate(&config, DEV_DEFAULT_CONFIG_PATH)?;
        configuration::save_to_file(&config, DEV_DEFAULT_CONFIG_PATH)?;

        let mut ebpf_guard = self.ebpf.lock().await;
        let mut probe_id_map_guard = self.probe_id_map.lock().await;


        update_from_config(
            ebpf_guard.deref_mut(),
            DEV_DEFAULT_CONFIG_PATH,
            probe_id_map_guard.deref_mut(),
        );

        Ok(Response::new(SetConfigurationResponse { response_type: 0 }))
    }
}

pub async fn serve_forever() {
    let ebpf = Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/backend-ebpf"
    ))).unwrap();
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
