// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use shared::{
    config::Configuration,
    events::Event,
    processes::Process,
    symbols::{search_symbols_response::Symbol, GetSymbolOffsetRequest, SearchSymbolsRequest},
    ziofa::ziofa_client::ZiofaClient,
};
use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Channel, Endpoint};

#[derive(Clone, Debug)]
pub struct Client {
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
        let ziofa = ZiofaClient::new(conn);

        Ok(Self { ziofa })
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
        Ok(self
            .ziofa
            .init_stream(())
            .await?
            .into_inner()
            .map(|s| Ok(s?)))
    }

    pub async fn index_symbols(&mut self) -> Result<()> {
        self.ziofa.index_symbols(()).await?;
        Ok(())
    }

    pub async fn search_symbols(&mut self, query: String, limit: u64) -> Result<Vec<Symbol>> {
        Ok(self
            .ziofa
            .search_symbols(SearchSymbolsRequest { query, limit })
            .await?
            .into_inner()
            .symbols)
    }

    pub async fn get_symbol_offset(
        &mut self,
        symbol_name: String,
        library_path: String,
    ) -> Result<Option<u64>> {
        Ok(self
            .ziofa
            .get_symbol_offset(GetSymbolOffsetRequest {
                symbol_name,
                library_path,
            })
            .await?
            .into_inner()
            .offset)
    }
}
