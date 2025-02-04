// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem::MaybeUninit;

use aya_ebpf::PtRegs;
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::{Write, WriteSource};

use super::SyscallProg;
use crate::{
    event_local::{EventLocal, EventLocalData, EventLocalValue},
    path::{get_path_from_fd, read_path_to_buf_with_default},
    pipeline::{ProgramInfo, SysEnterInfo, SysExitInfo},
    syscalls,
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WriteEntryData {
    pub source: WriteSource,
    pub file_descriptor: u64,
    pub bytes_written: u64,
}

impl EventLocalData for Write {
    type Data = WriteEntryData;
}

impl SyscallProg for Write {
    fn enter<'a>(
        sys_enter: SysEnterInfo,
        _: ProgramInfo,
        mem: &'a mut MaybeUninit<EventLocal<Self>>,
    ) -> Option<&'a mut EventLocal<Self>> {
        initialize_write_enter(sys_enter.syscall_id, sys_enter.pt_regs, mem)
    }

    fn exit<'a>(
        sys_exit: SysExitInfo,
        _: ProgramInfo,
        entry: &EventLocalValue<Self>,
        mem: &'a mut MaybeUninit<Self>,
    ) -> Option<&'a Self> {
        initialize_write_exit(sys_exit.task, entry, mem)
    }
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
fn initialize_write_enter(
    syscall_id: i64,
    pt_regs: PtRegs,
    write_data: &mut MaybeUninit<EventLocal<Write>>,
) -> Option<&mut EventLocal<Write>> {
    let ptr = write_data.as_mut_ptr();

    unsafe {
        (&raw mut (*ptr).data.source).write(write_syscall_to_write_source(syscall_id)?);
        (&raw mut (*ptr).data.file_descriptor).write(pt_regs.arg::<*const u64>(0)? as u64);
        (&raw mut (*ptr).data.bytes_written).write(pt_regs.arg::<*const u64>(2)? as u64);

        Some(write_data.assume_init_mut())
    }
}

fn initialize_write_exit<'a>(
    task: TaskStruct,
    write_entry: &EventLocalValue<Write>,
    write_data: &'a mut MaybeUninit<Write>,
) -> Option<&'a Write> {
    let path = get_path_from_fd(write_entry.data.file_descriptor, task)?;

    let ptr = write_data.as_mut_ptr();
    unsafe {
        read_path_to_buf_with_default(path, &mut (*ptr).file_path)?;
        (&raw mut (*ptr).source).write(write_entry.data.source);
        (&raw mut (*ptr).bytes_written).write(write_entry.data.bytes_written);
        (&raw mut (*ptr).file_descriptor).write(write_entry.data.file_descriptor);

        if (*ptr).file_path[0] != b'/' {
            return None;
        }
        if &(*ptr).file_path[0..4] == b"/dev" {
            return None;
        }
        if &(*ptr).file_path[0..5] == b"/proc" {
            return None;
        }

        Some(write_data.assume_init_ref())
    }
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
