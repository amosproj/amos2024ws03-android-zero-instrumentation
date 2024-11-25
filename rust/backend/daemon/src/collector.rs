// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use async_broadcast::Sender;
use aya::Ebpf;
use aya::maps::{MapData, MapError, RingBuf};
use tokio::io::unix::AsyncFd;
use tokio::select;
use tonic::Status;
use tracing::error;
use backend_common::VfsWriteCall;
use shared::ziofa::{Event, VfsWriteEvent};
use shared::ziofa::event::{EventData};

pub struct VfsWriteCollector {
    map: AsyncFd<RingBuf<MapData>>
}

impl VfsWriteCollector {
    pub fn from_ebpf(ebpf: &mut Ebpf) -> Result<Self, MapError> {
        let map: RingBuf<_> = ebpf.take_map("VFS_WRITE_MAP")
            .ok_or(MapError::InvalidName { name: "VFS_WRITE_MAP".to_string() })?
            .try_into()?;

        let map = AsyncFd::new(map)?;

        Ok(Self { map })
    }

    pub async fn collect(&mut self, tx: Sender<Result<Event, Status>>, mut shutdown: tokio::sync::oneshot::Receiver<()>) -> Result<(), std::io::Error> {
        loop {
            select! {
                handle = self.map.readable_mut() => {
                    let mut handle = handle?;
                    let rb = handle.get_inner_mut();

                    while let Some(item) = rb.next() {
                        let data = unsafe { &*(item.as_ptr() as  *const VfsWriteCall) };
                        let event = Event {
                            event_data: Some(EventData::VfsWrite(VfsWriteEvent {
                                pid: data.pid,
                                tid: data.tid,
                                begin_time_stamp: data.begin_time_stamp,
                                fp: data.fp,
                                bytes_written: data.bytes_written as u64
                            }))
                        };
                        match tx.broadcast(Ok(event)).await {
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

