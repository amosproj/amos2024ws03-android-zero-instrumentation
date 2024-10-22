use anyhow::Context as _;
use aya::{maps::{RingBuf}, programs::{Xdp, XdpFlags}, Ebpf};
use clap::Parser;
use shared::counter::{counter_server::{Counter, CounterServer}, Count, LoadProgramRequest, LoadProgramResponse};
#[rustfmt::skip]
use log::{debug, warn};
use tokio::{io::unix::AsyncFd, sync::{mpsc, Mutex}};
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status};
use std::{net::ToSocketAddrs, pin::Pin, sync::Arc};

struct CounterImpl {
    iface: String,
    ebpf: Arc<Mutex<Ebpf>>,
}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<Count, Status>> + Send>>;

impl CounterImpl {
    pub fn new(iface: String, ebpf: Ebpf) -> CounterImpl {
        CounterImpl { iface, ebpf: Arc::new(Mutex::new(ebpf)) }
    }
}

#[tonic::async_trait]
impl Counter for CounterImpl {
    type ServerCountStream = ResponseStream;

    async fn load_program(&self, req: Request<LoadProgramRequest>) -> Result<Response<LoadProgramResponse>, Status> {
        let mut guard = self.ebpf.lock().await;
        let program: &mut Xdp = guard.program_mut(&req.into_inner().name).unwrap().try_into().unwrap();
        program.load().unwrap();
        program.attach(&self.iface, XdpFlags::default())
            .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE").unwrap();

        Ok(Response::new(LoadProgramResponse { loaded: true }))
    }

    async fn server_count(&self, _: Request<()>) -> Result<Response<Self::ServerCountStream>, Status> {
        let mut guard = self.ebpf.lock().await;
        let events = RingBuf::try_from(guard.take_map("EVENTS").unwrap()).unwrap();
        let mut poll = AsyncFd::new(events).unwrap(); 

        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            loop {
                let mut guard = poll.readable_mut().await.unwrap();
                let ring_buf = guard.get_inner_mut();
                while let Some(item) = ring_buf.next() {
                    let sized = <[u8; 4]>::try_from(&*item).unwrap();
                    let count = u32::from_le_bytes(sized);
                    tx.send(Ok(Count { count })).await.unwrap();
                }
                guard.clear_ready();
            }
        });
        

        let output_stream = ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(output_stream) as Self::ServerCountStream
        ))
    }
}

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/example"
    )))?;
    if let Err(e) = aya_log::EbpfLogger::init(&mut ebpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let Opt { iface } = opt;

    let server = CounterImpl::new(iface, ebpf);

    Server::builder()
        .add_service(CounterServer::new(server))
        .serve("[::1]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}
