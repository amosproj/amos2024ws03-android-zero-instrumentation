// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{collections::BTreeMap, fmt::Debug, pin::Pin, sync::Arc};

use aya::{
    maps::{MapData, MapError, RingBuf},
    programs::{xdp::XdpLinkId, ProgramError, Xdp, XdpFlags},
    Ebpf,
};
use shared::counter::{Count, IfaceMessage};
use thiserror::Error;
use tokio::{
    io::unix::AsyncFd,
    select, spawn,
    sync::{oneshot, watch, Mutex, MutexGuard},
};
use tokio_stream::{wrappers::WatchStream, Stream, StreamExt};
use tonic::{Request, Response, Status};
use tracing::{error, instrument, trace};

#[derive(Debug)]
struct CounterChannels {
    counter_state: (watch::Sender<u32>, watch::Receiver<u32>),
    cancel: oneshot::Sender<()>,
}

pub struct Counter {
    ebpf: Arc<Mutex<Ebpf>>,
    events: Arc<Mutex<Option<RingBuf<MapData>>>>,
    links: Arc<Mutex<BTreeMap<String, XdpLinkId>>>,
    channels: Arc<Mutex<Option<CounterChannels>>>,
}

impl Debug for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter")
            .field("ebpf", &self.ebpf)
            .field("events", &"Arc<Mutex<Option<RingBuf<MapData>>>>")
            .field("links", &self.links)
            .field("channels", &self.channels)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum CounterError {
    #[error("program {program} does not exist")]
    ProgramDoesNotExist { program: String },
    #[error("iface {iface} not attached")]
    InterfaceNotAttached { iface: String },
    #[error("already collecting")]
    AlreadyCollecting,
    #[error("not collecting")]
    NotCollecting,
    #[error(transparent)]
    ProgramError(#[from] ProgramError),
    #[error(transparent)]
    MapError(#[from] MapError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SendError(#[from] watch::error::SendError<u32>),
}

impl From<CounterError> for Status {
    fn from(value: CounterError) -> Self {
        Self::from_error(Box::new(value))
    }
}

struct ExampleGuard<'a>(MutexGuard<'a, Ebpf>);

impl ExampleGuard<'_> {
    fn inner_mut(&mut self) -> Result<&mut Xdp, CounterError> {
        Ok(self
            .0
            .program_mut("example")
            .expect("example should be exported from ebpf")
            .try_into()?)
    }
}

impl Counter {
    #[instrument()]
    pub async fn new(ebpf: Arc<Mutex<Ebpf>>) -> Self {
        let events: RingBuf<_> = ebpf
            .lock()
            .await
            .take_map("EVENTS")
            .expect("EVENTS should be exported by ebpf")
            .try_into()
            .expect("EVENTS should be RingBuf");
        Self {
            ebpf,
            links: Default::default(),
            channels: Default::default(),
            events: Arc::new(Mutex::new(Some(events))),
        }
    }

