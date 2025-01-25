// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{
    env::current_exe,
    fs, io,
    mem::{self},
    os::{
        fd::{AsFd, AsRawFd},
        unix::{ffi::OsStrExt, process::parent_id},
    },
    process::id,
};

use aya::{
    maps::{HashMap, RingBuf},
    programs::RawTracePoint,
    Ebpf, EbpfLoader,
};
use aya_log::EbpfLogger;
use aya_obj::generated::{bpf_attr, bpf_cmd::BPF_PROG_TEST_RUN};
use bytemuck::checked;
use ebpf_types::{Event, EventKind, ProcessContext, TaskContext};
use libc::{syscall, SYS_bpf, SYS_gettid};

const PROG_BYTES: &[u8] = aya::include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ebpf.o"));

fn setup() -> Ebpf {
    let mut ebpf = EbpfLoader::default().load(PROG_BYTES).unwrap();

    EbpfLogger::init(&mut ebpf).unwrap();

    ebpf
}

#[test_log::test(tokio::test)]
async fn prog_test_example() {
    let mut ebpf = setup();

    let prog: &mut RawTracePoint = ebpf
        .program_mut("task_info_test")
        .unwrap()
        .try_into()
        .unwrap();

    prog.load().unwrap();

    let fd = prog.fd().unwrap().as_fd().as_raw_fd();

    let mut attr = unsafe { mem::zeroed::<bpf_attr>() };

    attr.test.prog_fd = fd as u32;
    attr.test.ctx_in = 0;
    attr.test.ctx_size_in = 0;

    let ret = unsafe { syscall(SYS_bpf, BPF_PROG_TEST_RUN, &mut attr, size_of::<bpf_attr>()) };

    if ret < 0 {
        panic!("Failed to run test: {:?}", io::Error::last_os_error());
    }

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
async fn test_vfs_write() {
    let mut ebpf = setup();

    let prog: &mut RawTracePoint = ebpf
        .program_mut("vfs_write_test")
        .unwrap()
        .try_into()
        .unwrap();

    prog.load().unwrap();

    let fd = prog.fd().unwrap().as_fd().as_raw_fd();

    let mut attr = unsafe { mem::zeroed::<bpf_attr>() };

    let args = [0u64, 0, 66, 0];

    attr.test.prog_fd = fd as u32;
    attr.test.ctx_in = args.as_ptr() as u64;
    attr.test.ctx_size_in = 4 * 8;

    let ret = unsafe { syscall(SYS_bpf, BPF_PROG_TEST_RUN, &mut attr, size_of::<bpf_attr>()) };

    if ret < 0 {
        panic!("Failed to run test: {:?}", io::Error::last_os_error());
    }

    let mut events: RingBuf<_> = ebpf.take_map("EVENTS").unwrap().try_into().unwrap();
    let event = events.next().unwrap();
    let event = checked::from_bytes::<Event>(&*event);

    let kind = match event.kind {
        EventKind::VfsWrite(kind) => kind,
        _ => panic!("Expected VfsWrite"),
    };

    assert_eq!(kind.bytes_written, 66);
}

#[test_log::test(tokio::test)]
async fn test_bin_path() {
    let mut ebpf = setup();

    let prog: &mut RawTracePoint = ebpf
        .program_mut("process_info_test")
        .unwrap()
        .try_into()
        .unwrap();

    prog.load().unwrap();

    let fd = prog.fd().unwrap().as_fd().as_raw_fd();

    let mut attr = unsafe { mem::zeroed::<bpf_attr>() };

    attr.test.prog_fd = fd as u32;
    attr.test.ctx_in = 0;
    attr.test.ctx_size_in = 0;

    let ret = unsafe { syscall(SYS_bpf, BPF_PROG_TEST_RUN, &mut attr, size_of::<bpf_attr>()) };

    if ret < 0 {
        panic!("Failed to run test: {:?}", io::Error::last_os_error());
    }

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
