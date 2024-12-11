// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::io;

use async_broadcast::Sender;
use tokio::io::unix::AsyncFd;
use tokio::{join, select};
use tonic::Status;
use tracing::error;
use backend_common::{JNICall, JNIMethodName, SysSendmsgCall, TryFromRaw, VfsWriteCall};
use shared::ziofa::{Event, JniReferencesEvent, SysSendmsgEvent, VfsWriteEvent};
use shared::ziofa::event::{EventData};
use shared::ziofa::jni_references_event::{JniMethodName};

use crate::registry::{EbpfRegistry, RegistryGuard, RegistryItem, TypedRingBuffer};

pub trait IntoEvent {
    fn into_event(self) -> Event;
}

impl IntoEvent for VfsWriteCall {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::VfsWrite(VfsWriteEvent {
                pid: self.pid,
                tid: self.tid,
                begin_time_stamp: self.begin_time_stamp,
                fp: self.fp,
                bytes_written: self.bytes_written as u64
            }))
        }
    }
}

impl IntoEvent for SysSendmsgCall {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::SysSendmsg(SysSendmsgEvent {
                pid: self.pid,
                tid: self.tid,
                begin_time_stamp: self.begin_time_stamp,
                fd: self.fd,
                duration_nano_sec: self.duration_nano_sec
            }))
        }
    }
}

impl IntoEvent for JNICall {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::JniReferences(JniReferencesEvent {
                pid: self.pid,
                tid: self.tid,
                begin_time_stamp: self.begin_time_stamp,
                jni_method_name: (match self.method_name {
                    JNIMethodName::AddLocalRef => JniMethodName::AddLocalRef,
                    JNIMethodName::DeleteLocalRef => JniMethodName::DeleteLocalRef,
                    JNIMethodName::AddGlobalRef => JniMethodName::AddGlobalRef,
                    JNIMethodName::DeleteGlobalRef => JniMethodName::DeleteGlobalRef,
                }).into(),
            }))
        }
    }
}

pub struct MultiCollector {
    vfs_write: Option<Collector<VfsWriteCall>>,
    sys_sendmsg: Option<Collector<SysSendmsgCall>>,
    jni_event: Option<Collector<JNICall>>,
}

impl MultiCollector {
    pub fn from_registry(registry: &EbpfRegistry) -> Result<Self, io::Error> {
        Ok(Self {
            vfs_write: Some(Collector::from_registry_item(registry.event.vfs_write_events.clone())?),
            sys_sendmsg: Some(Collector::from_registry_item(registry.event.sys_sendmsg_events.clone())?),
            jni_event: Some(Collector::from_registry_item(registry.event.jni_ref_calls.clone())?),
        })
    }
    
    pub async fn collect(&mut self, tx: Sender<Result<Event, Status>>, shutdown: tokio::sync::oneshot::Receiver<()>) -> Result<(), std::io::Error> {
        
        let (vfs_write_shutdown_tx, vfs_write_shutdown_rx) = tokio::sync::oneshot::channel();
        let (sys_sendmsg_shutdown_tx, sys_sendmsg_shutdown_rx) = tokio::sync::oneshot::channel();
        let (jni_event_shutdown_tx, jni_event_shutdown_rx) = tokio::sync::oneshot::channel();

        let cancellation_task = async move {
            if shutdown.await.is_err() {
                error!("Error while waiting for shutdown signal");
            }
            if vfs_write_shutdown_tx.send(()).is_err() {
                error!("Error while cancelling vfs_write collector");
            }
            if sys_sendmsg_shutdown_tx.send(()).is_err() {
                error!("Error while cancelling sys_sendmsg collector");
            }
            if jni_event_shutdown_tx.send(()).is_err() {
                error!("Error while cancelling sys_sendmsg collector");
            }
        };
        
        let vfs_write_tx = tx.clone();
        let mut vfs_write = self.vfs_write.take().expect("vfs_write should be initialized");
        let vfs_write_task = async {
            vfs_write.collect(vfs_write_tx, vfs_write_shutdown_rx).await?;
            Ok::<(), std::io::Error>(())
        };
        
        let sys_sendmsg_tx = tx.clone();
        let mut sys_sendmsg = self.sys_sendmsg.take().expect("sys_sendmsg should be initialized");
        let sys_sendmsg_task = async {
            sys_sendmsg.collect(sys_sendmsg_tx, sys_sendmsg_shutdown_rx).await?;
            Ok::<(), std::io::Error>(())
        };

        let jni_event_tx = tx;
        let mut jni_event = self.jni_event.take().expect("jni_event should be initialized");
        let jni_event_task = async {
            jni_event.collect(jni_event_tx, jni_event_shutdown_rx).await?;
            Ok::<(), std::io::Error>(())
        };
        
        let (_, vfs_write_result, sys_sendmsg_result, jni_event_result) = join!(cancellation_task, vfs_write_task, sys_sendmsg_task, jni_event_task);
        
        self.vfs_write = Some(vfs_write);
        self.sys_sendmsg = Some(sys_sendmsg);
        self.jni_event = Some(jni_event);

        // TODO: multiple errors
        vfs_write_result?;
        sys_sendmsg_result?;
        jni_event_result?;

        Ok(())
    }
}

pub struct Collector<T: IntoEvent + TryFromRaw>(AsyncFd<RegistryGuard<TypedRingBuffer<T>>>);

impl<T: IntoEvent + TryFromRaw> Collector<T> {
    pub fn from_registry_item(item: RegistryItem<TypedRingBuffer<T>>) -> Result<Self, io::Error> {
        let map = AsyncFd::new(item.take())?;
        Ok(Self(map))
    }

    pub async fn collect(&mut self, tx: Sender<Result<Event, Status>>, mut shutdown: tokio::sync::oneshot::Receiver<()>) -> Result<(), std::io::Error> {
        loop {
            select! {
                handle = self.0.readable_mut() => {
                    let mut handle = handle?;
                    let rb = handle.get_inner_mut();

                    while let Some(item) = rb.next() {
                        match tx.broadcast(Ok(item.into_event())).await {
                            Ok(_) => {},
                            Err(async_broadcast::SendError(event)) => {
                                error!(
                                    ?event,
                                    "Failed to broadcast"
                                );
                            }
                        }
                    }
                    handle.clear_ready();
                }
                _ = &mut shutdown => {
                    break;
                }
            }
        }

        Ok(())
    }
}

