// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::ptr::copy_nonoverlapping;

#[cfg(bpf_target_arch = "x86_64")]
use aya_ebpf::bindings::pt_regs;

#[cfg(bpf_target_arch = "aarch64")]
use aya_ebpf::bindings::user_pt_regs as pt_regs;

use aya_ebpf::{
    helpers::{bpf_get_current_task, bpf_ktime_get_ns, bpf_probe_read_user},
    macros::{raw_tracepoint, uprobe, uretprobe},
    programs::{ProbeContext, RawTracePointContext, RetProbeContext},
    EbpfContext, PtRegs,
};
use ebpf_relocation_helpers::{ffi::art_heap, ArtHeap, TaskStruct};
use ebpf_types::{
    Blocking, Event, EventData, FileDescriptorChange, GarbageCollect, JniReferences,
    ProcessContext, Signal, TaskContext, Write,
};

use crate::{
    event_local::{EventLocal, EventLocalData, EventLocalValue},
    events::SyscallProg,
    filter::FilterEntry,
    maps::{
        EventFilter, EventStorage, ProcessInfoCache, ScratchEventLocal, TaskInfoCache, EVENTS,
        GLOBAL_BLOCKING_THRESHOLD,
    },
    scratch::ScratchValue,
};

#[derive(Clone, Copy)]
pub struct ProgramInfo<'a> {
    pub task_context: &'a TaskContext,
    pub process_context: &'a ProcessContext,
}

impl ProgramInfo<'_> {
    #[inline(always)]
    pub fn new(task: TaskStruct) -> Option<Self> {
        Some(Self {
            task_context: TaskInfoCache::get(task)?,
            process_context: ProcessInfoCache::get(task)?,
        })
    }

    fn filters(&self) -> [FilterEntry; 6] {
        [
            FilterEntry::OwnPid(&self.task_context.pid),
            FilterEntry::Pid(&self.task_context.pid),
            FilterEntry::Tid(&self.task_context.tid),
            FilterEntry::Comm(&self.task_context.comm),
            FilterEntry::ExePath(&self.process_context.exe_path),
            FilterEntry::Cmdline(&self.process_context.cmdline),
        ]
    }

    fn submit<T: EventData + 'static>(self, event: &T) -> Option<()> {
        let mut entry = EVENTS.reserve::<Event<T>>(0)?;
        let ptr = entry.as_mut_ptr();

        unsafe {
            (&raw mut (*ptr).kind).write(T::EVENT_KIND);
            (&raw mut (*ptr).context.timestamp).write(bpf_ktime_get_ns());
            copy_nonoverlapping(self.task_context, &raw mut (*ptr).context.task, 1);
            copy_nonoverlapping(event, &raw mut (*ptr).data, 1);
        }

        entry.submit(0);

        Some(())
    }
}

struct ProgramInfoEntry<'a, T: EventLocalData + 'static> {
    info: ProgramInfo<'a>,
    event: ScratchValue<EventLocal<T>>,
}

impl<T: EventLocalData + 'static> ProgramInfoEntry<'_, T> {
    #[inline(always)]
    pub fn new(task: TaskStruct) -> Option<Self> {
        Some(Self {
            info: ProgramInfo::new(task)?,
            event: ScratchEventLocal::get::<EventLocal<T>>()?,
        })
    }
}

struct ProgramInfoExit<'a, T: EventLocalData + 'static> {
    info: ProgramInfo<'a>,
    event_entry: EventLocalValue<T>,
}

impl<T: EventLocalData + 'static> ProgramInfoExit<'_, T> {
    #[inline(always)]
    pub fn new(task: TaskStruct) -> Option<Self> {
        let info = ProgramInfo::new(task)?;
        Some(Self {
            info,
            event_entry: EventStorage::get::<T>(info.task_context.tid).ok()?,
        })
    }
}

unsafe fn current_task() -> TaskStruct {
    TaskStruct::new(bpf_get_current_task() as *mut _)
}

unsafe fn sys_enter_sycall_id(ctx: &RawTracePointContext) -> i64 {
    *(ctx.as_ptr().add(8) as *mut i64)
}

unsafe fn sys_pt_regs(ctx: &RawTracePointContext) -> PtRegs {
    PtRegs::new(*(ctx.as_ptr() as *const *mut pt_regs))
}

unsafe fn sys_exit_return_value(ctx: &RawTracePointContext) -> u64 {
    *(ctx.as_ptr().add(8) as *mut u64)
}

pub struct SysEnterInfo {
    pub task: TaskStruct,
    pub syscall_id: i64,
    pub pt_regs: PtRegs,
}

impl SysEnterInfo {
    fn new(ctx: &RawTracePointContext) -> Self {
        unsafe {
            Self {
                task: current_task(),
                syscall_id: sys_enter_sycall_id(ctx),
                pt_regs: sys_pt_regs(ctx),
            }
        }
    }
}

pub struct SysExitInfo {
    pub task: TaskStruct,
    pub return_value: u64,
    pub pt_regs: PtRegs,
}

impl SysExitInfo {
    fn new(ctx: &RawTracePointContext) -> Self {
        unsafe {
            Self {
                task: current_task(),
                return_value: sys_exit_return_value(ctx),
                pt_regs: sys_pt_regs(ctx),
            }
        }
    }
}

fn sys_enter<P: SyscallProg>(ctx: &RawTracePointContext) -> Option<()> {
    let enter_info = SysEnterInfo::new(ctx);
    let mut program_info = ProgramInfoEntry::new(enter_info.task)?;

    let data = P::enter(enter_info, program_info.info, &mut program_info.event)?;

    EventStorage::set(program_info.info.task_context.tid, data).ok()
}

