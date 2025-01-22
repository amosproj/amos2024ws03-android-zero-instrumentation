use core::option::Option::Some;

use aya_ebpf::{
    bindings::{task_struct, BPF_NOEXIST},
    macros::map,
    maps::{LruHashMap, PerCpuArray},
};
use ebpf_types::TaskContext;

use crate::relocation_helpers::{
    mm_struct_arg_end, mm_struct_arg_start, proc_cmdline, task_struct_comm,
    task_struct_group_leader, task_struct_mm, task_struct_pid, task_struct_real_parent,
    task_struct_tgid,
};

#[map]
static TASK_INFO: LruHashMap<u32, TaskContext> = LruHashMap::with_max_entries(10240, 0);

#[map]
static TASK_INFO_SCRATCH: PerCpuArray<TaskContext> = PerCpuArray::with_max_entries(1, 0);

/// # SAFETY
///
/// Must point to a valid `task_struct` struct.
pub unsafe fn task_info_from_task(task: *mut task_struct) -> Option<*mut TaskContext> {
    let tid = task_struct_pid(task).ok()? as u32;

    if let Some(ctx) = TASK_INFO.get_ptr_mut(&tid) {
        return Some(ctx);
    }

    let x = TASK_INFO_SCRATCH.get_ptr_mut(0)?;
    core::ptr::write_bytes(x, 0, 1);

    TASK_INFO.insert(&tid, &*x, BPF_NOEXIST as u64).ok()?;

    let ctx = TASK_INFO.get_ptr_mut(&tid)?;
    let ctx = &mut *ctx;

    ctx.pid = task_struct_tgid(task).ok()? as u32;
    ctx.tid = tid;

    let leader = task_struct_group_leader(task).ok()?;
    let parent = task_struct_real_parent(leader).ok()?;
    let parent_process = task_struct_group_leader(parent).ok()?;

    ctx.ppid = task_struct_pid(parent_process).ok()? as u32;
    task_struct_comm(task, &mut ctx.comm).ok()?;

    let mm = task_struct_mm(task).ok()?;
    let arg_start = mm_struct_arg_start(mm).ok()?;
    let arg_end = mm_struct_arg_end(mm).ok()?;

    proc_cmdline(arg_start, arg_end, &mut ctx.cmdline).ok()?;

    Some(ctx)
}
