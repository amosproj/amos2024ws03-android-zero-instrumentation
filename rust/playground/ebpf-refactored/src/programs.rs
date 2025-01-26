// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT


#[cfg(bpf_target_arch = "x86_64")]
use aya_ebpf::bindings::pt_regs;
#[cfg(bpf_target_arch = "aarch64")]
use aya_ebpf::bindings::user_pt_regs as pt_regs;
use aya_ebpf::{
    helpers::{
        bpf_get_current_task, bpf_ktime_get_ns,
    },
    macros::{map, raw_tracepoint},
    maps::{ProgramArray, RingBuf},
    programs::RawTracePointContext,
    EbpfContext, PtRegs,
};
use aya_log_ebpf::info;
use ebpf_types::{Event, EventContext, EventKind, ProcessContext, TaskContext, Write, WriteSource};
use relocation_helpers::TaskStruct;

use crate::{
    path::{
        get_path_from_fd, read_path_to_buf_with_default,
    },
    process_info::process_info_from_task,
    task_info::task_info_from_task,
};

#[raw_tracepoint]
fn task_info_test(_ctx: RawTracePointContext) -> Option<*mut TaskContext> {
    unsafe { task_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _)) }
}

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(8192, 0);

#[raw_tracepoint]
fn write_test(ctx: RawTracePointContext) -> Option<()> {
    let mut entry = EVENTS.reserve::<Event<Write>>(0)?;
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
unsafe fn try_vfs_write(ctx: &RawTracePointContext, entry: *mut Event<Write>) -> Option<()> {
    let task = unsafe { TaskStruct::new(bpf_get_current_task() as *mut _) };

    let task_context_src = task_info_from_task(task)?;
    let write_info = extract_write_syscall(ctx)?;
    let path = get_path_from_fd(write_info.fd, task)?;

    let entry = &mut *entry;

    entry.context = EventContext {
        task: *task_context_src,
        timestamp: bpf_ktime_get_ns(),
    };
    entry.kind = EventKind::Write;
    entry.data.bytes_written = write_info.bytes_written;
    entry.data.source = write_info.source;
    read_path_to_buf_with_default(path, &mut entry.data.file_path)?;

    Some(())
}

#[raw_tracepoint]
fn process_info_test(_ctx: RawTracePointContext) -> Option<*mut ProcessContext> {
    unsafe { process_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _)) }
}

#[map]
static WRITE_TAILCALLS: ProgramArray = ProgramArray::with_max_entries(1024, 0);

struct WriteSyscallInfo {
    source: WriteSource,
    bytes_written: u64,
    fd: u64,
}

/*
 * long sys_write(unsigned int fd, const char __user *buf,size_t count);
 * long sys_writev(unsigned long fd, const struct iovec __user *vec, unsigned long vlen);
 * long sys_pwrite64(unsigned int fd, const char __user *buf, size_t count, loff_t pos);
 * long sys_pwritev(unsigned long fd, const struct iovec __user *vec, unsigned long vlen, unsigned long pos_l, unsigned long pos_h);
 * long sys_pwritev2(unsigned long fd, const struct iovec __user *vec, unsigned long vlen, unsigned long pos_l, unsigned long pos_h, rwf_t flags);
 *
 * TODO: if we want to support 32 bit archs we need to change this
 *
 * struct iovec {
 *   iov_base: *const u8,
 *   iov_len: u64,
 * }
 *
 * u64 sys_write(fd: u32, buf: *const i8, count: u64)
 * u64 sys_writev(fd: u64, buf: *const iovec, vlen: u64)
 * u64 sys_pwrite64(fd: u32, buf: *const i64, count: u64, pos: u64)
 * u64 sys_pwritev(fd: u64, buf: *const iovec, vlen: u64, pos_l: u64, pos_h: u64)
 * u64 sys_pwritev2(fd: u64, buf: *const iovec, vlen: u64, pos_l: u64, pos_h: u64, flags: i32)
 */
fn extract_write_syscall(ctx: &RawTracePointContext) -> Option<WriteSyscallInfo> {
    let id = unsafe { *(ctx.as_ptr().byte_add(8) as *const i32) } as i64;
    let source = write_syscall_to_write_source(id)?;
    let pt_regs = PtRegs::new(unsafe { *(ctx.as_ptr() as *const *mut pt_regs) });
    let fd = pt_regs.arg::<*const u64>(0)? as u64;
    let count = pt_regs.arg::<*const u64>(2)? as u64;

    Some(WriteSyscallInfo {
        source,
        bytes_written: count,
        fd,
    })
}

fn write_syscall_to_write_source(syscall: i64) -> Option<WriteSource> {
    let source = match syscall {
        syscalls::SYS_write => WriteSource::Write,
        syscalls::SYS_writev => WriteSource::WriteV,
        syscalls::SYS_pwritev => WriteSource::WriteV,
        syscalls::SYS_pwrite64 => WriteSource::Write64,
        syscalls::SYS_pwritev2 => WriteSource::WriteV2,
        _ => return None,
    };
    Some(source)
}

mod syscalls {
    #[cfg(bpf_target_arch = "aarch64")]
    pub use syscall_numbers::aarch64::*;
    #[cfg(bpf_target_arch = "x86_64")]
    pub use syscall_numbers::x86_64::*;
}
