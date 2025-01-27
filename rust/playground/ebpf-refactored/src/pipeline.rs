// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem;

use aya_ebpf::{
    bindings::pt_regs,
    helpers::{bpf_get_current_task, bpf_ktime_get_ns},
    macros::{map, raw_tracepoint},
    maps::{Array, HashMap, PerCpuArray, RingBuf},
    programs::RawTracePointContext,
    EbpfContext, PtRegs,
};
use ebpf_types::{
    Blocking, Event, EventContext, EventData, FileDescriptorChange, FilterConfig, ProcessContext,
    Signal, TaskContext, Write,
};
use relocation_helpers::TaskStruct;

use crate::{
    blocking::{initialize_blocking_enter, initialize_blocking_exit},
    fdtracking::{initialize_fdtracking_enter, initialize_fdtracking_exit},
    process_info::process_info_from_task,
    programs::{filter, FILTER_CONFIG},
    signal::initialize_signal_enter,
    task_info::task_info_from_task,
    write::{initialize_write_enter, initialize_write_exit},
};

struct ProgramInfo<'a, T> {
    task_context: &'a TaskContext,
    process_context: &'a ProcessContext,
    filter_config: &'a FilterConfig,
    event_info: EventInfo<'a, T>,
}

impl<T: EventData + Clone + Copy + 'static> ProgramInfo<'_, T> {
    fn submit(self) -> Option<()> {
        let mut entry = EVENT_RB.reserve::<Event<T>>(0)?;
        let entry_mut = unsafe { entry.assume_init_mut() };
        entry_mut.kind = T::EVENT_KIND;
        entry_mut.context = EventContext {
            timestamp: unsafe { bpf_ktime_get_ns() },
            task: *self.task_context,
        };
        entry_mut.data = *self.event_info.event_data;
        entry.submit(0);

        Some(())
    }

    fn submit_intermediate(&self) -> Option<()> {
        EVENT_BRIDGE
            .insert(
                &self.task_context.tid,
                unsafe { &*self.event_info.raw_event_data },
                0,
            )
            .ok()?;

        Some(())
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

struct EventInfo<'a, T> {
    event_data: &'a mut T,
    raw_event_data: *mut RawEventData,
}

struct RawEventData {
    data: [u8; max_raw_event_data_size()],
}

#[map]
static EVENT_BUFFER: PerCpuArray<RawEventData> = PerCpuArray::with_max_entries(1, 0);

#[map]
static EVENT_BRIDGE: HashMap<u32, RawEventData> = HashMap::with_max_entries(10240, 0);

#[map]
static EVENT_RB: RingBuf = RingBuf::with_byte_size(8192 * 1024, 0);

#[map]
static GLOBAL_BLOCKING_THRESHOLD: Array<u64> = Array::with_max_entries(1, 0);

unsafe fn program_info_base<T: EventData>(
    task: TaskStruct,
    raw_event_data: *mut RawEventData,
) -> Option<ProgramInfo<'static, T>> {
    let event_data = &mut *((*raw_event_data).data[..mem::size_of::<T>()].as_mut_ptr() as *mut T);
    let event_info = EventInfo {
        event_data,
        raw_event_data,
    };
    Some(ProgramInfo {
        task_context: &*task_info_from_task(task)?,
        process_context: &*process_info_from_task(task)?,
        filter_config: FILTER_CONFIG.get(T::EVENT_KIND as u32)?,
        event_info,
    })
}

unsafe fn program_info<T: EventData>(task: TaskStruct) -> Option<ProgramInfo<'static, T>> {
    let raw_event_data = EVENT_BUFFER.get_ptr_mut(0)?;
    let program_info = program_info_base::<T>(task, raw_event_data)?;
    let _ = EVENT_BRIDGE.remove(&program_info.task_context.tid);
    Some(program_info)
}

unsafe fn program_info_intermediate<T: EventData>(
    task: TaskStruct,
) -> Option<ProgramInfo<'static, T>> {
    let raw_event_data = EVENT_BRIDGE.get_ptr_mut(&task.pid().ok()?)?;
    let program_info = program_info_base::<T>(task, raw_event_data)?;
    let _ = EVENT_BRIDGE.remove(&program_info.task_context.tid);
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
        let program_info = enter_info.program_info::<Write>()?;

        if filter::<Write>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_write_enter(
            enter_info.syscall_id,
            enter_info.pt_regs,
            program_info.event_info.event_data,
        )?;

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_write(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        if exit_info.return_value != 0 {
            return None;
        }

        let program_info = exit_info.program_info::<Write>()?;

        if filter::<Write>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_write_exit(exit_info.task, program_info.event_info.event_data)?;

        program_info.submit()?;
    }
    Some(())
}

#[raw_tracepoint]
pub fn sys_enter_blocking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let enter_info = SysEnterInfo::new(&ctx);
        let program_info = enter_info.program_info::<Blocking>()?;

        if filter::<Blocking>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_blocking_enter(enter_info.syscall_id, program_info.event_info.event_data)?;

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_blocking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        let program_info = exit_info.program_info::<Blocking>()?;

        if filter::<Blocking>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_blocking_exit(program_info.event_info.event_data)?;

        if let Some(threshold) = GLOBAL_BLOCKING_THRESHOLD.get(0) {
            if program_info.event_info.event_data.duration <= *threshold {
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
        let program_info = enter_info.program_info::<Signal>()?;

        if filter::<Signal>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_signal_enter(
            enter_info.syscall_id,
            enter_info.pt_regs,
            program_info.event_info.event_data,
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

        if filter::<Signal>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
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
        let program_info = enter_info.program_info::<FileDescriptorChange>()?;

        if filter::<FileDescriptorChange>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_fdtracking_enter(enter_info.syscall_id, program_info.event_info.event_data)?;

        program_info.submit_intermediate()?;
    }

    Some(())
}

#[raw_tracepoint]
pub fn sys_exit_fdtracking(ctx: RawTracePointContext) -> Option<()> {
    unsafe {
        let exit_info = SysExitInfo::new(&ctx);
        let program_info = exit_info.program_info::<FileDescriptorChange>()?;

        if filter::<FileDescriptorChange>(
            program_info.filter_config,
            program_info.task_context,
            program_info.process_context,
        ) {
            return None;
        }

        initialize_fdtracking_exit(exit_info.task, program_info.event_info.event_data)?;

        program_info.submit()?;
    }
    Some(())
}
