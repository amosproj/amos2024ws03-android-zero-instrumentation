// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut}, ptr::null_mut,
};

use aya_ebpf::{
    bindings::pt_regs,
    helpers::{bpf_get_current_task, bpf_ktime_get_ns},
    macros::{raw_tracepoint, uprobe, uretprobe},
    programs::{ProbeContext, RawTracePointContext, RetProbeContext},
    EbpfContext, PtRegs,
};
use aya_log_ebpf::info;
use ebpf_relocation_helpers::{ffi::art_heap, ArtHeap, TaskStruct};
use ebpf_types::{
    Blocking, Event, EventContext, EventData, FileDescriptorChange, GarbageCollect,
    Jni, ProcessContext, Signal, TaskContext, Write,
};

use crate::{
    events::{
        blocking::{initialize_blocking_enter, initialize_blocking_exit},
        fdtracking::{initialize_fdtracking_enter, initialize_fdtracking_exit},
        signal::initialize_signal_enter,
        write::{initialize_write_enter, initialize_write_exit},
    },
    filter::FilterEntry,
    maps::{
        EventFilter, ProcessInfoCache, TaskInfoCache, EVENTS,
        GLOBAL_BLOCKING_THRESHOLD,
    },
};

struct ProgramInfo<'a, T> {
    task_context: &'a TaskContext,
    process_context: &'a ProcessContext,
    event_info: EventInfo<T>,
}

impl<T: EventData + Clone + Copy + 'static> ProgramInfo<'_, T> {
    fn submit(self) -> Option<()> {
        let mut entry = EVENTS.reserve::<Event<T>>(0)?;
        let entry_mut = unsafe { entry.assume_init_mut() };
        entry_mut.kind = T::EVENT_KIND;

        entry_mut.context = EventContext {
            timestamp: unsafe { bpf_ktime_get_ns() },
            task: *self.task_context,
        };
        entry_mut.data = *self.event_info;
        entry.submit(0);

        Some(())
    }

    fn submit_intermediate(&self) -> Option<()> {
        self.event_info.set_initialized(true);

        Some(())
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
}

const fn max_raw_event_data_size() -> usize {
    static SIZES: [usize; 4] = [
        size_of::<Write>(),
        size_of::<Blocking>(),
        size_of::<FileDescriptorChange>(),
        size_of::<Signal>(),
    ];

    let mut max_size = SIZES[0];
    let mut i = 1;
    while i < SIZES.len() {
        if SIZES[i] > max_size {
            max_size = SIZES[i]
        };

        i += 1;
    }

    max_size -= 1;
    max_size |= max_size >> 1;
    max_size |= max_size >> 2;
    max_size |= max_size >> 4;
    max_size |= max_size >> 8;
    max_size |= max_size >> 16;
    max_size |= max_size >> 32;

    max_size + 1
}

#[repr(C)]
struct EventInfo<T> {
    raw_event_data: *mut RawEventData,
    _t: PhantomData<T>,
}

impl<T> Deref for EventInfo<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*((*self.raw_event_data).data.as_ptr() as *const T) }
    }
}

impl<T> DerefMut for EventInfo<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *((*self.raw_event_data).data.as_mut_ptr() as *mut T) }
    }
}

impl<T> EventInfo<T> {
    pub fn set_initialized(&self, initialized: bool) {
        unsafe { (*self.raw_event_data).initialized = initialized }
    }
    pub fn get_initialized(&self) -> bool {
        unsafe { (*self.raw_event_data).initialized }
    }
}

#[repr(C)]
pub struct RawEventData {
    initialized: bool,
    data: [u8; max_raw_event_data_size()],
}

fn event_bridge_key<T: EventData>(task: TaskStruct) -> Option<u64> {
    let tid = task.pid().ok()? as u64;
    let event_id = T::EVENT_KIND as u64;

    Some((tid << 32) | event_id)
}

unsafe fn program_info_base<T: EventData>(
    task: TaskStruct,
    raw_event_data: *mut RawEventData,
) -> Option<ProgramInfo<'static, T>> {
    let event_info = EventInfo {
        raw_event_data,
        _t: PhantomData,
    };
    Some(ProgramInfo {
        task_context: TaskInfoCache::get(task)?,
        process_context: ProcessInfoCache::get(task)?,
        event_info,
    })
}

