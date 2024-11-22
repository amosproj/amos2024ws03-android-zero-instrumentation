// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use ringbuf::{CachingCons, SharedRb};
use ringbuf::consumer::Consumer;
use ringbuf::storage::Heap;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tonic::Status;
use tracing::debug;
use shared::ziofa::EbpfStreamObject;

pub fn bump_rlimit() {
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
}

pub async fn write_buffer_to_pipe(
    buffer: Arc<Mutex<CachingCons<Arc<SharedRb<Heap<EbpfStreamObject>>>>>>,
    pipe: Arc<Mutex<Option<Sender<Result<EbpfStreamObject, Status>>>>>,
) {
    loop {
        // buffer take
        let buffer_item = buffer
            .lock()
            .await
            .try_pop()
            .unwrap();


        // pipe put
        let mut guard =pipe
            .lock()
            .await;

        let tx_guard: &mut Sender<Result<EbpfStreamObject, Status>> = guard
            .as_mut()
            .unwrap();


        tx_guard
            .send(Ok(buffer_item))
            .await
            .expect("TODO: panic message");
    }

}