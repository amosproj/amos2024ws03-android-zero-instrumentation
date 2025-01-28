// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{
    ffi::CStr,
    fs::read_dir,
    io,
    mem::{self},
    os::fd::{AsFd, AsRawFd, RawFd},
    process::id,
};

use aya::{maps::RingBuf, programs::RawTracePoint, Ebpf, EbpfLoader};
use aya_ebpf::bindings::pt_regs;
use aya_log::EbpfLogger;
use aya_obj::generated::{bpf_attr, bpf_cmd::BPF_PROG_TEST_RUN};
use bytemuck::{checked, CheckedBitPattern};
use ebpf_types::{
    Blocking, Event, EventKind, FileDescriptorChange, FileDescriptorOp, Signal, Write, WriteSource,
};
use libc::{syscall, SYS_bpf, SYS_futex, SYS_gettid, SYS_kill, SYS_open, SYS_write};

const PROG_BYTES: &[u8] = aya::include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ebpf.o"));

fn setup() -> Ebpf {
    let mut ebpf = EbpfLoader::default().load(PROG_BYTES).unwrap();

    EbpfLogger::init(&mut ebpf).unwrap();

    ebpf
}

fn load_tracepoint(ebpf: &mut Ebpf, name: &str) -> RawFd {
    let prog: &mut RawTracePoint = ebpf.program_mut(name).unwrap().try_into().unwrap();

    prog.load().unwrap();

    prog.fd().unwrap().as_fd().as_raw_fd()
}

fn prog_run(fd: RawFd, args: &[u64]) -> Result<i64, io::Error> {
    let mut attr = unsafe { mem::zeroed::<bpf_attr>() };

    attr.test.prog_fd = fd as u32;
    attr.test.ctx_in = args.as_ptr() as u64;
    attr.test.ctx_size_in = args.len() as u32 * 8;

    let ret = unsafe { syscall(SYS_bpf, BPF_PROG_TEST_RUN, &mut attr, size_of::<bpf_attr>()) };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

fn get_event<T: CheckedBitPattern + 'static>(ebpf: &mut Ebpf) -> Box<Event<T>> {
    let mut map: RingBuf<_> = ebpf.map_mut("EVENT_RB").unwrap().try_into().unwrap();
    let event = map.next().unwrap();
    let event = checked::from_bytes::<Event<T>>(&event);
    Box::from(*event)
}

#[derive(Debug, Clone, Copy, Default)]
struct PtRegs {
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
    ret: u64,
    syscall: u64,
}

impl PtRegs {
    fn build(self) -> pt_regs {
        let mut regs = unsafe { mem::zeroed::<pt_regs>() };
        regs.orig_rax = self.syscall;
        regs.rdi = self.arg1;
        regs.rsi = self.arg2;
        regs.rdx = self.arg3;
        regs.r10 = self.arg4;
        regs.r8 = self.arg5;
        regs.r9 = self.arg6;
        regs.rax = self.ret;
        regs
    }
}

#[test_log::test(tokio::test)]
async fn test_write() {
    let mut ebpf = setup();
    let enter_fd = load_tracepoint(&mut ebpf, "sys_enter_write");
    let exit_fd = load_tracepoint(&mut ebpf, "sys_exit_write");

    let syscall_id = SYS_write as u64;
    let bytes_written = 66;
    let file_descriptor = enter_fd as u64;
    let ret_value = 0;

    let pt_regs_enter = PtRegs {
        arg1: file_descriptor,
        arg3: bytes_written,
        ..Default::default()
    }
    .build();
    let pt_regs_exit = PtRegs {
        ret: ret_value,
        syscall: syscall_id,
        ..Default::default()
    }
    .build();

    let _ = prog_run(enter_fd, &[&raw const pt_regs_enter as u64, syscall_id]).unwrap();
    let _ = prog_run(exit_fd, &[&raw const pt_regs_exit as u64, ret_value]).unwrap();

    let event = get_event::<Write>(&mut ebpf);
    assert_eq!(event.context.task.pid, id());
    assert_eq!(event.context.task.tid, unsafe {
        libc::syscall(SYS_gettid) as u32
    });
    assert!(matches!(event.data.source, WriteSource::Write));
    assert_eq!(event.data.bytes_written, bytes_written);
    assert_eq!(event.data.file_descriptor, file_descriptor);
    let file_path = CStr::from_bytes_until_nul(&event.data.file_path[..])
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(file_path, "bpf-prog");
}

#[test_log::test(tokio::test)]
async fn test_blocking() {
    let mut ebpf = setup();
    let enter_fd = load_tracepoint(&mut ebpf, "sys_enter_blocking");
    let exit_fd = load_tracepoint(&mut ebpf, "sys_exit_blocking");

    let syscall = SYS_futex as u64;
    let ret = 0;

    let pt_regs_enter = PtRegs {
        ..Default::default()
    }
    .build();
    let pt_regs_exit = PtRegs {
        ret,
        syscall,
        ..Default::default()
    }
    .build();

    let _ = prog_run(enter_fd, &[&raw const pt_regs_enter as u64, syscall]).unwrap();
    let _ = prog_run(exit_fd, &[&raw const pt_regs_exit as u64, ret]).unwrap();

    let event = get_event::<Blocking>(&mut ebpf);
    assert!(matches!(event.kind, EventKind::Blocking));
    assert_eq!(event.data.syscall_id, syscall);
    assert!(event.data.duration > 0);
}

#[test_log::test(tokio::test)]
async fn test_signal() {
    let mut ebpf = setup();
    let enter_fd = load_tracepoint(&mut ebpf, "sys_enter_signal");
    let exit_fd = load_tracepoint(&mut ebpf, "sys_exit_signal");

    let syscall = SYS_kill as u64;
    let ret = 0;
    let target_pid = 123;
    let signal = 1;

    let pt_regs_enter = PtRegs {
        arg1: target_pid as u64,
        arg2: signal as u64,
        ..Default::default()
    }
    .build();
    let pt_regs_exit = PtRegs {
        ret,
        syscall,
        ..Default::default()
    };

    let _ = prog_run(enter_fd, &[&raw const pt_regs_enter as u64, syscall]).unwrap();
    let _ = prog_run(exit_fd, &[&raw const pt_regs_exit as u64, ret]).unwrap();

    let event = get_event::<Signal>(&mut ebpf);
    assert!(matches!(event.kind, EventKind::Signal));
    assert_eq!(event.data.target_pid, target_pid);
    assert_eq!(event.data.signal, signal);
}

#[test_log::test(tokio::test)]
async fn test_fdtracking() {
    let mut ebpf = setup();

    let enter_fd = load_tracepoint(&mut ebpf, "sys_enter_fdtracking");
    let exit_fd = load_tracepoint(&mut ebpf, "sys_exit_fdtracking");

    let syscall = SYS_open as u64;
    let ret = 0;

    let pt_regs_enter = PtRegs {
        ..Default::default()
    }
    .build();
    let pt_regs_exit = PtRegs {
        ret,
        syscall,
        ..Default::default()
    }
    .build();

    let _ = prog_run(enter_fd, &[&raw const pt_regs_enter as u64, syscall]).unwrap();
    let _ = prog_run(exit_fd, &[&raw const pt_regs_exit as u64, ret]).unwrap();

    let event = get_event::<FileDescriptorChange>(&mut ebpf);
    assert!(matches!(event.kind, EventKind::FileDescriptorChange));
    assert!(matches!(event.data.operation, FileDescriptorOp::Open));

    let open_fds = read_dir("/proc/self/fd").unwrap().count() as u64 - 1;
    assert_eq!(event.data.open_fds, open_fds);
}
