use aya_ebpf::{
    macros::map,
    maps::{Array, HashMap, LruHashMap, PerCpuArray, RingBuf},
};
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::{Equality, EventKind, FilterConfig, ProcessContext, TaskContext};

use crate::{
    cache::{Cache, TryWithCache},
    path::PATH_MAX,
    pipeline::RawEventData,
    scratch::ScratchSpace,
};

#[map]
pub static PATH_BUF: PerCpuArray<[u8; PATH_MAX * 2]> = PerCpuArray::with_max_entries(1, 0);

#[map]
pub static EVENTS: RingBuf = RingBuf::with_byte_size(8192 * 1024, 0);

#[map]
pub static START_TIME: HashMap<u32, u64> = HashMap::with_max_entries(10240, 0);
#[map]
pub static PID_FILTER: HashMap<u32, Equality> = HashMap::with_max_entries(256, 0);

#[map]
pub static COMM_FILTER: HashMap<[u8; 16], Equality> = HashMap::with_max_entries(256, 0);

#[map]
pub static EXE_PATH_FILTER: HashMap<[u8; 4096], Equality> = HashMap::with_max_entries(256, 0);

#[map]
pub static CMDLINE_FILTER: HashMap<[u8; 256], Equality> = HashMap::with_max_entries(256, 0);

#[map]
pub static FILTER_CONFIG: Array<FilterConfig> = Array::with_max_entries(EventKind::MAX as u32, 0);

#[map]
pub static CONFIG: Array<u32> = Array::with_max_entries(1, 0);

#[map]
pub static EVENT_BUFFER: PerCpuArray<RawEventData> = PerCpuArray::with_max_entries(1, 0);

#[map]
pub static EVENT_BRIDGE: HashMap<u64, RawEventData> = HashMap::with_max_entries(10240, 0);

#[map]
pub static EVENT_RB: RingBuf = RingBuf::with_byte_size(8192 * 1024, 0);

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

static TASK_INFO_CACHE: Cache<u32, TaskContext> = {
    // SAFETY: The map is private and only accessible through the cache
    unsafe { Cache::new(&TASK_INFO) }
};

static PROCESS_INFO_CACHE: Cache<u32, ProcessContext> = {
    // SAFETY: The map is private and only accessible through the cache
    unsafe { Cache::new(&PROCESS_INFO) }
};

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
