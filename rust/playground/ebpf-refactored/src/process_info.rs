// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::option::Option::Some;

use aya_ebpf::{
    bindings::BPF_NOEXIST,
    helpers::bpf_probe_read_user_buf,
    macros::map,
    maps::{LruHashMap, PerCpuArray},
};
use ebpf_types::ProcessContext;
use relocation_helpers::TaskStruct;

use crate::{bounds_check::EbpfBoundsCheck, path::read_path_to_buf_with_default};

#[map]
static PROCESS_INFO: LruHashMap<u32, ProcessContext> = LruHashMap::with_max_entries(10240, 0);

#[map]
static PROCESS_INFO_SCRATCH: PerCpuArray<ProcessContext> = PerCpuArray::with_max_entries(1, 0);

#[inline(always)]
pub fn process_info_from_task(task: TaskStruct) -> Option<*mut ProcessContext> {
    let pid = task.tgid().ok()?;

    if let Some(ctx) = PROCESS_INFO.get_ptr_mut(&pid) {
        return Some(ctx);
    }

    let x = PROCESS_INFO_SCRATCH.get_ptr_mut(0)?;
    unsafe { core::ptr::write_bytes(x, 0, 1) };

    PROCESS_INFO
        .insert(&pid, unsafe { &*x }, BPF_NOEXIST as u64)
        .ok()?;

    let process_ctx = PROCESS_INFO.get_ptr_mut(&pid)?;
    let process_ctx = unsafe { &mut *process_ctx };

    let mm = task.mm().ok()?;
    let arg_start = mm.arg_start().ok()?;
    let arg_end = mm.arg_end().ok()?;

    let len = unsafe { ((arg_end - arg_start) as usize).bounded::<256>()? };
    let dst = &mut process_ctx.cmdline[..len];
    unsafe { bpf_probe_read_user_buf(arg_start as *mut u8, dst).ok() };

    let exe = mm.exe_file().ok()?;
    let exe_path = exe.f_path();

    read_path_to_buf_with_default(exe_path, &mut process_ctx.exe_path)?;

    Some(process_ctx)
}
