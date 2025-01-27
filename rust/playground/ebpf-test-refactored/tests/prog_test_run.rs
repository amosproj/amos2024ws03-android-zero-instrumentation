// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{
    any::Any,
    env::current_exe,
    ffi::CStr,
    fs::{self, read_dir},
    io,
    mem::{self},
    os::{
        fd::{AsFd, AsRawFd, RawFd},
        unix::{ffi::OsStrExt, process::parent_id},
    },
    process::id,
};

use aya::{
    maps::{HashMap, RingBuf},
    programs::RawTracePoint,
    Ebpf, EbpfLoader,
};
use aya_ebpf::bindings::pt_regs;
use aya_log::EbpfLogger;
use aya_obj::generated::{bpf_attr, bpf_cmd::BPF_PROG_TEST_RUN};
use bytemuck::{checked, CheckedBitPattern};
use ebpf_types::{
    unpack_event, Blocking, Event, EventKind, FileDescriptorChange, FileDescriptorOp,
    GarbageCollect, Jni, ProcessContext, Signal, TaskContext, Write, WriteSource,
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

#[test_log::test(tokio::test)]
async fn prog_test_example() {
    let mut ebpf = setup();

    let fd = load_tracepoint(&mut ebpf, "task_info_test");

    let _ = prog_run(fd, &[0, 0]).unwrap();

    let map: HashMap<_, u32, TaskContext> = ebpf.map("TASK_INFO").unwrap().try_into().unwrap();

    let my_tid = unsafe { syscall(SYS_gettid) as u32 };

    let task_infos = map
        .iter()
        .map(Result::unwrap)
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(task_infos.len(), 1);
    let entry = task_infos.get(&my_tid).unwrap();

    assert_eq!(entry.pid, id());
    assert_eq!(entry.ppid, parent_id());

    let mut comm = [0u8; 16];

    let comm_vec = fs::read_to_string(format!("/proc/self/task/{}/comm", my_tid))
        .unwrap()
        .into_bytes();
    let len = comm_vec.len().min(16).saturating_sub(1); // newline
    comm[..len].copy_from_slice(&comm_vec[..len]);

    assert_eq!(entry.comm, comm);
}

#[test_log::test(tokio::test)]
async fn test_syswrite() {
    let mut ebpf = setup();

    let fd = load_tracepoint(&mut ebpf, "write_test");

    let mut regs = unsafe { mem::zeroed::<pt_regs>() };
    regs.rdi = fd as u64; // fd
    regs.rdx = 66; // bytes written

    let _ = prog_run(fd, &[&raw mut regs as u64, 1]).unwrap();

    let mut events: RingBuf<_> = ebpf.take_map("EVENTS").unwrap().try_into().unwrap();
    let event = events.next().unwrap();
    let event = checked::from_bytes::<Event<ebpf_types::Write>>(&event);

    let file_path = CStr::from_bytes_until_nul(&event.data.file_path[..]).unwrap();
    let file_path = file_path.to_str().unwrap();

    assert!(matches!(event.data.source, WriteSource::Write));
    assert_eq!(event.data.bytes_written, 66);
    assert_eq!(file_path, "bpf-prog");
}

#[test_log::test(tokio::test)]
async fn test_bin_path() {
    let mut ebpf = setup();
    let fd = load_tracepoint(&mut ebpf, "process_info_test");
    let _ = prog_run(fd, &[0, 0]).unwrap();

    let exe_path = {
        let mut tmp = [0u8; 4096];
        let exe = current_exe().unwrap();
        let exe_bytes = exe.as_os_str().as_bytes();
        let len = exe_bytes.len().min(4096);
        tmp[..len].copy_from_slice(&exe_bytes[..len]);
        tmp
    };
    let cmdline = {
        let mut tmp = [0u8; 256];
        let cmdline_vec = fs::read_to_string("/proc/self/cmdline")
            .unwrap()
            .into_bytes();
        let len = cmdline_vec.len().min(256);
        tmp[..len].copy_from_slice(&cmdline_vec[..len]);
        tmp
    };

    let map: HashMap<_, u32, ProcessContext> =
        ebpf.map("PROCESS_INFO").unwrap().try_into().unwrap();
    let my_pid = id();

    let entry = map.get(&my_pid, 0).unwrap();

    assert_eq!(entry.exe_path, exe_path);
    assert_eq!(entry.cmdline, cmdline);
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn test_new_fd() {
    let mut ebpf = setup();

    let fd = load_tracepoint(&mut ebpf, "file_descriptor_test");
    let mut regs = unsafe { mem::zeroed::<pt_regs>() };
    regs.orig_rax = SYS_open as u64;

    let _ = prog_run(fd, &[&raw mut regs as u64, 0]).unwrap();

    let mut events: RingBuf<_> = ebpf.take_map("EVENTS").unwrap().try_into().unwrap();
    let raw_event = events.next().unwrap();

    let event = checked::from_bytes::<Event<FileDescriptorChange>>(&raw_event);

    let open_fds = read_dir("/proc/self/fd").unwrap().count() as u64 - 1;

    assert_eq!(event.data.open_fds, open_fds);
    assert!(matches!(event.data.operation, FileDescriptorOp::Open));
}

#[test_log::test(tokio::test)]
async fn test_kill() {
    let mut ebpf = setup();

    let fd = load_tracepoint(&mut ebpf, "kill_test");
    let mut regs = unsafe { mem::zeroed::<pt_regs>() };

    let target_pid = 123;
    let signal = 1;

    regs.rdi = target_pid as u64;
    regs.rsi = signal as u64;

    let _ = prog_run(fd, &[&raw mut regs as u64, SYS_kill as u64]).unwrap();

    let mut events: RingBuf<_> = ebpf.take_map("EVENTS").unwrap().try_into().unwrap();
    let raw_event = events.next().unwrap();

    let event = unpack_event!(raw_event);
    let data = event.data.downcast_ref::<Signal>().unwrap();

    assert_eq!(data.target_pid, target_pid);
    assert_eq!(data.signal, signal);
}

#[test_log::test(tokio::test)]
async fn blocking_test() {
    let mut ebpf = setup();

    let enter_fd = load_tracepoint(&mut ebpf, "sys_enter_test");
    let exit_fd = load_tracepoint(&mut ebpf, "sys_exit_test");

    let mut regs = unsafe { mem::zeroed::<pt_regs>() };

    let _ = prog_run(enter_fd, &[&raw mut regs as u64, 1]).unwrap();

    regs.orig_rax = 1;
    let _ = prog_run(exit_fd, &[&raw mut regs as u64, 0]).unwrap();

    let mut events: RingBuf<_> = ebpf.take_map("EVENTS").unwrap().try_into().unwrap();
    let raw_event = events.next().unwrap();

    let event = unpack_event!(raw_event);
    let data = event.data.downcast_ref::<Blocking>().unwrap();

    assert_eq!(data.syscall_id, 1);
    assert!(data.duration > 0);
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
    assert!(matches!(event.kind, EventKind::Write));
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
