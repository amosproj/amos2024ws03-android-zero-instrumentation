// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::option::Option::Some;

use aya_ebpf::{
    bindings::BPF_NOEXIST,
    helpers::bpf_probe_read_kernel_buf,
    macros::map,
    maps::{LruHashMap, PerCpuArray},
};
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::TaskContext;

#[map]
static TASK_INFO: LruHashMap<u32, TaskContext> = LruHashMap::with_max_entries(10240, 0);

#[map]
static TASK_INFO_SCRATCH: PerCpuArray<TaskContext> = PerCpuArray::with_max_entries(1, 0);

#[inline(always)]
pub fn task_info_from_task(task: TaskStruct) -> Option<*mut TaskContext> {
    let tid = task.pid().ok()?;

    if let Some(ctx) = TASK_INFO.get_ptr_mut(&tid) {
        return Some(ctx);
    }

    let x = TASK_INFO_SCRATCH.get_ptr_mut(0)?;
    unsafe { core::ptr::write_bytes(x, 0, 1) };

    TASK_INFO
        .insert(&tid, unsafe { &*x }, BPF_NOEXIST as u64)
        .ok()?;

    let task_ctx = TASK_INFO.get_ptr_mut(&tid)?;
    let task_ctx = unsafe { &mut *task_ctx };

    task_ctx.pid = task.tgid().ok()?;
    task_ctx.tid = tid;

    let leader = task.group_leader().ok()?;
    let parent = leader.real_parent().ok()?;
    let parent_process = parent.group_leader().ok()?;

    task_ctx.ppid = parent_process.pid().ok()?;
    let comm_ptr = task.comm();
    unsafe {
        bpf_probe_read_kernel_buf(comm_ptr as *const _ as *const u8, &mut task_ctx.comm).ok()?
    };

    Some(task_ctx)
}
