use std::sync::Arc;

use async_stream::try_stream;
use aya_wrapper::{aya::{programs::{ProgramError, XdpFlags}, maps::MapError}, maps::{RingBuf, RingBufRef}, programs::{Xdp, XdpMut}, Ebpf};
use thiserror::Error;
use tokio::{io::unix::AsyncFd, sync::Mutex};
use tokio_stream::Stream;

#[derive(Error, Debug)]
pub enum CounterError {
    #[error("program not found")]
    ProgramNotFound,
    
    #[error("map not found")]
    MapNotFound,
    
    #[error(transparent)]
    ProgramError(#[from] ProgramError),

    #[error(transparent)]
    MapError(#[from] MapError),
    
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("counter size is invalid")]
    InvalidCounterSize,
}

pub struct Counter<E: Ebpf> {
    ebpf: Arc<Mutex<E>>,
    iface: String,
}


impl<E: Ebpf> Counter<E> {
    pub fn new(ebpf: E, iface: String) -> Self {
        Self {
            ebpf: Arc::new(Mutex::new(ebpf)),
            iface,
        }
    }
    
    async fn load_program(&self, name: &str) -> Result<(), CounterError> {
        let mut guard = self.ebpf.lock().await;
        let mut program = guard
            .program_mut(name)
            .map(Xdp::try_from)
            .ok_or(CounterError::ProgramNotFound)??;

        program.load()?;
        program.attach(&self.iface, XdpFlags::default())?;
        
        Ok(())
    }
    
    async fn server_count(&self) -> Result<impl Stream<Item = Result<u32, CounterError>>, CounterError> {
        let mut guard = self.ebpf.lock().await;
        let events = guard
            .take_map("EVENTS")
            .map(RingBuf::try_from)
            .ok_or(CounterError::MapNotFound)??;

        let mut poll = AsyncFd::new(events)?; 

        let s = try_stream! {
            loop {
                let mut guard = poll.readable_mut().await?;
                let ring_buf = guard.get_inner_mut();
                while let Some(item) = ring_buf.next() {
                    let sized = <[u8; 4]>::try_from(&*item)
                        .or(Err(CounterError::InvalidCounterSize))?;
                    yield u32::from_le_bytes(sized);
                }
                guard.clear_ready();
            }
        };
        
        Ok(s)
    }
}


#[cfg(test)]
mod tests {
    use aya_wrapper::{maps::{ring_buf::{self, MockRingBufItem}, MapOwned, MockMapConverter, MockRingBuf}, mockall::predicate::{always, eq}, programs::{xdp::{self, MockXdpLinkId}, MockProgramConverter, MockXdp, ProgramMut}, MockEbpf};
    use eventfd::{EfdFlags, EventFD};
    use tokio_stream::StreamExt;
    use std::os::fd::AsRawFd;
    
    struct MockingSetup {
        ebpf: MockEbpf,
        xdp_ctx: xdp::TryFromContext,
        rb_ctx: ring_buf::TryFromContext,
    }
    
    use super::*;
    
    fn setup_mock_ebpf() -> MockingSetup {
        
        let mut ebpf = MockEbpf::new();
        let xdp_ctx = MockXdp::try_from_context();
        let rb_ctx = MockRingBuf::try_from_context();
        
        ebpf.expect_program_mut()
            .with(eq("example"))
            .returning(|_| {
                let mock = MockProgramConverter::new();
                Some(ProgramMut::new(mock))
            });
        
        ebpf.expect_take_map()
            .with(eq("EVENTS"))
            .return_once(|_| {
                let mock = MockMapConverter::new();
                Some(MapOwned::new(mock))
            });
        
        xdp_ctx.expect()
            .returning(|_| {
                let mut mock = MockXdp::new();
                mock.expect_load().returning(|| Ok(()));
                mock.expect_attach()
                    .with(eq("eth0"), always())
                    .returning(|_, _| Ok(MockXdpLinkId::new()));
                Ok(mock)
            });
        
        
        rb_ctx.expect()
            .returning(|_| {
                let mut mock = MockRingBuf::new();
                let mut count = Arc::new(Mutex::new(1u32));
                let events_fd = EventFD::new(0, EfdFlags::EFD_NONBLOCK).expect("should work");
                let events_raw_fd = events_fd.as_raw_fd();
                //events_fd.write(1).unwrap();

                mock
                    .expect_next()
                    .returning( move || {
                        let mut item = MockRingBufItem::new();
                        let mut guard = count.try_lock().unwrap();
                        let current_count = *guard;
                        *guard += 1;
                        item.expect_deref()
                            .return_const(current_count.to_le_bytes().to_vec());
                        events_fd.write(1).unwrap();
                        Some(item)
                    });
                mock
                    .expect_as_raw_fd()
                    .return_const(events_raw_fd);
                    
                Ok(mock)
            });
        
        
        MockingSetup {
            ebpf,
            xdp_ctx,
            rb_ctx
        }
    }
    

    #[tokio::test]
    async fn example() -> anyhow::Result<()> {
        let MockingSetup { ebpf, xdp_ctx, rb_ctx } = setup_mock_ebpf();
        
        let iface = "eth0".to_owned();
        
        let counter = Counter::new(ebpf, iface);
        
        counter.load_program("example").await?;
        let counts = counter.server_count().await?
            .take(4)
            .collect::<Result<Vec<_>, _>>().await?;
        
        assert_eq!(&counts, &[1, 2, 3, 4]);
        
        Ok(())
    }
}