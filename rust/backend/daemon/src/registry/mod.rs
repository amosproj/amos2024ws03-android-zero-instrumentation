// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::fs::{create_dir_all, remove_dir_all};

use crate::constants::{GC_HEAP_META_JSON, ZIOFA_EBPF_PATH};

mod pinning;
mod single_owner;

use aya::{
    maps::{Array, HashMap, MapData, MapError, RingBuf},
    programs::{ProbeKind, ProgramError, RawTracePoint, UProbe},
    Btf, EbpfError, EbpfLoader,
};
use aya_log::EbpfLogger;
use ebpf_types::{Equality, FilterConfig};
use garbage_collection::{btf::apply_to_btf, HeapMetadata};
use pinning::{LoadAndPin, PinMap, TryMapFromPin};
pub use single_owner::{RegistryGuard, RegistryItem};

pub type OwnedRingBuf = RingBuf<MapData>;
pub type OwnedHashMap<K, V> = HashMap<MapData, K, V>;
pub type OwnedArray<V> = Array<MapData, V>;

#[derive(Clone)]
pub struct EbpfRegistry {
    pub config: EbpfConfigRegistry,
    pub event: EbpfEventRegistry,
    pub program: EbpfProgramRegistry,
}

#[derive(Clone)]
pub struct EbpfConfigRegistry {
    pub pid_filter: RegistryItem<OwnedHashMap<u32, Equality>>,
    pub _comm_filter: RegistryItem<OwnedHashMap<[u8; 16], Equality>>,
    pub _exe_path_filter: RegistryItem<OwnedHashMap<[u8; 4096], Equality>>,
    pub _cmdline_filter: RegistryItem<OwnedHashMap<[u8; 256], Equality>>,
    pub global_blocking_threshold: RegistryItem<OwnedArray<u64>>,
    pub filter_config: RegistryItem<OwnedArray<FilterConfig>>,
    pub config: RegistryItem<OwnedArray<u32>>,
}

#[derive(Clone)]
pub struct EbpfEventRegistry {
    pub events: RegistryItem<OwnedRingBuf>,
}

#[derive(Clone)]
pub struct EbpfProgramRegistry {
    pub sys_enter_write: RegistryItem<RawTracePoint>,
    pub sys_exit_write: RegistryItem<RawTracePoint>,
    pub sys_enter_blocking: RegistryItem<RawTracePoint>,
    pub sys_exit_blocking: RegistryItem<RawTracePoint>,
    pub sys_enter_signal: RegistryItem<RawTracePoint>,
    pub sys_exit_signal: RegistryItem<RawTracePoint>,
    pub sys_enter_fdtracking: RegistryItem<RawTracePoint>,
    pub sys_exit_fdtracking: RegistryItem<RawTracePoint>,
    pub trace_jni_add_local: RegistryItem<UProbe>,
    pub trace_jni_del_local: RegistryItem<UProbe>,
    pub trace_jni_add_global: RegistryItem<UProbe>,
    pub trace_jni_del_global: RegistryItem<UProbe>,
    pub trace_gc_enter: RegistryItem<UProbe>,
    pub trace_gc_exit: RegistryItem<UProbe>,
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
            pid_filter: HashMap::<_, u32, Equality>::try_from_pin(path("PID_FILTER"))?.into(),
            _comm_filter: HashMap::<_, [u8; 16], Equality>::try_from_pin(path("COMM_FILTER"))?
                .into(),
            _exe_path_filter: HashMap::<_, [u8; 4096], Equality>::try_from_pin(path(
                "EXE_PATH_FILTER",
            ))?
            .into(),
            _cmdline_filter: HashMap::<_, [u8; 256], Equality>::try_from_pin(path(
                "CMDLINE_FILTER",
            ))?
            .into(),
            global_blocking_threshold: Array::<_, u64>::try_from_pin(path(
                "GLOBAL_BLOCKING_THRESHOLD",
            ))?
            .into(),
            filter_config: Array::<_, FilterConfig>::try_from_pin(path("FILTER_CONFIG"))?.into(),
            config: Array::<_, u32>::try_from_pin(path("CONFIG"))?.into(),
        })
    }
}

