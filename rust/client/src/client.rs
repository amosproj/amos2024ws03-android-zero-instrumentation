// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::counter::{counter_client::CounterClient, IfaceMessage};
use tokio_stream::{Stream, StreamExt};
use tonic::{transport::Channel, Request};

#[derive(Clone, Debug)]
pub struct Client(CounterClient<Channel>);

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
        Ok(Self(CounterClient::connect(url).await?))
    }

    pub async fn load(&mut self) -> Result<()> {
        self.0.load(Request::new(())).await?;
        Ok(())
    }

    pub async fn unload(&mut self) -> Result<()> {
        self.0.unload(Request::new(())).await?;
        Ok(())
    }

    pub async fn attach(&mut self, iface: String) -> Result<()> {
        self.0.attach(Request::new(IfaceMessage { iface })).await?;
        Ok(())
    }

    pub async fn detach(&mut self, iface: String) -> Result<()> {
        self.0.detach(Request::new(IfaceMessage { iface })).await?;
        Ok(())
    }

    pub async fn start_collecting(&mut self) -> Result<()> {
        self.0.start_collecting(Request::new(())).await?;
        Ok(())
    }

    pub async fn stop_collecting(&mut self) -> Result<()> {
        self.0.stop_collecting(Request::new(())).await?;
        Ok(())
    }

    pub async fn server_count(&mut self) -> Result<impl Stream<Item = Result<u32>>> {
        let stream = self
            .0
            .server_count(())
            .await?
            .into_inner()
            .map(|s| Ok(s.map(|c| c.count)?));

        Ok(stream)
    }
}
