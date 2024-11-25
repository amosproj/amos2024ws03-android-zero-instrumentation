// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::{
    config::Configuration,
    counter::{counter_client::CounterClient, IfaceMessage},
    ziofa::{ziofa_client::ZiofaClient, Process},
};
use tokio_stream::{Stream, StreamExt};
use tonic::{
    transport::{Channel, Endpoint},
    Request,
};
use shared::ziofa::Event;

#[derive(Clone, Debug)]
pub struct Client {
    counter: CounterClient<Channel>,
    ziofa: ZiofaClient<Channel>,
}

pub type Result<T> = core::result::Result<T, ClientError>;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    Status(#[from] tonic::Status),

    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
}

impl Client {
    pub async fn connect(url: String) -> Result<Self> {
        let conn = Endpoint::new(url)?.connect().await?;
        let counter = CounterClient::new(conn.clone());
        let ziofa = ZiofaClient::new(conn);

        Ok(Self { counter, ziofa })
    }

    pub async fn load(&mut self) -> Result<()> {
        self.counter.load(Request::new(())).await?;
        Ok(())
    }

    pub async fn unload(&mut self) -> Result<()> {
        self.counter.unload(Request::new(())).await?;
        Ok(())
    }

    pub async fn attach(&mut self, iface: String) -> Result<()> {
        self.counter
            .attach(Request::new(IfaceMessage { iface }))
            .await?;
        Ok(())
    }

    pub async fn detach(&mut self, iface: String) -> Result<()> {
        self.counter
            .detach(Request::new(IfaceMessage { iface }))
            .await?;
        Ok(())
    }

    pub async fn start_collecting(&mut self) -> Result<()> {
        self.counter.start_collecting(Request::new(())).await?;
        Ok(())
    }

    pub async fn stop_collecting(&mut self) -> Result<()> {
        self.counter.stop_collecting(Request::new(())).await?;
        Ok(())
    }

    pub async fn server_count(&mut self) -> Result<impl Stream<Item = Result<u32>>> {
        let stream = self
            .counter
            .server_count(())
            .await?
            .into_inner()
            .map(|s| Ok(s.map(|c| c.count)?));

        Ok(stream)
    }

    pub async fn check_server(&mut self) -> Result<()> {
        self.ziofa.check_server(()).await?;
        Ok(())
    }

    pub async fn list_processes(&mut self) -> Result<Vec<Process>> {
        Ok(self.ziofa.list_processes(()).await?.into_inner().processes)
    }

    pub async fn get_configuration(&mut self) -> Result<Configuration> {
        Ok(self.ziofa.get_configuration(()).await?.into_inner())
    }

    pub async fn set_configuration(&mut self, configuration: Configuration) -> Result<()> {
        self.ziofa.set_configuration(configuration).await?;
        Ok(())
    }

    pub async fn init_stream(&mut self) -> Result<impl Stream<Item = Result<Event>>> {
        Ok(self.ziofa.init_stream(()).await?.into_inner().map(|s| Ok(s?)))
    }
}
