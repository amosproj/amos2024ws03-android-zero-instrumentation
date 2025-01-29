use aya_ebpf::{
    macros::map,
    maps::{Array, HashMap, LruHashMap, PerCpuArray, RingBuf},
};
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::{Equality, EventData, EventKind, FilterConfig, ProcessContext, TaskContext};

use crate::{
    cache::{Cache, TryWithCache},
    filter::{FilterConfigs, FilterEntry},
    pipeline::RawEventData,
    scratch::{ScratchSpace, ScratchValue},
};

#[map]
static PID_FILTER: HashMap<u32, Equality> = HashMap::with_max_entries(256, 0);

#[map]
static COMM_FILTER: HashMap<[u8; 16], Equality> = HashMap::with_max_entries(256, 0);

#[map]
static EXE_PATH_FILTER: HashMap<[u8; 4096], Equality> = HashMap::with_max_entries(256, 0);

#[map]
static CMDLINE_FILTER: HashMap<[u8; 256], Equality> = HashMap::with_max_entries(256, 0);

#[map]
static FILTER_CONFIG: Array<FilterConfig> = Array::with_max_entries(EventKind::MAX as u32, 0);

#[map]
static CONFIG: Array<u32> = Array::with_max_entries(1, 0);

#[map]
pub static EVENT_BUFFER: PerCpuArray<RawEventData> = PerCpuArray::with_max_entries(1, 0);

#[map]
pub static EVENT_BRIDGE: HashMap<u64, RawEventData> = HashMap::with_max_entries(10240, 0);

#[map]
pub static EVENTS: RingBuf = RingBuf::with_byte_size(8192 * 1024, 0);

#[map]
pub static GLOBAL_BLOCKING_THRESHOLD: Array<u64> = Array::with_max_entries(1, 0);

#[map]
static TASK_INFO: LruHashMap<u32, TaskContext> = LruHashMap::with_max_entries(10240, 0);

#[map]
static PROCESS_INFO: LruHashMap<u32, ProcessContext> = LruHashMap::with_max_entries(1, 0);

#[map]
static SCRATCH_MAP: PerCpuArray<(bool, [u8; 8192])> = PerCpuArray::with_max_entries(1, 0);

static SCRATCH_SPACE: ScratchSpace<[u8; 8192]> = {
    // SAFETY: The map is private and only accessible through the cache
    unsafe { ScratchSpace::new(&SCRATCH_MAP) }
};

#[map]
static SCRATCH_MAP_PATH: PerCpuArray<(bool, [u8; 8192])> = PerCpuArray::with_max_entries(1, 0);

static SCRATCH_SPACE_PATH: ScratchSpace<[u8; 8192]> = {
    // SAFETY: The map is private and only accessible through the cache
    unsafe { ScratchSpace::new(&SCRATCH_MAP_PATH) }
};

static TASK_INFO_CACHE: Cache<u32, TaskContext> = {
    // SAFETY: The map is private and only accessible through the cache
    unsafe { Cache::new(&TASK_INFO) }
};

static PROCESS_INFO_CACHE: Cache<u32, ProcessContext> = {
    // SAFETY: The map is private and only accessible through the cache
    unsafe { Cache::new(&PROCESS_INFO) }
};

static FILTER_CONFIGS: FilterConfigs = FilterConfigs::new(
    &FILTER_CONFIG,
    &CONFIG,
    &PID_FILTER,
    &COMM_FILTER,
    &EXE_PATH_FILTER,
    &CMDLINE_FILTER,
);

pub struct ScratchPath;

impl ScratchPath {
    #[inline(always)]
    pub fn get() -> Option<ScratchValue<[u8; 8192]>> {
        SCRATCH_SPACE_PATH.cast().get()
    }
}

pub struct TaskInfoCache;

impl TaskInfoCache {
    #[inline(always)]
    pub fn get(task: TaskStruct) -> Option<&'static TaskContext> {
        task.with_cache(&TASK_INFO_CACHE, SCRATCH_SPACE.cast()).ok()
    }
}

pub struct ProcessInfoCache;

impl ProcessInfoCache {
    #[inline(always)]
    pub fn get(task: TaskStruct) -> Option<&'static ProcessContext> {
        task.with_cache(&PROCESS_INFO_CACHE, SCRATCH_SPACE.cast())
            .ok()
    }
}

pub struct EventFilter;

impl EventFilter {
    pub fn filter_many<T: EventData>(entries: &[FilterEntry]) -> bool {
        FILTER_CONFIGS.filter_many::<T>(entries)
    }

    pub fn filter_one<T: EventData>(entry: &FilterEntry) -> bool {
        FILTER_CONFIGS.filter_one::<T>(entry)
    }
}
