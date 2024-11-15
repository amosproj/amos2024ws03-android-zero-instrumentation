// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{pin::Pin, sync::Arc};

use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};

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

    async fn server_count(&self) -> Result<CountStream> {
        let mut guard = self.0.lock().await;
        let stream = guard
            .server_count()
            .await?
            .map(|x| x.map_err(ClientError::from));

        Ok(CountStream(Mutex::new(Box::pin(stream))))
    }
}
