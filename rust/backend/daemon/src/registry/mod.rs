// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::fs::{create_dir_all, remove_dir_all};

use crate::constants::{GC_HEAP_META_JSON, ZIOFA_EBPF_PATH};

mod pinning;
mod single_owner;
mod typed_ringbuf;

use aya::{maps::{HashMap, MapData, MapError, RingBuf}, programs::{KProbe, ProbeKind, ProgramError, TracePoint, UProbe}, EbpfError, EbpfLoader};
use aya_log::EbpfLogger;
use backend_common::{JNICall, SysFdActionCall, SysGcCall, SysSendmsgCall, SysSigquitCall, VfsWriteCall};
use bytemuck::bytes_of;
use garbage_collection::HeapMetadata;
use pinning::{LoadAndPin, TryMapFromPin};
pub use typed_ringbuf::TypedRingBuffer;
pub use single_owner::{RegistryGuard, RegistryItem};

pub type OwnedRingBuf = RingBuf<MapData>;
pub type OwnedHashMap<K, V> = HashMap<MapData, K, V>;

#[derive(Clone)]
pub struct EbpfRegistry {
    pub config: EbpfConfigRegistry,
    pub event: EbpfEventRegistry,
    pub program: EbpfProgramRegistry,
}

#[derive(Clone)]
pub struct EbpfConfigRegistry {
    pub vfs_write_pids: RegistryItem<OwnedHashMap<u32, u64>>,
    pub sys_sendmsg_pids: RegistryItem<OwnedHashMap<u32, u64>>,
    pub jni_ref_pids: RegistryItem<OwnedHashMap<u32, u64>>,
    pub sys_sigquit_pids: RegistryItem<OwnedHashMap<u32, u64>>,
    pub sys_fd_tracking_pids: RegistryItem<OwnedHashMap<u32, u64>>,
}

#[derive(Clone)]
pub struct EbpfEventRegistry {
    pub vfs_write_events: RegistryItem<TypedRingBuffer<VfsWriteCall>>,
    pub sys_sendmsg_events: RegistryItem<TypedRingBuffer<SysSendmsgCall>>,
    pub jni_ref_calls: RegistryItem<TypedRingBuffer<JNICall>>,
    pub sys_sigquit_events: RegistryItem<TypedRingBuffer<SysSigquitCall>>,
    pub gc_events: RegistryItem<TypedRingBuffer<SysGcCall>>,
    pub sys_fd_tracking_events: RegistryItem<TypedRingBuffer<SysFdActionCall>>,
}

#[derive(Clone)]
pub struct EbpfProgramRegistry {
    pub vfs_write: RegistryItem<KProbe>,
    pub vfs_write_ret: RegistryItem<KProbe>,
    pub sys_enter_sendmsg: RegistryItem<TracePoint>,
    pub sys_exit_sendmsg: RegistryItem<TracePoint>,
    pub trace_add_local: RegistryItem<UProbe>,
    pub trace_del_local: RegistryItem<UProbe>,
    pub trace_add_global: RegistryItem<UProbe>,
    pub trace_del_global: RegistryItem<UProbe>,
    pub sys_sigquit: RegistryItem<TracePoint>,
    pub collect_garbage_internal: RegistryItem<UProbe>,
    pub collect_garbage_internal_ret: RegistryItem<UProbe>,
    pub sys_create_fd: RegistryItem<TracePoint>,
    pub sys_destroy_fd: RegistryItem<TracePoint>,
}

impl EbpfRegistry {
    fn from_pin() -> Result<Self, EbpfError> {
        Ok(Self {
            config: EbpfConfigRegistry::from_pin()?,
            event: EbpfEventRegistry::from_pin()?,
            program: EbpfProgramRegistry::from_pin()?,
        })
    }
}

impl EbpfConfigRegistry {
    fn from_pin() -> Result<Self, MapError> {
        Ok(Self {
            vfs_write_pids: HashMap::<_, u32, u64>::try_from_pin(path("VFS_WRITE_PIDS"))?.into(),
            sys_sendmsg_pids: HashMap::<_, u32, u64>::try_from_pin(path("SYS_SENDMSG_PIDS"))?.into(),
            jni_ref_pids: HashMap::<_, u32, u64>::try_from_pin(path("JNI_REF_PIDS"))?.into(),
            sys_sigquit_pids: HashMap::<_, u32, u64>::try_from_pin(path("SYS_SIGQUIT_PIDS"))?.into(),
            sys_fd_tracking_pids: HashMap::<_, u32, u64>::try_from_pin(path("SYS_FDTRACKING_PIDS"))?.into(),
        })
    }
}

