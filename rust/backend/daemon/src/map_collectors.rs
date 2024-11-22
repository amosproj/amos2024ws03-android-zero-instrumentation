// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};
use aya::Ebpf;
use aya::maps::{MapData, RingBuf};
use ringbuf::{CachingProd, SharedRb};
use ringbuf::storage::Heap;
use shared::ziofa::EbpfStreamObject;

fn map_as_ringbuf(ebpf: Arc<Mutex<Ebpf>>, map_name: &str) -> RingBuf<MapData>{
    ebpf
        .lock()
        .unwrap()
        .take_map(map_name) 
        .expect(format!("{map_name} map should be exported by ebpf").as_str())
        .try_into()
        .expect("VFS_TRACING should be a RingBuf")
}

type ServerConsType = fn(RingBuf<MapData>, Arc<Mutex<CachingProd<Arc<SharedRb<Heap<EbpfStreamObject>>>>>>);

async fn collect_from_all_maps(
    ebpf: Arc<Mutex<Ebpf>>,
    server_buffer: ServerConsType
){
    let name_function_tuples:
        Vec<(&str, &fn(RingBuf<MapData>, ServerConsType))> = vec![
        ("VFS_WRITE_MAP", &(vfs_tracing_taker as fn(RingBuf<MapData>, ServerConsType))),
    ];
    
    let name_function_buffer_tuples: Vec<(&str, fn(RingBuf<MapData>, ServerConsType), RingBuf<MapData>)> = name_function_tuples
        .iter()
        .map(
            |(map_name, func)| {
                (*map_name, *func, map_as_ringbuf(ebpf.clone(), *map_name))
            })
        .collect();
    
    for (_, func, buffer) in name_function_buffer_tuples{
        func(buffer, server_buffer)
    }
    
}


fn vfs_tracing_taker(bpf_buffer: RingBuf<MapData>, server_buffer: ServerConsType ){
    
}

