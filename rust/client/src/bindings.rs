// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{pin::Pin, sync::Arc};

use shared::{config::Configuration, ziofa::Process};
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};
use shared::ziofa::{Event, StringResponse, Symbol};

type Result<T> = core::result::Result<T, ClientError>;

#[derive(uniffi::Object)]
struct CountStream(Mutex<Pin<Box<dyn Stream<Item = Result<u32>> + Send>>>);

#[uniffi::export(async_runtime = "tokio")]
impl CountStream {
    pub async fn next(&self) -> Result<Option<u32>> {
        let mut guard = self.0.lock().await;
        match guard.next().await {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[derive(uniffi::Object)]
struct EventStream(Mutex<Pin<Box<dyn Stream<Item = Result<Event>> + Send>>>);

#[uniffi::export(async_runtime = "tokio")]
impl EventStream {
    pub async fn next(&self) -> Result<Option<Event>> {
        let mut guard = self.0.lock().await;
        match guard.next().await {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[derive(uniffi::Object)]
struct OdexFileStream(Mutex<Pin<Box<dyn Stream<Item = Result<StringResponse>> + Send>>>);

#[uniffi::export(async_runtime = "tokio")]
impl OdexFileStream {
    pub async fn next(&self) -> Result<Option<StringResponse>> {
        let mut guard = self.0.lock().await;
        match guard.next().await {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[derive(uniffi::Object)]
struct SymbolStream(Mutex<Pin<Box<dyn Stream<Item = Result<Symbol>> + Send>>>);

#[uniffi::export(async_runtime = "tokio")]
impl SymbolStream {
    pub async fn next(&self) -> Result<Option<Symbol>> {
        let mut guard = self.0.lock().await;
        match guard.next().await {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[derive(uniffi::Object)]
struct Client(Mutex<crate::client::Client>);

#[derive(uniffi::Error, thiserror::Error, Debug)]
#[uniffi(flat_error)] // TODO: convert errors
enum ClientError {
    #[error(transparent)]
    Inner(#[from] crate::client::ClientError),
}

#[uniffi::export(async_runtime = "tokio")]
impl Client {
    #[uniffi::constructor]
    async fn connect(url: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Client(Mutex::new(
            crate::client::Client::connect(url).await?,
        ))))
    }

    pub async fn load(&self) -> Result<()> {
        Ok(self.0.lock().await.load().await?)
    }

    pub async fn unload(&self) -> Result<()> {
        Ok(self.0.lock().await.unload().await?)
    }

    pub async fn attach(&self, iface: String) -> Result<()> {
        Ok(self.0.lock().await.attach(iface).await?)
    }

    pub async fn detach(&self, iface: String) -> Result<()> {
        Ok(self.0.lock().await.detach(iface).await?)
    }

    pub async fn start_collecting(&self) -> Result<()> {
        Ok(self.0.lock().await.start_collecting().await?)
    }

    pub async fn stop_collecting(&self) -> Result<()> {
        Ok(self.0.lock().await.stop_collecting().await?)
    }

    pub async fn server_count(&self) -> Result<CountStream> {
        let mut guard = self.0.lock().await;
        let stream = guard
            .server_count()
            .await?
            .map(|x| x.map_err(ClientError::from));

        Ok(CountStream(Mutex::new(Box::pin(stream))))
    }

    pub async fn check_server(&self) -> Result<()> {
        Ok(self.0.lock().await.check_server().await?)
    }

    pub async fn list_processes(&self) -> Result<Vec<Process>> {
        Ok(self.0.lock().await.list_processes().await?)
    }

    pub async fn get_configuration(&self) -> Result<Configuration> {
        Ok(self.0.lock().await.get_configuration().await?)
    }

    pub async fn set_configuration(&self, configuration: Configuration) -> Result<u32> {
        Ok(self.0.lock().await.set_configuration(configuration).await?)
    }

    pub async fn init_stream(&self) -> Result<EventStream> {
        let mut guard = self.0.lock().await;
        let stream = guard
            .init_stream()
            .await?
            .map(|x| x.map_err(ClientError::from));

        Ok(EventStream(Mutex::new(Box::pin(stream))))
    }

    pub async fn get_odex_files(&self, pid: u32) -> Result<OdexFileStream> {
        let mut guard = self.0.lock().await;
        let stream = guard
            .get_odex_files(pid)
            .await?
            .map(|x| x.map_err(ClientError::from));

        Ok(OdexFileStream(Mutex::new(Box::pin(stream))))
    }

    pub async fn get_symbols(&self, odex_file: String) -> Result<SymbolStream> {
        let mut guard = self.0.lock().await;
        let stream = guard
            .get_symbols(odex_file)
            .await?
            .map(|x| x.map_err(ClientError::from));

        Ok(SymbolStream(Mutex::new(Box::pin(stream))))
    }
}