fn sys_exit<P: SyscallProg>(
    ctx: &RawTracePointContext,
    filter: impl Fn(&P) -> Option<()>,
) -> Option<()> {
    let exit_info = SysExitInfo::new(ctx);
    let program_info = ProgramInfoExit::new(exit_info.task)?;

    if EventFilter::filter_many::<P>(&program_info.info.filters()) {
        return None;
    }

    let mut event = ScratchEventLocal::get()?;
    let event = P::exit(
        exit_info,
        program_info.info,
        &program_info.event_entry,
        &mut event,
    )?;

    filter(event)?;

    program_info.info.submit(event)
}

#[raw_tracepoint]
pub fn sys_enter_write(ctx: RawTracePointContext) -> Option<()> {
    sys_enter::<Write>(&ctx)
}

#[raw_tracepoint]
pub fn sys_exit_write(ctx: RawTracePointContext) -> Option<()> {
    sys_exit::<Write>(&ctx, |_| Some(()))
}

#[raw_tracepoint]
pub fn sys_enter_blocking(ctx: RawTracePointContext) -> Option<()> {
    sys_enter::<Blocking>(&ctx)
}

#[raw_tracepoint]
pub fn sys_exit_blocking(ctx: RawTracePointContext) -> Option<()> {
    sys_exit::<Blocking>(&ctx, |event| {
        if let Some(threshold) = GLOBAL_BLOCKING_THRESHOLD.get(0) {
            if event.duration <= *threshold {
                return None;
            }
        }
        Some(())
    })
}

#[raw_tracepoint]
pub fn sys_enter_signal(ctx: RawTracePointContext) -> Option<()> {
    sys_enter::<Signal>(&ctx)
}

#[raw_tracepoint]
pub fn sys_exit_signal(ctx: RawTracePointContext) -> Option<()> {
    sys_exit::<Signal>(&ctx, |_| Some(()))
}

#[raw_tracepoint]
pub fn sys_enter_fdtracking(ctx: RawTracePointContext) -> Option<()> {
    sys_enter::<FileDescriptorChange>(&ctx)
}

#[raw_tracepoint]
pub fn sys_exit_fdtracking(ctx: RawTracePointContext) -> Option<()> {
    sys_exit::<FileDescriptorChange>(&ctx, |_| Some(()))
}

unsafe fn trace_jni_enter(data: JniReferences) -> Option<()> {
    let task = current_task();
    let program_info = ProgramInfo::new(task)?;

    if EventFilter::filter_many::<JniReferences>(&program_info.filters()) {
        return None;
    }

    let mut event = ScratchEventLocal::get::<JniReferences>()?;
    event.as_mut_ptr().write(data);
    let event = event.assume_init_ref();

    program_info.submit(event)
}

#[uprobe]
fn trace_jni_add_local(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(JniReferences::AddLocalRef) }
}
#[uprobe]
fn trace_jni_del_local(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(JniReferences::DeleteLocalRef) }
}
#[uprobe]
fn trace_jni_add_global(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(JniReferences::AddGlobalRef) }
}
#[uprobe]
fn trace_jni_del_global(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(JniReferences::DeleteGlobalRef) }
}

impl EventLocalData for GarbageCollect {
    type Data = ArtHeap;
}

#[uprobe]
fn trace_gc_enter(ctx: ProbeContext) -> Option<()> {
    let task = unsafe { current_task() };
    let mut program_info = ProgramInfoEntry::<GarbageCollect>::new(task)?;

    let ptr = program_info.event.as_mut_ptr();
    let data = unsafe {
        (&raw mut (*ptr).data).write(ArtHeap::new(ctx.arg::<*mut art_heap>(0)?));
        program_info.event.assume_init_mut()
    };

    EventStorage::set(program_info.info.task_context.tid, data).ok()
}

#[uretprobe]
fn trace_gc_exit(_: RetProbeContext) -> Option<()> {
    let task = unsafe { current_task() };
    let program_info = ProgramInfoExit::<GarbageCollect>::new(task)?;

    if EventFilter::filter_many::<GarbageCollect>(&program_info.info.filters()) {
        return None;
    }

    let mut event = ScratchEventLocal::get::<GarbageCollect>()?;
    let event = unsafe {
        let heap = program_info.event_entry.data;

        #[cfg(bpf_target_arch = "aarch64")]
        #[inline(always)]
        unsafe fn read<T>(ptr: *mut T) -> Option<T> {
            bpf_probe_read_user(((ptr as usize) & 0xffffffffff) as *const T).ok()
        }

        #[cfg(bpf_target_arch = "x86_64")]
        #[inline(always)]
        unsafe fn read<T>(ptr: *mut T) -> Option<T> {
            bpf_probe_read_user(ptr).ok()
        }

        event.as_mut_ptr().write(GarbageCollect {
            target_footprint: read(heap.target_footprint())?,
            num_bytes_allocated: read(heap.num_bytes_allocated())?,
            gc_cause: read(heap.gc_cause())?,
            duration_ns: read(heap.duration_ns())?,
            freed_objects: read(heap.freed_objects())?,
            freed_bytes: read(heap.freed_bytes())?,
            freed_los_objects: read(heap.freed_los_objects())?,
            freed_los_bytes: read(heap.freed_los_bytes())?,
            gcs_completed: read(heap.gcs_completed())?,
        });

        event.assume_init_ref()
    };

    program_info.info.submit(event)
}