impl EbpfEventRegistry {
    fn from_pin() -> Result<Self, MapError> {
        Ok(Self {
            events: RingBuf::try_from_pin(path("EVENT_RB"))?.into(),
        })
    }
}

impl EbpfProgramRegistry {
    fn from_pin() -> Result<Self, ProgramError> {
        Ok(Self {
            sys_enter_write: RawTracePoint::from_pin(path("sys_enter_write"))?.into(),
            sys_exit_write: RawTracePoint::from_pin(path("sys_exit_write"))?.into(),
            sys_enter_blocking: RawTracePoint::from_pin(path("sys_enter_blocking"))?.into(),
            sys_exit_blocking: RawTracePoint::from_pin(path("sys_exit_blocking"))?.into(),
            sys_enter_signal: RawTracePoint::from_pin(path("sys_enter_signal"))?.into(),
            sys_exit_signal: RawTracePoint::from_pin(path("sys_exit_signal"))?.into(),
            sys_enter_fdtracking: RawTracePoint::from_pin(path("sys_enter_fdtracking"))?.into(),
            sys_exit_fdtracking: RawTracePoint::from_pin(path("sys_exit_fdtracking"))?.into(),
            trace_jni_add_local: UProbe::from_pin(path("trace_jni_add_local"), ProbeKind::UProbe)?
                .into(),
            trace_jni_del_local: UProbe::from_pin(path("trace_jni_del_local"), ProbeKind::UProbe)?
                .into(),
            trace_jni_add_global: UProbe::from_pin(
                path("trace_jni_add_global"),
                ProbeKind::UProbe,
            )?
            .into(),
            trace_jni_del_global: UProbe::from_pin(
                path("trace_jni_del_global"),
                ProbeKind::UProbe,
            )?
            .into(),
            trace_gc_enter: UProbe::from_pin(path("trace_gc_enter"), ProbeKind::UProbe)?.into(),
            trace_gc_exit: UProbe::from_pin(path("trace_gc_exit"), ProbeKind::URetProbe)?.into(),
        })
    }
}

pub fn load_and_pin() -> Result<EbpfRegistry, EbpfError> {
    // TODO: better map dir handling
    let _ = remove_dir_all(ZIOFA_EBPF_PATH);
    create_dir_all(ZIOFA_EBPF_PATH).unwrap();

    let mut btf = Btf::from_sys_fs()?;
    let heap_meta =
        serde_json::from_str::<HeapMetadata>(GC_HEAP_META_JSON).expect("valid heap metadata");

    apply_to_btf(&mut btf, &heap_meta)?;

    let mut ebpf = EbpfLoader::default()
        .btf(Some(&btf))
        .map_pin_path(ZIOFA_EBPF_PATH)
        .load(aya::include_bytes_aligned!(concat!(
            env!("OUT_DIR"),
            "/ebpf-refactored"
        )))
        .unwrap();

    ebpf.pin_map("PID_FILTER", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("COMM_FILTER", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("EXE_PATH_FILTER", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("CMDLINE_FILTER", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("FILTER_CONFIG", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("CONFIG", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("EVENT_RB", ZIOFA_EBPF_PATH).unwrap();
    ebpf.pin_map("GLOBAL_BLOCKING_THRESHOLD", ZIOFA_EBPF_PATH)
        .unwrap();

    EbpfLogger::init(&mut ebpf).unwrap();

    ebpf.load_and_pin::<RawTracePoint>("sys_enter_write", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_exit_write", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_enter_blocking", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_exit_blocking", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_enter_signal", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_exit_signal", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_enter_fdtracking", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<RawTracePoint>("sys_exit_fdtracking", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<UProbe>("trace_jni_add_local", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<UProbe>("trace_jni_del_local", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<UProbe>("trace_jni_add_global", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<UProbe>("trace_jni_del_global", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<UProbe>("trace_gc_enter", ZIOFA_EBPF_PATH)
        .unwrap();
    ebpf.load_and_pin::<UProbe>("trace_gc_exit", ZIOFA_EBPF_PATH)
        .unwrap();

    EbpfRegistry::from_pin()
}

fn path(name: &str) -> String {
    format!("{ZIOFA_EBPF_PATH}/{name}")
}
