// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::{constants, configuration};
use crate::ebpf_utils::{ProbeID, update_from_config};
use shared::config::Configuration;
use shared::ziofa::ziofa_server::{Ziofa, ZiofaServer};
use shared::ziofa::{
    CheckServerResponse,
    Process, ProcessList, SetConfigurationResponse,
};
use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use aya::Ebpf;
use tokio::sync::Mutex;

pub struct ZiofaImpl {
    // tx: Option<Sender<Result<EbpfStreamObject, Status>>>,
    probe_id_map: Arc<Mutex<HashMap<String, ProbeID>>>,
    ebpf: Arc<Mutex<Ebpf>>,
}

impl ZiofaImpl {
    pub fn new(probe_id_map: HashMap<String, ProbeID>, ebpf: Ebpf) -> ZiofaImpl {
        ZiofaImpl { probe_id_map: Arc::new(Mutex::new(probe_id_map)), ebpf: Arc::new(Mutex::new(ebpf)) }
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

    async fn get_configuration(
        &self,
        _: Request<()>,
    ) -> Result<Response<Configuration>, Status> {

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
        update_from_config(ebpf_guard.deref_mut(), "", probe_id_map_guard.deref_mut());

        Ok(Response::new(SetConfigurationResponse{ response_type: 0}))
    }

    // type InitStreamStream = ReceiverStream<Result<EbpfStreamObject, Status>>;
    // fn init_stream(
    //     &self,
    //     _: Request<()>,
    // ) -> Result<Response<Self::InitStreamStream>, Status> {
    //     let (_tx, rx) = mpsc::channel(1);
    //
    //     Ok(Response::new(Self::InitStreamStream::new(rx)))
    // }
}

pub async fn serve_forever() {
    let ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/example"
    ))).unwrap();
    let probe_id_map = HashMap::new();
    let service = ZiofaServer::new(ZiofaImpl::new(probe_id_map, ebpf));
    Server::builder()
        .add_service(service)
        .serve(constants::sock_addr())
        .await
        .unwrap();
}
