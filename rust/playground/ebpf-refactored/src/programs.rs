// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

#[cfg(bpf_target_arch = "x86_64")]
use aya_ebpf::bindings::pt_regs;
#[cfg(bpf_target_arch = "aarch64")]
use aya_ebpf::bindings::user_pt_regs as pt_regs;
use aya_ebpf::{
    helpers::{bpf_get_current_task, bpf_ktime_get_ns, bpf_probe_read, bpf_probe_read_kernel},
    macros::{map, raw_tracepoint},
    maps::RingBuf,
    programs::RawTracePointContext,
    EbpfContext, PtRegs,
};
use aya_log_ebpf::info;
use ebpf_types::{
    Event, EventContext, FileDescriptorChange, FileDescriptorOp, ProcessContext, Signal,
    TaskContext, Write, WriteSource,
};
use relocation_helpers::TaskStruct;

use crate::{
    path::{get_path_from_fd, read_path_to_buf_with_default},
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

#[raw_tracepoint]
fn file_descriptor_test(ctx: RawTracePointContext) -> Option<()> {
    let mut entry = EVENTS.reserve::<Event<FileDescriptorChange>>(0)?;
    match unsafe { try_fd_change(&ctx, entry.as_mut_ptr()) } {
        Some(_) => entry.submit(0),
        None => {
            info!(&ctx, "fd change discard");
            entry.discard(0)
        }
    }
    Some(())
}

#[raw_tracepoint]
fn kill_test(ctx: RawTracePointContext) -> Option<()> {
    let mut entry = EVENTS.reserve::<Event<Signal>>(0)?;
    match unsafe { try_kill(&ctx, entry.as_mut_ptr()) } {
        Some(_) => entry.submit(0),
        None => {
            info!(&ctx, "kill discard");
            entry.discard(0)
        }
    }
    Some(())
}

unsafe fn try_kill(ctx: &RawTracePointContext, entry: *mut Event<Signal>) -> Option<()> {
    let id = unsafe { *(ctx.as_ptr().byte_add(8) as *const i32) } as i64;
    if id != syscalls::SYS_kill {
        return None;
    }
    let pt_regs = PtRegs::new(unsafe { *(ctx.as_ptr() as *const *mut pt_regs) });
    let target_pid = pt_regs.arg::<*const u64>(0)? as i32;
    let signal = pt_regs.arg::<*const u64>(1)? as u32;

    let task_info_src = task_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _))?;

    (*entry).context = EventContext {
        task: *task_info_src,
        timestamp: bpf_ktime_get_ns(),
    };
    (*entry).data = Signal { target_pid, signal };
    Some(())
}

unsafe fn try_fd_change(
    ctx: &RawTracePointContext,
    entry: *mut Event<FileDescriptorChange>,
) -> Option<()> {
    let task = TaskStruct::new(bpf_get_current_task() as *mut _);
    let raw_pt_regs = unsafe { *(ctx.as_ptr() as *const *mut pt_regs) };
    let syscall_id =
        unsafe { bpf_probe_read(&raw const (*raw_pt_regs).orig_rax as *const i64) }.ok()?;
    let filed_descriptor_op = get_file_op(syscall_id)?;
    let open_fds = get_open_fds(task)?;

    let task_context_src = task_info_from_task(task)?;

    (*entry).context = EventContext {
        task: *task_context_src,
        timestamp: bpf_ktime_get_ns(),
    };
    (*entry).data = FileDescriptorChange {
        operation: filed_descriptor_op,
        open_fds,
    };

    Some(())
}

#[inline(always)]
fn get_open_fds(task: TaskStruct) -> Option<u64> {
    let files = task.files().ok()?;
    let fdt = files.fdt().ok()?;
    let max_fds = fdt.max_fds().ok()?;
    let len = (max_fds / 64).min(1024) as usize;
    let open_fds = fdt.open_fds().ok()?;

    let mut count = 0;
    for i in 0..len {
        let bitmap = unsafe { bpf_probe_read_kernel(open_fds.add(i)).ok() }?;
        count += bitmap.count_ones() as u64;
    }

    Some(count)
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
    entry.data.bytes_written = write_info.bytes_written;
    entry.data.source = write_info.source;
    read_path_to_buf_with_default(path, &mut entry.data.file_path)?;

    Some(())
}

#[raw_tracepoint]
fn process_info_test(_ctx: RawTracePointContext) -> Option<*mut ProcessContext> {
    unsafe { process_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _)) }
}

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

fn get_file_op(syscall_number: i64) -> Option<FileDescriptorOp> {
    match syscall_number {
        syscalls::SYS_pipe
        | syscalls::SYS_pipe2
        | syscalls::SYS_pidfd_getfd
        | syscalls::SYS_pidfd_open
        | syscalls::SYS_perf_event_open
        | syscalls::SYS_signalfd
        | syscalls::SYS_signalfd4
        | syscalls::SYS_socket
        | syscalls::SYS_socketpair
        | syscalls::SYS_userfaultfd
        | syscalls::SYS_timerfd_create
        | syscalls::SYS_memfd_create
        | syscalls::SYS_memfd_secret
        | syscalls::SYS_landlock_create_ruleset
        | syscalls::SYS_io_uring_setup
        | syscalls::SYS_inotify_init
        | syscalls::SYS_inotify_init1
        | syscalls::SYS_epoll_create
        | syscalls::SYS_epoll_create1
        | syscalls::SYS_eventfd
        | syscalls::SYS_eventfd2
        | syscalls::SYS_execve
        | syscalls::SYS_execveat
        | syscalls::SYS_fanotify_init
        | syscalls::SYS_fcntl
        | syscalls::SYS_fork
        | syscalls::SYS_dup
        | syscalls::SYS_dup2
        | syscalls::SYS_dup3
        | syscalls::SYS_open
        | syscalls::SYS_openat
        | syscalls::SYS_creat
        | syscalls::SYS_openat2
        | syscalls::SYS_open_by_handle_at
        | syscalls::SYS_name_to_handle_at
        | syscalls::SYS_open_tree
        | syscalls::SYS_clone
        | syscalls::SYS_clone3
        | syscalls::SYS_bpf
        | syscalls::SYS_accept4
        | syscalls::SYS_accept => Some(FileDescriptorOp::Open),
        syscalls::SYS_close | syscalls::SYS_close_range => Some(FileDescriptorOp::Close),
        _ => None,
    }
}

mod syscalls {
    #[cfg(bpf_target_arch = "aarch64")]
    pub use syscall_numbers::aarch64::*;
    #[cfg(bpf_target_arch = "x86_64")]
    pub use syscall_numbers::x86_64::*;
}
