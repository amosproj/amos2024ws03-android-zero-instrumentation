// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{pin::Pin, sync::Arc};

use shared::{
    config::Configuration,
    events::{jni_references_event::JniMethodName, file_descriptor_change_event::FileDescriptorOp, Event},
    symbols::search_symbols_response::Symbol, 
    processes::Process,
};
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};

type Result<T> = core::result::Result<T, ClientError>;

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

    pub async fn list_processes(&self) -> Result<Vec<Process>> {
        Ok(self.0.lock().await.list_processes().await?)
    }

    pub async fn get_configuration(&self) -> Result<Configuration> {
        Ok(self.0.lock().await.get_configuration().await?)
    }

    pub async fn set_configuration(&self, configuration: Configuration) -> Result<()> {
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

    pub async fn index_symbols(&self) -> Result<()> {
        Ok(self.0.lock().await.index_symbols().await?)
    }

    pub async fn search_symbols(&self, query: String, limit: u64) -> Result<Vec<Symbol>> {
        Ok(self.0.lock().await.search_symbols(query, limit).await?)
    }

    pub async fn get_symbol_offset(
        &self,
        symbol_name: String,
        library_path: String,
    ) -> Result<Option<u64>> {
        Ok(self
            .0
            .lock()
            .await
            .get_symbol_offset(symbol_name, library_path)
            .await?)
    }
}

#[uniffi::export]
pub fn jni_method_name_from_i32(num: i32) -> JniMethodName {
    JniMethodName::try_from(num).unwrap()
}

#[uniffi::export]
pub fn file_descriptor_change_op_from_i32(num: i32) -> FileDescriptorOp {
    FileDescriptorOp::try_from(num).unwrap()
}
