// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::counter::{counter_client::CounterClient, LoadProgramRequest};
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;

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

    pub async fn load_program(&mut self, name: String) -> Result<()> {
        self.0.load_program(LoadProgramRequest { name }).await?;
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
