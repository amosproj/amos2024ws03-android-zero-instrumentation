// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem::MaybeUninit;

use aya_ebpf::helpers::bpf_probe_read_kernel;
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::{FileDescriptorChange, FileDescriptorOp};

use super::SyscallProg;
use crate::{
    event_local::{EventLocal, EventLocalData, EventLocalValue},
    pipeline::{ProgramInfo, SysEnterInfo, SysExitInfo},
    syscalls,
};

#[repr(C)]
pub struct FdTrackingEnter {
    operation: FileDescriptorOp,
}

impl EventLocalData for FileDescriptorChange {
    type Data = FdTrackingEnter;
}

impl SyscallProg for FileDescriptorChange {
    fn enter<'a>(
        sys_enter: SysEnterInfo,
        _: ProgramInfo,
        mem: &'a mut MaybeUninit<EventLocal<Self>>,
    ) -> Option<&'a mut EventLocal<Self>> {
        initialize_fdtracking_enter(sys_enter.syscall_id, mem)
    }

    fn exit<'a>(
        sys_exit: SysExitInfo,
        _: ProgramInfo,
        entry: &EventLocalValue<Self>,
        mem: &'a mut MaybeUninit<Self>,
    ) -> Option<&'a Self> {
        initialize_fdtracking_exit(sys_exit.task, entry, mem)
    }
}

#[inline(always)]
fn initialize_fdtracking_enter(
    syscall_id: i64,
    fdtracking_data: &mut MaybeUninit<EventLocal<FileDescriptorChange>>,
) -> Option<&mut EventLocal<FileDescriptorChange>> {
    let ptr = fdtracking_data.as_mut_ptr();

    unsafe {
        (&raw mut (*ptr).data.operation).write(get_file_op(syscall_id)?);

        Some(fdtracking_data.assume_init_mut())
    }
}

#[inline(always)]
fn initialize_fdtracking_exit<'a>(
    task: TaskStruct,
    fdtracking_enter: &EventLocalValue<FileDescriptorChange>,
    fdtracking_data: &'a mut MaybeUninit<FileDescriptorChange>,
) -> Option<&'a FileDescriptorChange> {
    let ptr = fdtracking_data.as_mut_ptr();

    unsafe {
        (&raw mut (*ptr).open_fds).write(get_open_fds(task)?);
        (&raw mut (*ptr).operation).write(fdtracking_enter.data.operation);

        Some(fdtracking_data.assume_init_ref())
    }
}

pub fn get_file_op(syscall_number: i64) -> Option<FileDescriptorOp> {
    match syscall_number {
        | syscalls::SYS_pipe2
        | syscalls::SYS_pidfd_getfd
        | syscalls::SYS_pidfd_open
        | syscalls::SYS_perf_event_open
        | syscalls::SYS_signalfd4
        | syscalls::SYS_socket
        | syscalls::SYS_socketpair
        | syscalls::SYS_userfaultfd
        | syscalls::SYS_timerfd_create
        | syscalls::SYS_memfd_create
        | syscalls::SYS_landlock_create_ruleset
        | syscalls::SYS_io_uring_setup
        | syscalls::SYS_inotify_init1
        | syscalls::SYS_epoll_create1
        | syscalls::SYS_eventfd2
        | syscalls::SYS_execve
        | syscalls::SYS_execveat
        | syscalls::SYS_fanotify_init
        | syscalls::SYS_fcntl
        | syscalls::SYS_dup
        | syscalls::SYS_dup3
        | syscalls::SYS_openat
        | syscalls::SYS_openat2
        | syscalls::SYS_open_by_handle_at
        | syscalls::SYS_name_to_handle_at
        | syscalls::SYS_open_tree
        | syscalls::SYS_clone
        | syscalls::SYS_clone3
        | syscalls::SYS_bpf
        | syscalls::SYS_accept4
        | syscalls::SYS_accept => Some(FileDescriptorOp::Open),
        #[cfg(bpf_target_arch = "x86_64")]
        syscalls::SYS_pipe
        | syscalls::SYS_signalfd
        | syscalls::SYS_memfd_secret
        | syscalls::SYS_inotify_init
        | syscalls::SYS_epoll_create
        | syscalls::SYS_eventfd
        | syscalls::SYS_fork
        | syscalls::SYS_dup2
        | syscalls::SYS_open
        | syscalls::SYS_creat => Some(FileDescriptorOp::Open),
        syscalls::SYS_close | syscalls::SYS_close_range => Some(FileDescriptorOp::Close),
        _ => None,
    }
}

#[inline(always)]
pub fn get_open_fds(task: TaskStruct) -> Option<u64> {
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