impl EbpfEventRegistry {
    fn from_pin() -> Result<Self, MapError> {
        Ok(Self {
            vfs_write_events: RingBuf::try_from_pin(path("VFS_WRITE_EVENTS"))?.into(),
            sys_sendmsg_events: RingBuf::try_from_pin(path("SYS_SENDMSG_EVENTS"))?.into(),
            jni_ref_calls: RingBuf::try_from_pin(path("JNI_REF_CALLS"))?.into(),
            sys_sigquit_events: RingBuf::try_from_pin(path("SYS_SIGQUIT_EVENTS"))?.into(),
            gc_events: RingBuf::try_from_pin(path("GC_EVENTS"))?.into(),
            sys_fd_tracking_events: RingBuf::try_from_pin(path("SYS_FDTRACKING_EVENTS"))?.into(),
        })
    }
}

impl EbpfProgramRegistry {
    fn from_pin() -> Result<Self, ProgramError> {
        Ok(Self {
            vfs_write: KProbe::from_pin(path("vfs_write"), ProbeKind::KProbe)?.into(),
            vfs_write_ret: KProbe::from_pin(path("vfs_write_ret"), ProbeKind::KRetProbe)?.into(),
            sys_enter_sendmsg: TracePoint::from_pin(path("sys_enter_sendmsg"))?.into(),
            sys_exit_sendmsg: TracePoint::from_pin(path("sys_exit_sendmsg"))?.into(),
            trace_add_local: UProbe::from_pin(path("trace_add_local"), ProbeKind::UProbe)?.into(),
            trace_del_local: UProbe::from_pin(path("trace_del_local"), ProbeKind::UProbe)?.into(),
            trace_add_global: UProbe::from_pin(path("trace_add_global"), ProbeKind::UProbe)?.into(),
            trace_del_global: UProbe::from_pin(path("trace_del_global"), ProbeKind::UProbe)?.into(),
            sys_sigquit: TracePoint::from_pin(path("sys_sigquit"))?.into(),
            collect_garbage_internal: UProbe::from_pin(path("collect_garbage_internal"), ProbeKind::UProbe)?.into(),
            collect_garbage_internal_ret: UProbe::from_pin(path("collect_garbage_internal_ret"), ProbeKind::URetProbe)?.into(),
            sys_create_fd: TracePoint::from_pin(path("sys_create_fd"))?.into(),
            sys_destroy_fd: TracePoint::from_pin(path("sys_destroy_fd"))?.into(),
        })
    }
}

pub fn load_and_pin() -> Result<EbpfRegistry, EbpfError> {
    // TODO: better map dir handling
    let _ = remove_dir_all(ZIOFA_EBPF_PATH);
    create_dir_all(ZIOFA_EBPF_PATH).unwrap();
    
    let heap_meta = Some(serde_json::from_str::<HeapMetadata>(GC_HEAP_META_JSON).expect("valid heap metadata"));

    let mut ebpf = EbpfLoader::default()
        .set_global("GC_HEAP_META", bytes_of(&heap_meta), true)
        .map_pin_path(ZIOFA_EBPF_PATH)
        .load(aya::include_bytes_aligned!(concat!(
            env!("OUT_DIR"),
            "/backend-ebpf"
        )))
        .unwrap();
    
    EbpfLogger::init(&mut ebpf).unwrap();
    
    ebpf.load_and_pin::<KProbe>("vfs_write", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<KProbe>("vfs_write_ret", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<TracePoint>("sys_enter_sendmsg", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<TracePoint>("sys_exit_sendmsg", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<UProbe>("trace_add_local", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<UProbe>("trace_del_local", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<UProbe>("trace_add_global", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<UProbe>("trace_del_global", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<TracePoint>("sys_sigquit", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<UProbe>("collect_garbage_internal", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<UProbe>("collect_garbage_internal_ret", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<TracePoint>("sys_create_fd", ZIOFA_EBPF_PATH).unwrap();
    ebpf.load_and_pin::<TracePoint>("sys_destroy_fd", ZIOFA_EBPF_PATH).unwrap();

    EbpfRegistry::from_pin()
}

fn path(name: &str) -> String {
    format!("{ZIOFA_EBPF_PATH}/{name}")
}
