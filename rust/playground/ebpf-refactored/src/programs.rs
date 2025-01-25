// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT


use aya_ebpf::{
    bindings::task_struct,
    helpers::{bpf_get_current_task, bpf_ktime_get_ns},
    macros::{map, raw_tracepoint},
    maps::{PerCpuArray, RingBuf},
    programs::RawTracePointContext,
    EbpfContext,
};
use aya_log_ebpf::info;
use ebpf_types::{Event, EventContext, EventKind, TaskContext, VfsWrite};

use crate::{
    path::{get_path_str, PATH_MAX},
    relocation_helpers::TaskStruct,
    task_info::task_info_from_task,
};

#[raw_tracepoint]
fn task_info_test(ctx: RawTracePointContext) -> Option<*mut TaskContext> {
    info!(&ctx, "task_info_test");
    unsafe { task_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _)) }
}

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(8192, 0);

#[raw_tracepoint]
fn vfs_write_test(ctx: RawTracePointContext) -> Option<()> {
    info!(&ctx, "vfs_write");
    let mut entry = EVENTS.reserve::<Event>(0)?;
    match unsafe { try_vfs_write(&ctx, entry.as_mut_ptr()) } {
        Some(_) => entry.submit(0),
        None => {
            info!(&ctx, "vfs_write discard");
            entry.discard(0)
        }
    }
    Some(())
}

#[inline(always)]
unsafe fn try_vfs_write(ctx: &RawTracePointContext, entry: *mut Event) -> Option<()> {
    let task_context_src = task_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _))?;
    let bytes_written = *(ctx.as_ptr().add(16) as *const u64);

    entry.write(Event {
        context: EventContext {
            task: *task_context_src,
            timestamp: bpf_ktime_get_ns(),
        },
        kind: EventKind::VfsWrite(VfsWrite { bytes_written }),
    });

    Some(())
}

#[raw_tracepoint]
fn bin_path_test(ctx: RawTracePointContext) -> Option<()> {
    let mut bin_path = BINARY_PATH.reserve::<[u8; 4096]>(0)?;
    match unsafe { try_bin_path((*bin_path.as_mut_ptr()).as_mut_slice()) } {
        Some(_) => bin_path.submit(0),
        None => {
            info!(&ctx, "bin_path discard");
            bin_path.discard(0)
        }
    }
    Some(())
}

#[map]
static COMPONENT_BUF: PerCpuArray<[u8; 8192]> = PerCpuArray::with_max_entries(1, 0);

#[map]
static BINARY_PATH: RingBuf = RingBuf::with_byte_size(4096 * 128, 0);

unsafe fn try_bin_path(dst: &mut [u8]) -> Option<()> {
    let task_struct = TaskStruct::new(bpf_get_current_task() as *mut task_struct);

    let mm_struct = task_struct.mm().ok()?;
    let exe_file = mm_struct.exe_file().ok()?;
    let f_path = exe_file.f_path();
    let buf = &mut *COMPONENT_BUF.get_ptr_mut(0)?;

    let offset = get_path_str(f_path, buf)?;

    let dst = dst.as_mut_ptr();
    let src = (*buf).get(offset..PATH_MAX)?.as_ptr();
    let count = PATH_MAX - offset;

    core::ptr::copy_nonoverlapping(src, dst, count);

    Some(())
}