    async fn get_program(&self) -> ExampleGuard<'_> {
        ExampleGuard(self.ebpf.lock().await)
    }

    #[instrument]
    pub async fn load(&self) -> Result<(), CounterError> {
        let mut program = self.get_program().await;
        program.inner_mut()?.load()?;
        Ok(())
    }

    #[instrument]
    pub async fn unload(&self) -> Result<(), CounterError> {
        let mut program = self.get_program().await;
        program.inner_mut()?.unload()?;
        self.links.lock().await.clear();
        Ok(())
    }

    #[instrument]
    pub async fn attach(&self, iface: String) -> Result<(), CounterError> {
        let mut program = self.get_program().await;
        let id = program.inner_mut()?.attach(&iface, XdpFlags::default())?;
        self.links.lock().await.insert(iface, id);
        Ok(())
    }

    #[instrument]
    pub async fn detach(&self, iface: String) -> Result<(), CounterError> {
        let mut program = self.get_program().await;
        let id = self
            .links
            .lock()
            .await
            .remove(&iface)
            .ok_or(CounterError::InterfaceNotAttached { iface })?;
        program.inner_mut()?.detach(id)?;
        Ok(())
    }

    #[instrument]
    async fn start_collecting(&self) -> Result<(), CounterError> {
        let mut channels = self.channels.lock().await;

        if channels.is_some() {
            return Err(CounterError::AlreadyCollecting);
        }

        let ring_buf = AsyncFd::new(self.events.lock().await.take().expect("Map should exist"))?;

        let counter_state = watch::channel(0);
        let (cancel, mut is_cancelled) = oneshot::channel();

        let mut sch = ServerCountHolder {
            fd: ring_buf,
            tx: counter_state.0.clone(),
        };
        let events_holder = self.events.clone();
        spawn(async move {
            loop {
                select! {
                    val = server_count_iteration(&mut sch) => {
                        if let Err(e) = val {
                            error!("{e:?}")
                        }
                    }
                    _ = &mut is_cancelled => {
                        break;
                    }
                }
            }
            *events_holder.lock().await = Some(sch.fd.into_inner());
        });

        *channels = Some(CounterChannels {
            counter_state,
            cancel,
        });

        Ok(())
    }

    #[instrument]
    async fn stop_collecting(&self) -> Result<(), CounterError> {
        let channels = if let Some(c) = self.channels.lock().await.take() {
            c
        } else {
            return Ok(());
        };

        if channels.cancel.send(()).is_err() {
            error!("could not send cancel request");
        };

        Ok(())
    }

    #[instrument]
    pub async fn server_count(&self) -> Result<impl Stream<Item = u32>, CounterError> {
        let guard = self.channels.lock().await;
        let channels = if let Some(c) = &*guard {
            c
        } else {
            return Err(CounterError::NotCollecting);
        };

        let rx = channels.counter_state.1.clone();

        Ok(WatchStream::new(rx))
    }
}

struct ServerCountHolder {
    fd: AsyncFd<RingBuf<aya::maps::MapData>>,
    tx: watch::Sender<u32>,
}

impl Debug for ServerCountHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerCountHolder")
            .field("fd", &"AsyncFd<RingBuf>")
            .field("tx", &self.tx)
            .finish()
    }
}

#[instrument]
async fn server_count_iteration(sch: &mut ServerCountHolder) -> Result<(), CounterError> {
    let mut guard = sch.fd.readable_mut().await?;
    let inner = guard.get_inner_mut();
    while let Some(item) = inner.next() {
        let sized: [u8; 4] = (&*item).try_into().expect("EVENTS items should be u32");
        let count = u32::from_le_bytes(sized);
        trace!(
            name: "count",
            item = ?item,
            count,
        );
        sch.tx.send(count)?;
    }
    guard.clear_ready();
    Ok(())
}

#[tonic::async_trait]
impl shared::counter::counter_server::Counter for Counter {
    type ServerCountStream =
        Pin<Box<dyn Stream<Item = Result<shared::counter::Count, Status>> + Send + Sync + 'static>>;

    async fn load(&self, _: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(self.load().await?))
    }
    async fn unload(&self, _: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(self.unload().await?))
    }
    async fn attach(&self, req: Request<IfaceMessage>) -> Result<Response<()>, Status> {
        let IfaceMessage { iface } = req.into_inner();
        Ok(Response::new(self.attach(iface).await?))
    }
    async fn detach(&self, req: Request<IfaceMessage>) -> Result<Response<()>, Status> {
        let IfaceMessage { iface } = req.into_inner();
        Ok(Response::new(self.detach(iface).await?))
    }
    async fn start_collecting(&self, _: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(self.start_collecting().await?))
    }
    async fn stop_collecting(&self, _: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(self.stop_collecting().await?))
    }

    async fn server_count(
        &self,
        _: tonic::Request<()>,
    ) -> Result<Response<Self::ServerCountStream>, Status> {
        let stream = self.server_count().await?.map(|i| Ok(Count { count: i }));
        let stream_pinned = Box::pin(stream);

        Ok(Response::new(stream_pinned))
    }
}
