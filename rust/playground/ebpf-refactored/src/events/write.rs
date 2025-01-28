// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::PtRegs;
use ebpf_relocation_helpers::TaskStruct;
use ebpf_types::{Write, WriteSource};

use crate::{
    path::{get_path_from_fd, read_path_to_buf_with_default},
    syscalls,
};

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
pub fn initialize_write_enter(
    syscall_id: i64,
    pt_regs: PtRegs,
    write_data: &mut Write,
) -> Option<()> {
    write_data.source = write_syscall_to_write_source(syscall_id)?;
    write_data.bytes_written = pt_regs.arg::<*const u64>(2)? as u64;
    write_data.file_descriptor = pt_regs.arg::<*const u64>(0)? as u64;

    Some(())
}

pub fn initialize_write_exit(task: TaskStruct, write_data: &mut Write) -> Option<()> {
    let path = get_path_from_fd(write_data.file_descriptor, task)?;
    read_path_to_buf_with_default(path, &mut write_data.file_path)?;
    Some(())
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