unsafe fn program_info<T: EventData>(task: TaskStruct) -> Option<ProgramInfo<'static, T>> {
    let key = event_bridge_key::<T>(task)?;
    /* 
    let raw_event_data = match EVENT_BRIDGE.get_ptr_mut(&key) {
        Some(bridge) => bridge,
        None => {
            let raw_event_data = EVENT_BUFFER.get_ptr_mut(0)?;
            EVENT_BRIDGE.insert(&key, &*raw_event_data, 0).ok()?;
            EVENT_BRIDGE.get_ptr_mut(&key)?
        }
    };
    */
    let raw_event_data = null_mut();
    let program_info = program_info_base::<T>(task, raw_event_data)?;
    program_info.event_info.set_initialized(false);
    Some(program_info)
}

unsafe fn program_info_intermediate<T: EventData>(
    task: TaskStruct,
) -> Option<ProgramInfo<'static, T>> {
    let key = event_bridge_key::<T>(task)?;
    //let raw_event_data = EVENT_BRIDGE.get_ptr_mut(&key)?;
    let raw_event_data = null_mut();
    let program_info = program_info_base::<T>(task, raw_event_data)?;
    if !program_info.event_info.get_initialized() {
        return None;
    };
    Some(program_info)
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

struct SysEnterInfo {
    task: TaskStruct,
    syscall_id: i64,
    pt_regs: PtRegs,
}

impl SysEnterInfo {
    unsafe fn new(ctx: &RawTracePointContext) -> Self {
        Self {
            task: current_task(),
            syscall_id: sys_enter_sycall_id(ctx),
            pt_regs: sys_pt_regs(ctx),
        }
    }
    unsafe fn program_info<T: EventData + Copy + 'static>(
        &self,
    ) -> Option<ProgramInfo<'static, T>> {
        program_info::<T>(self.task)
    }
}

struct SysExitInfo {
    task: TaskStruct,
    return_value: u64,
    _pt_regs: PtRegs,
}

impl SysExitInfo {
    unsafe fn new(ctx: &RawTracePointContext) -> Self {
        Self {
            task: current_task(),
            return_value: sys_exit_return_value(ctx),
            _pt_regs: sys_pt_regs(ctx),
        }
    }
    unsafe fn program_info<T: EventData + Copy + 'static>(
        &self,
    ) -> Option<ProgramInfo<'static, T>> {
        program_info_intermediate::<T>(self.task)
    }
}

#[raw_tracepoint]
pub fn sys_enter_write(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let enter_info = SysEnterInfo::new(&ctx);
        let mut program_info = enter_info.program_info::<Write>()?;

        if EventFilter::filter_many::<Write>(&program_info.filters()) {
            return None;
        }

        initialize_write_enter(
            enter_info.syscall_id,
            enter_info.pt_regs,
            &mut program_info.event_info,
        )?;

        info!(&ctx, "{}", program_info.event_info.bytes_written);

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_write(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        let mut program_info = exit_info.program_info::<Write>()?;

        if EventFilter::filter_many::<Write>(&program_info.filters()) {
            return None;
        }

        initialize_write_exit(exit_info.task, &mut program_info.event_info)?;

        program_info.submit()?;
    }
    Some(())
}

#[raw_tracepoint]
pub fn sys_enter_blocking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let enter_info = SysEnterInfo::new(&ctx);
        let mut program_info = enter_info.program_info::<Blocking>()?;

        if EventFilter::filter_many::<Blocking>(&program_info.filters()) {
            return None;
        }

        initialize_blocking_enter(enter_info.syscall_id, &mut program_info.event_info)?;

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_blocking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        let mut program_info = exit_info.program_info::<Blocking>()?;

        if EventFilter::filter_many::<Blocking>(&program_info.filters()) {
            return None;
        }

        initialize_blocking_exit(&mut program_info.event_info)?;

        if let Some(threshold) = GLOBAL_BLOCKING_THRESHOLD.get(0) {
            if program_info.event_info.duration <= *threshold {
                return None;
            }
        }

        program_info.submit()?;
    }
    Some(())
}

