// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::marker::PhantomData;

use async_broadcast::Sender;
use aya::maps::ring_buf::RingBufItem;
use aya::Ebpf;
use aya::maps::{MapData, MapError, RingBuf};
use tokio::io::unix::AsyncFd;
use tokio::{join, select};
use tonic::Status;
use tracing::error;
use backend_common::{JNICall, JNIMethodName, SysSendmsgCall, VfsWriteCall};
use shared::ziofa::{Event, JniReferencesEvent, SysSendmsgEvent, VfsWriteEvent};
use shared::ziofa::event::{EventData};
use shared::ziofa::jni_references_event;

pub trait CollectFromMap {
    const MAP_NAME: &'static str;

    fn convert(item: RingBufItem<'_>) -> Result<Event, Status>;
}

struct VfsWriteCollect;
struct JNICollect;
struct SysSendmsgCollect;

impl CollectFromMap for VfsWriteCollect {
    const MAP_NAME: &'static str = "VFS_WRITE_EVENTS";

    fn convert(item: RingBufItem<'_>) -> Result<Event, Status> {
        let data = unsafe { &*(item.as_ptr() as *const VfsWriteCall) };
        Ok(Event {
            event_data: Some(EventData::VfsWrite(VfsWriteEvent {
                pid: data.pid,
                tid: data.tid,
                begin_time_stamp: data.begin_time_stamp,
                fp: data.fp,
                bytes_written: data.bytes_written as u64
            }))
        })
    }
}

impl CollectFromMap for JNICollect {
    const MAP_NAME: &'static str = "JNI_REF_CALLS";

    fn convert(item: RingBufItem<'_>) -> Result<Event, Status> {
        let data = unsafe { &*(item.as_ptr() as *const JNICall) };
        
        // manual cast from the ebpf (c rep.) typ to protobuf (rust rep.) type
        let jni_method_name = match data.method_name {
                JNIMethodName::AddLocalRef => jni_references_event::JniMethodName::AddLocalRef,
                JNIMethodName::DeleteLocalRef => jni_references_event::JniMethodName::DeleteLocalRef,
                JNIMethodName::AddGlobalRef => jni_references_event::JniMethodName::AddGlobalRef,
                JNIMethodName::DeleteGlobalRef => jni_references_event::JniMethodName::DeleteGlobalRef,
            };
        
        Ok(Event {
            event_data: Some(EventData::JniReferences(JniReferencesEvent {
                pid: data.pid,
                tid: data.tid,
                begin_time_stamp: data.begin_time_stamp,
                jni_method_name: i32::from(jni_method_name),
            }))
        })
    }
}


impl CollectFromMap for SysSendmsgCollect {
    const MAP_NAME: &'static str = "SYS_SENDMSG_EVENTS";

    fn convert(item: RingBufItem<'_>) -> Result<Event, Status> {
        let data = unsafe { &*(item.as_ptr() as *const SysSendmsgCall) };
        Ok(Event {
            event_data: Some(EventData::SysSendmsg(SysSendmsgEvent {
                pid: data.pid,
                tid: data.tid,
                begin_time_stamp: data.begin_time_stamp,
                fd: data.fd,
                duration_nano_sec: data.duration_nano_sec
            }))
        })
    }
}

pub struct MultiCollector {
    vfs_write: Option<Collector<VfsWriteCollect>>,
    sys_sendmsg: Option<Collector<SysSendmsgCollect>>,
    jni_event: Option<Collector<JNICollect>>,
}

impl MultiCollector {
    pub fn from_ebpf(ebpf: &mut Ebpf) -> Result<Self, MapError> {
        let vfs_write = Collector::<VfsWriteCollect>::from_ebpf(ebpf)?;
        let sys_sendmsg = Collector::<SysSendmsgCollect>::from_ebpf(ebpf)?;
        let jni_collect = Collector::<JNICollect>::from_ebpf(ebpf)?;
        Ok(Self { vfs_write: Some(vfs_write), sys_sendmsg: Some(sys_sendmsg), jni_event: Some(jni_collect) })
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

pub struct Collector<T: CollectFromMap> {
    map: AsyncFd<RingBuf<MapData>>,
    _collector: PhantomData<T>,
}

impl<T: CollectFromMap> Collector<T> {
    pub fn from_ebpf(ebpf: &mut Ebpf) -> Result<Self, MapError> {
        let map: RingBuf<_> = ebpf.take_map(T::MAP_NAME)
            .ok_or(MapError::InvalidName { name: T::MAP_NAME.to_string() })?
            .try_into()?;

        let map = AsyncFd::new(map)?;

        Ok(Self { map, _collector: PhantomData })
    }

    pub async fn collect(&mut self, tx: Sender<Result<Event, Status>>, mut shutdown: tokio::sync::oneshot::Receiver<()>) -> Result<(), std::io::Error> {
        loop {
            select! {
                handle = self.map.readable_mut() => {
                    let mut handle = handle?;
                    let rb = handle.get_inner_mut();

                    while let Some(item) = rb.next() {
                        let event = T::convert(item);
                        match tx.broadcast(event).await {
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

