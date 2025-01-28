// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::{mem::MaybeUninit, ptr::write_bytes};

use aya_ebpf::helpers::{bpf_probe_read_kernel_buf, bpf_probe_read_user_buf};
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::{ProcessContext, TaskContext};

use crate::{
    bounds_check::EbpfBoundsCheck, cache::TryWithCache, path::read_path_to_buf_with_default,
    scratch::TryIntoMem,
};

impl TryIntoMem<TaskContext> for TaskStruct {
    #[inline(always)]
    fn convert_into_mem<'a>(
        &self,
        mem: &'a mut MaybeUninit<TaskContext>,
    ) -> Result<&'a mut TaskContext, i64> {
        task_get_task_context(self, mem)
    }
}

impl TryIntoMem<ProcessContext> for TaskStruct {
    #[inline(always)]
    fn convert_into_mem<'a>(
        &self,
        mem: &'a mut MaybeUninit<ProcessContext>,
    ) -> Result<&'a mut ProcessContext, i64> {
        task_get_process_context(self, mem)
    }
}

impl TryWithCache<u32, TaskContext> for TaskStruct {
    fn get_key(&self) -> Result<u32, i64> {
        self.pid()
    }
}

impl TryWithCache<u32, ProcessContext> for TaskStruct {
    fn get_key(&self) -> Result<u32, i64> {
        self.tgid()
    }
}

#[inline(always)]
fn task_get_task_context<'a>(
    task: &TaskStruct,
    mem: &'a mut MaybeUninit<TaskContext>,
) -> Result<&'a mut TaskContext, i64> {
    let ptr = mem.as_mut_ptr();

    let leader = task.group_leader()?;
    let parent = leader.real_parent()?;
    let parent_process = parent.group_leader()?;

    unsafe {
        (&raw mut (*ptr).pid).write(task.tgid()?);
        (&raw mut (*ptr).tid).write(task.pid()?);
        (&raw mut (*ptr).ppid).write(parent_process.pid()?);
        bpf_probe_read_kernel_buf(task.comm() as *const u8, &mut (*ptr).comm)?;

        Ok(mem.assume_init_mut())
    }
}

#[inline(always)]
fn task_get_process_context<'a>(
    task: &TaskStruct,
    mem: &'a mut MaybeUninit<ProcessContext>,
) -> Result<&'a mut ProcessContext, i64> {
    let mm = task.mm()?;
    let arg_start = mm.arg_start()?;
    let arg_end = mm.arg_end()?;
    let exe = mm.exe_file()?;
    let exe_path = exe.f_path();

    let len = (arg_end - arg_start) as usize;
    let len = unsafe { len.bounded::<256>().ok_or(-1)? };

    let ptr = mem.as_mut_ptr();
    unsafe {
        write_bytes(&raw mut (*ptr).cmdline, 0, 1);
        bpf_probe_read_user_buf(arg_start as *const u8, &mut (*ptr).cmdline[..len])?;

        write_bytes(&raw mut (*ptr).exe_path, 0, 1);
        read_path_to_buf_with_default(exe_path, &mut (*ptr).exe_path).ok_or(-1)?;

        Ok(mem.assume_init_mut())
    }
}