#[raw_tracepoint]
pub fn sys_enter_signal(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let enter_info = SysEnterInfo::new(&ctx);
        let mut program_info = enter_info.program_info::<Signal>()?;

        if EventFilter::filter_many::<Signal>(&program_info.filters()) {
            return None;
        }

        initialize_signal_enter(
            enter_info.syscall_id,
            enter_info.pt_regs,
            &mut program_info.event_info,
        )?;

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_signal(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        if exit_info.return_value != 0 {
            return None;
        }

        let program_info = exit_info.program_info::<Signal>()?;

        if EventFilter::filter_many::<Signal>(&program_info.filters()) {
            return None;
        }

        program_info.submit()?;
    }
    Some(())
}

#[raw_tracepoint]
pub fn sys_enter_fdtracking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let enter_info = SysEnterInfo::new(&ctx);
        let mut program_info = enter_info.program_info::<FileDescriptorChange>()?;

        if EventFilter::filter_many::<FileDescriptorChange>(&program_info.filters()) {
            return None;
        }

        initialize_fdtracking_enter(enter_info.syscall_id, &mut program_info.event_info)?;

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_fdtracking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        let mut program_info = exit_info.program_info::<FileDescriptorChange>()?;

        if EventFilter::filter_many::<FileDescriptorChange>(&program_info.filters()) {
            return None;
        }

        initialize_fdtracking_exit(exit_info.task, &mut program_info.event_info)?;

        program_info.submit()?;
    }
    Some(())
}

unsafe fn trace_jni_enter(data: Jni) -> Option<()> {
    let task = current_task();
    let mut program_info = program_info::<Jni>(task)?;

    if EventFilter::filter_many::<Jni>(&program_info.filters()) {
        return None;
    }

    *program_info.event_info = data;

    program_info.submit()?;
    Some(())
}

#[uprobe]
fn trace_jni_add_local(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(Jni::AddLocalRef) }
}
#[uprobe]
fn trace_jni_del_local(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(Jni::DeleteLocalRef) }
}
#[uprobe]
fn trace_jni_add_global(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(Jni::AddGlobalRef) }
}
#[uprobe]
fn trace_jni_del_global(_: ProbeContext) -> Option<()> {
    unsafe { trace_jni_enter(Jni::DeleteGlobalRef) }
}

#[uprobe]
fn trace_gc_enter(ctx: ProbeContext) -> Option<()> {
    unsafe {
        let task = current_task();
        let program_info = program_info::<GarbageCollect>(task)?;

        if EventFilter::filter_many::<GarbageCollect>(&program_info.filters()) {
            return None;
        }

        let heap = ArtHeap::new(ctx.arg::<*mut art_heap>(0)?);
        (program_info.event_info.raw_event_data as *mut ArtHeap).write(heap);

        program_info.submit_intermediate()?;
    }
    Some(())
}

#[uretprobe]
fn trace_gc_exit(_: RetProbeContext) -> Option<()> {
    unsafe {
        let task = current_task();
        let mut program_info = program_info_intermediate::<GarbageCollect>(task)?;

        if EventFilter::filter_many::<GarbageCollect>(&program_info.filters()) {
            return None;
        }

        let heap = (program_info.event_info.raw_event_data as *mut ArtHeap).read();

        *program_info.event_info = GarbageCollect {
            target_footprint: heap.target_footprint().ok()?,
            num_bytes_allocated: heap.num_bytes_allocated().ok()?,
            gc_cause: heap.gc_cause().ok()?,
            duration_ns: heap.duration_ns().ok()?,
            freed_objects: heap.freed_objects().ok()?,
            freed_bytes: heap.freed_bytes().ok()?,
            freed_los_objects: heap.freed_los_objects().ok()?,
            freed_los_bytes: heap.freed_los_bytes().ok()?,
            gcs_completed: heap.gcs_completed().ok()?,
        };

        program_info.submit()?;
    }
    Some(())
}
