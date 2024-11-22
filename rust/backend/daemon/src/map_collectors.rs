// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use aya::maps::{MapData, RingBuf};
use aya::Ebpf;
use ringbuf::storage::Heap;
use ringbuf::{CachingProd, SharedRb};
use shared::ziofa::{EbpfStreamObject, VfsWriteCall};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use tokio::io::unix::AsyncFd;
use tokio_stream::StreamExt;

type ServerConsType = Arc<Mutex<CachingProd<Arc<SharedRb<Heap<EbpfStreamObject>>>>>>;

fn map_as_ringbuf(ebpf: Arc<Mutex<Ebpf>>, map_name: &str) -> RingBuf<MapData>{
    ebpf
        .lock()
        .unwrap()
        .take_map(map_name) 
        .expect(format!("{map_name} map should be exported by ebpf").as_str())
        .try_into()
        .expect("VFS_TRACING should be a RingBuf")
}



async fn collect_from_all_maps(
    ebpf: Arc<Mutex<Ebpf>>,
    server_buffer: ServerConsType
){
    let name_function_tuples:
        Vec<(&str, &fn(RingBuf<MapData>, ServerConsType))> = vec![
        ("VFS_WRITE_MAP", &(vfs_tracing_taker as fn(RingBuf<MapData>, ServerConsType))),
    ];
    
    let name_function_buffer_tuples: Vec<(fn(RingBuf<MapData>, ServerConsType), RingBuf<MapData>)> = name_function_tuples
        .iter()
        .map(
            |(map_name, func)| {
                (*func, map_as_ringbuf(ebpf.clone(), *map_name))
            })
        .collect();
    
    for (func, buffer) in name_function_buffer_tuples{
        spawn(
            async move { 
                func(buffer,  server_buffer.clone()).await;   
            }
        );
    }
    
}


async fn vfs_tracing_taker(bpf_buffer: RingBuf<MapData>, server_buffer: ServerConsType ){
    let map_async_fd= AsyncFd::new(bpf_buffer)
        .unwrap();
    loop{ 
        let mut guard = match map_async_fd.readable().await {
            Ok(guard) => guard,
            Err(..) => {
                println!("[vfs_tracing_taker] failed to read from vfs map");
                break
            },
        };
        
        guard.clear_ready();

        if let Some(raw_item) = guard.get_ref().next() {
            match raw_item.try_into::<VfsWriteCall>() {
                Ok(item) => {
                    let mut server_buffer_guard = server_buffer.lock().await;
                    server_buffer_guard.push(item);
                }
                Err(e) => {
                    println!("[vfs_tracing_taker] Failed to convert item: {:?}", e);
                }
            }
        }
    }
}

 